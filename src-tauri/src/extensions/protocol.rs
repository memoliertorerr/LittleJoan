//! The `ljext` scheme handler (glue). Tauri registers it for every webview; api.rs decides each
//! request, and this serves what it allows: an extension's files and the SDK with its CSP and
//! headers (csp.rs), and the API's methods. Refusals carry no extension data and a policy that
//! allows nothing. Each request runs on a blocking worker, off the engine's thread.

use std::borrow::Cow;
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Runtime, UriSchemeContext, UriSchemeResponder};

use super::api::{self, Context, Incoming, Method, Route};
use super::manifest::Permission;
use super::registry::{self, Extension};
use super::{clipboard, csp, limits, paths, session, storage, theme, urls};
use crate::bar;
use crate::keys::Platform;
use crate::smoke;
use crate::state;

/// The SDK every panel loads from `/__lj/sdk/v1.js`.
pub const SDK: &str = include_str!("../../../extensions/sdk/v1.js");
/// Where the served SDK carries the opening's token (`X-LJ-Api`).
pub const TOKEN_PLACEHOLDER: &str = "__LJ_SESSION_TOKEN__";

/// The handler: `tauri::Builder::register_asynchronous_uri_scheme_protocol(urls::SCHEME, …)`.
pub fn handle<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let incoming = Incoming {
        webview: ctx.webview_label().to_owned(),
        method: request.method().as_str().to_owned(),
        uri: request.uri().to_string(),
        headers: request
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_ascii_lowercase(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect(),
    };
    let body = request.into_body();
    // Files, storage and the clipboard can take a moment: never on the engine's thread.
    tauri::async_runtime::spawn_blocking(move || {
        responder.respond(respond(&app, &incoming, &body));
    });
}

fn respond<R: Runtime>(
    app: &AppHandle<R>,
    incoming: &Incoming,
    body: &[u8],
) -> Response<Cow<'static, [u8]>> {
    let registry = registry::current(app);
    let deadline = Instant::now() + api::COMMIT_WAIT;
    let response = loop {
        let showing = session::showing();
        let committed = session::committed();
        let opened = session::opened();
        let nonce = session::nonce();
        let token = session::token();
        let ctx = Context {
            platform: Platform::current(),
            showing: showing.as_deref(),
            committed: committed.as_deref(),
            registry: &registry,
            opened: opened.as_deref(),
            nonce: nonce.as_deref(),
            token: token.as_deref(),
        };
        match api::route(incoming, &ctx) {
            // The panel's own subresources can reach us a moment before its load event commits
            // it: hold them until it has. (A web page's request racing the app's navigation
            // waits too — and once the panel has committed, that page is gone.)
            Err(api::Refusal::Uncommitted) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            routed => break answer(app, incoming, body, routed, token),
        }
    };
    log_request(incoming, response.status().as_u16());
    response
}

/// The response to a routed (or refused) request.
fn answer<R: Runtime>(
    app: &AppHandle<R>,
    incoming: &Incoming,
    body: &[u8],
    routed: Result<Route, api::Refusal>,
    token: Option<String>,
) -> Response<Cow<'static, [u8]>> {
    match routed {
        Ok(Route::File(extension, rel)) => match (extension.read(&rel), paths::mime_type(&rel)) {
            (Some(bytes), Some(mime)) => response(200, mime, &extension.csp, bytes),
            _ => refuse(incoming, api::Refusal::NotFound),
        },
        // Routed only once the page shows this extension's committed opening: its own token.
        Ok(Route::Sdk(extension)) => match token {
            Some(token) => response(
                200,
                "text/javascript; charset=utf-8",
                &extension.csp,
                Cow::Owned(SDK.replace(TOKEN_PLACEHOLDER, &token).into_bytes()),
            ),
            None => refuse(incoming, api::Refusal::Page),
        },
        Ok(Route::Api(extension, method)) => {
            let (status, answer) = match limits::Call::start(extension.id()) {
                Some(_running) => answer_call(app, &extension, method, incoming, body),
                None => error(429, "busy", "Too many calls at once."),
            };
            let outcome = answer.get("error").and_then(Value::as_str).unwrap_or("ok");
            smoke::log(
                "ext-api",
                &format!("{} {} {outcome}", extension.id(), method.name()),
            );
            json(status, &extension.csp, &answer)
        }
        Err(refusal) => refuse(incoming, refusal),
    }
}

