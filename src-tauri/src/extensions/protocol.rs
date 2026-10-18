//! The `ljext` scheme handler (glue). Tauri registers it for every webview; api.rs decides each
//! request, and this serves what it allows: an extension's files and the SDK with its CSP and
//! headers (csp.rs), and the API's methods. Refusals carry no extension data and a policy that
//! allows nothing. Each request runs on a blocking worker, off the engine's thread.

use std::borrow::Cow;

use serde_json::{Map, Value, json};
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Runtime, UriSchemeContext, UriSchemeResponder};

use super::api::{self, Context, Incoming, Method, Route};
use super::registry::{self, Extension};
use super::{clipboard, csp, paths, session, storage, theme};
use crate::bar;
use crate::keys::Platform;
use crate::smoke;
use crate::state;

/// The SDK every panel loads from `/__lj/sdk/v1.js`.
pub const SDK: &str = include_str!("../../../extensions/sdk/v1.js");

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
    let page = bar::page_url(app);
    let opened = session::opened();
    let ctx = Context {
        platform: Platform::current(),
        page: page.as_ref(),
        registry: &registry,
        opened: opened.as_deref(),
    };
    match api::route(incoming, &ctx) {
        Ok(Route::File(extension, rel)) => match (extension.read(&rel), paths::mime_type(&rel)) {
            (Some(bytes), Some(mime)) => response(200, mime, &extension.csp, bytes),
            _ => refuse(incoming, api::Refusal::NotFound),
        },
        Ok(Route::Sdk(extension)) => response(
            200,
            "text/javascript; charset=utf-8",
            &extension.csp,
            Cow::Borrowed(SDK.as_bytes()),
        ),
        Ok(Route::Api(extension, method)) => {
            let (status, answer) = call(app, &extension, method, body);
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
            Method::Theme => match theme::current(theme::WAIT) {
                Some(tokens) => ok(Some(json!(tokens))),
                None => error(503, "unavailable", "The theme isn't known yet."),
            },
            Method::PageInfo => ok(Some(json!(session::page_info()))),
            Method::OpenUrl => {
                let url = api::openable(text_param(&params, "url")?)
                    .ok_or_else(|| error(400, "bad-url", "Only http and https addresses open."))?;
                match params
                    .get("where")
                    .and_then(Value::as_str)
                    .unwrap_or("page")
                {
                    "page" => match bar::navigate(app, url) {
                        Ok(()) => ok(None),
                        Err(err) => error(500, "failed", err.to_string()),
                    },
                    "browser" => match tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
                        Ok(()) => ok(None),
                        Err(err) => error(500, "failed", err.to_string()),
                    },
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