/// The smoke log only: an `ljext` request's answer (`status=`) and what the engine sent with it —
/// header names, the values of the ones the checks read (`Sec-Fetch-*`, `Origin`,
/// `Service-Worker`), and whose page the Referer is (`own` / `other`, never its value, which can
/// carry a nonce); never the token.
fn log_request(incoming: &Incoming, status: u16) {
    if !smoke::enabled() {
        return;
    }
    let referer = match (
        incoming.header("referer"),
        urls::parse_request(&incoming.uri),
    ) {
        (None, _) if incoming.headers.iter().any(|(n, _)| n == "referer") => "twice",
        (None, _) => "-",
        (Some(_), Some(target)) if api::referer_ok(incoming, &target.id, Platform::current()) => {
            "own"
        }
        (Some(_), _) => "other",
    };
    let mut names: Vec<&str> = incoming.headers.iter().map(|(n, _)| n.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    let value = |name: &str| incoming.header(name).unwrap_or("-").to_owned();
    let path = incoming.uri.split(['?', '#']).next().unwrap_or_default();
    smoke::log(
        "ljext-headers",
        &format!(
            "{} {} {path} status={status} referer={referer} names={} dest={} site={} mode={} origin={} service-worker={}",
            incoming.webview,
            incoming.method,
            names.join(","),
            value("sec-fetch-dest"),
            value("sec-fetch-site"),
            value("sec-fetch-mode"),
            value("origin"),
            value("service-worker"),
        ),
    );
}

/// An allowed API call: an over-long body is turned away before it is parsed (wry has read it
/// into memory by now — every engine's handler gets the whole body), then the method runs.
fn answer_call<R: Runtime>(
    app: &AppHandle<R>,
    extension: &Extension,
    method: Method,
    incoming: &Incoming,
    body: &[u8],
) -> (u16, Value) {
    let declared = incoming
        .header("content-length")
        .and_then(|length| length.trim().parse::<usize>().ok());
    if declared.is_some_and(|length| length > api::MAX_BODY_BYTES) {
        return error(413, "too-large", "The request is too large.");
    }
    call(app, extension, method, body)
}

fn refuse(incoming: &Incoming, refusal: api::Refusal) -> Response<Cow<'static, [u8]>> {
    // For an Origin refusal, what the engine sent (a web page's origin, `null`, or nothing).
    let origin = if refusal == api::Refusal::Origin {
        format!(
            " origin={} referer={}",
            incoming.header("origin").unwrap_or("-"),
            incoming.header("referer").unwrap_or("-")
        )
    } else {
        String::new()
    };
    smoke::log(
        "ljext-refused",
        &format!(
            "{} {} {}{origin}",
            refusal.code(),
            incoming.webview,
            incoming.uri
        ),
    );
    json(
        refusal.status(),
        csp::NOTHING,
        &json!({ "ok": false, "error": refusal.code(), "message": "Refused." }),
    )
}

fn response(
    status: u16,
    content_type: &str,
    policy: &str,
    body: Cow<'static, [u8]>,
) -> Response<Cow<'static, [u8]>> {
    let mut builder = Response::builder()
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header(header::CONTENT_TYPE, content_type);
    for (name, value) in csp::headers(policy) {
        builder = builder.header(name, value);
    }
    builder.body(body).unwrap_or_else(|err| {
        eprintln!("little-joan: could not build an ljext response: {err}");
        let mut failed = Response::new(Cow::Borrowed(&b""[..]));
        *failed.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        failed
    })
}

fn json(status: u16, policy: &str, value: &Value) -> Response<Cow<'static, [u8]>> {
    let body = serde_json::to_vec(value).unwrap_or_else(|_| b"{\"ok\":false}".to_vec());
    response(status, "application/json", policy, Cow::Owned(body))
}

fn ok(value: Option<Value>) -> (u16, Value) {
    match value {
        Some(value) => (200, json!({ "ok": true, "value": value })),
        None => (200, json!({ "ok": true })),
    }
}

fn error(status: u16, code: &str, message: impl Into<String>) -> (u16, Value) {
    (
        status,
        json!({ "ok": false, "error": code, "message": message.into() }),
    )
}

fn text_param<'a>(params: &'a Map<String, Value>, name: &str) -> Result<&'a str, (u16, Value)> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| error(400, "bad-request", format!("`{name}` must be text.")))
}

fn storage_answer<T>(
    result: Result<T, storage::StorageError>,
    value: impl FnOnce(T) -> Option<Value>,
) -> (u16, Value) {
    match result {
        Ok(result) => ok(value(result)),
        Err(err) => {
            let status = if err == storage::StorageError::Quota {
                413
            } else if matches!(err, storage::StorageError::Io(_)) {
                500
            } else {
                400
            };
            error(status, err.code(), err.message())
        }
    }
}

/// Runs an allowed API call.
fn call<R: Runtime>(
    app: &AppHandle<R>,
    extension: &Extension,
    method: Method,
    body: &[u8],
) -> (u16, Value) {
    let params = match api::params(body) {
        Ok(params) => params,
        Err((status, code)) => return error(status, code, "The request must be a JSON object."),
    };
    let id = extension.id();
    let dir = state::with(app, |s| s.data_dir.clone());
    let answer = || -> Result<(u16, Value), (u16, Value)> {
        Ok(match method {
            Method::StorageGet => {
                let key = text_param(&params, "key")?;
                storage_answer(
                    storage::check_key(key).and_then(|()| {
                        storage::with_store(&dir, id, |items| Ok((items.get(key).cloned(), false)))
                    }),
                    |value| value,
                )
            }
            Method::StorageSet => {
                if !limits::storage_set(id) {
                    return Err(error(429, "busy", "Too many writes (20 a second at most)."));
                }
                let key = text_param(&params, "key")?;
                let value = params
                    .get("value")
                    .cloned()
                    .ok_or_else(|| error(400, "bad-request", "`value` is missing."))?;
                storage_answer(
                    storage::with_store(&dir, id, |items| {
                        storage::set(items, key, value)?;
                        Ok(((), true))
                    }),
                    |()| None,
                )
            }
            Method::StorageRemove => {
                let key = text_param(&params, "key")?;
                storage_answer(
                    storage::check_key(key).and_then(|()| {
                        storage::with_store(&dir, id, |items| {
                            let removed = items.remove(key).is_some();
                            Ok((removed, removed))
                        })
                    }),
                    |removed| Some(Value::Bool(removed)),
                )
            }
            Method::StorageKeys => storage_answer(
                storage::with_store(&dir, id, |items| {
                    Ok((items.keys().cloned().collect::<Vec<_>>(), false))
                }),
                |keys| Some(json!(keys)),
            ),
            // The bar's colors, or the default theme's until the bar has reported (never a wait).
            Method::Theme => ok(Some(json!(theme::current()))),
            Method::PageInfo => {
                // Routing checked the opening; this checks it again where the page is handed out.
                if session::opened().as_deref() != Some(id) {
                    return Err(error(403, "page", "Not this extension's page."));
                }
                ok(Some(json!(session::page_info())))
            }
            Method::OpenUrl => {
                let url = api::openable(text_param(&params, "url")?).ok_or_else(|| {
                    error(
                        400,
                        "bad-url",
                        "Only public http and https addresses open (not this computer or the local network).",
                    )
                })?;
                match params
                    .get("where")
                    .and_then(Value::as_str)
                    .unwrap_or("page")
                {
                    // Not the address until the new page's own events say so.
                    "page" => match bar::navigate_pending(app, url) {
                        Ok(()) => ok(None),
                        Err(err) => error(500, "failed", err.to_string()),
                    },
                    "browser" => {
                        if !extension.manifest.has(&Permission::OpenBrowser) {
                            return Err(error(
                                403,
                                "permission",
                                "This needs the openBrowser permission.",
                            ));
                        }
                        if !limits::browser_open(id) {
                            return Err(error(429, "busy", "One address every 2 seconds at most."));
                        }
                        match tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
                            Ok(()) => ok(None),
                            Err(err) => error(500, "failed", err.to_string()),
                        }
                    }
                    _ => error(400, "bad-request", "`where` is \"page\" or \"browser\"."),
                }
            }
            Method::ClipboardWriteText => {
                let text = text_param(&params, "text")?.to_owned();
                match clipboard::write_text(app, text) {
                    Ok(()) => ok(None),
                    Err(err) => error(500, "failed", err),
                }
            }
            Method::Close => match session::close(app) {
                Ok(()) => ok(None),
                Err(err) => error(500, "failed", err),
            },
        })
    };
    answer().unwrap_or_else(|refused| refused)
}
