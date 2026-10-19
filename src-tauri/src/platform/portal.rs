//! Linux: the summon shortcut through the XDG GlobalShortcuts portal (LJ17), the Wayland way —
//! global-hotkey's X11 grab hears keys only while an X11 window has the focus. Asked once at
//! launch in a Wayland session (shortcut::register_at_launch): Little Joan registers itself with
//! the portal as its desktop file's app (the host registry; the `.deb` installs
//! `io.github.memoliertorerr.littlejoan.desktop` for it — Landmines), creates a session, binds one
//! shortcut and keeps the session for the life of the process; Activated toggles the window. The
//! desktop owns the key: it may ask the user first and may pick another trigger. Whatever goes
//! wrong, global-hotkey takes the shortcut as before (shortcut::portal_answered).

use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::task::Poll;
use std::time::Duration;

use ashpd::desktop::global_shortcuts::{
    Activated, BindShortcutsOptions, GlobalShortcuts, NewShortcut, ShortcutsChanged,
};
use ashpd::desktop::{CreateSessionOptions, ResponseError, Session};
use ashpd::zbus;
use ashpd::zbus::export::futures_core::Stream;
use ashpd::{AppID, PortalError};
use tauri::{AppHandle, Runtime};

use crate::joan;
use crate::shortcut::{
    self,
    backend::{self, PortalState, Unavailable},
};
use crate::smoke;
use crate::state;
use crate::toggle::Command;

/// How long the portal may take to set up a session (D-Bus activation included). Binding has no
/// limit: the desktop may ask the user first.
const SETUP_LIMIT: Duration = Duration::from_secs(15);

/// D-Bus errors that mean there is no portal to ask, or none with GlobalShortcuts.
const NO_PORTAL: [&str; 4] = [
    "org.freedesktop.DBus.Error.ServiceUnknown",
    "org.freedesktop.DBus.Error.UnknownInterface",
    "org.freedesktop.DBus.Error.UnknownMethod",
    "org.freedesktop.DBus.Error.UnknownObject",
];

/// Asks the portal for the shortcut, off the main thread; the answer comes through
/// shortcut::portal_answered.
pub fn start<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let (why, detail) = match tauri::async_runtime::spawn(serve(app.clone())).await {
            Ok(end) => end,
            // A panic in there (ashpd asserts on some of the portal's answers) ends it like any
            // other failure.
            Err(err) => (Unavailable::Failed, err.to_string()),
        };
        eprintln!(
            "little-joan: the GlobalShortcuts portal doesn't hold the shortcut ({}): {detail}",
            why.word()
        );
        let (event, line) = backend::unavailable_line(why, &detail);
        smoke::log(event, &line);
        shortcut::portal_answered(&app, PortalState::None);
    });
}

/// Binds the shortcut and serves it while the portal's session lives; returns why it ended (at
/// once when the portal can't take it).
async fn serve<R: Runtime>(app: AppHandle<R>) -> (Unavailable, String) {
    let preferred = state::with(&app, |s| backend::trigger(&s.settings.shortcut));
    let app_id = app.config().identifier.clone();
    let (portal, session) = match within(SETUP_LIMIT, open(&app_id)).await {
        Some(Ok(opened)) => opened,
        Some(Err(failure)) => return failure,
        None => return (Unavailable::Timeout, format!("{}s", SETUP_LIMIT.as_secs())),
    };
    // Listening before binding, so nothing the desktop sends after its answer is missed.
    let activated = match portal.receive_activated().await {
        Ok(stream) => stream,
        Err(err) => return failure(&err),
    };
    let changed = match portal.receive_shortcuts_changed().await {
        Ok(stream) => stream,
        Err(err) => return failure(&err),
    };
    let wanted = NewShortcut::new(backend::SHORTCUT_ID, backend::DESCRIPTION)
        .preferred_trigger(preferred.as_deref());
    let request = match portal
        .bind_shortcuts(&session, &[wanted], None, BindShortcutsOptions::default())
        .await
    {
        Ok(request) => request,
        Err(err) => return failure(&err),
    };
    let bound = match request.response() {
        Ok(bound) => bound,
        Err(err) => return failure(&err),
    };
    let Some(ours) = bound
        .shortcuts()
        .iter()
        .find(|s| s.id() == backend::SHORTCUT_ID)
    else {
        return (Unavailable::NoShortcut, String::new());
    };
    let trigger = backend::shown_trigger(ours.trigger_description());
    let (event, detail) = backend::registered_line(trigger.as_deref());
    smoke::log(event, &detail);
    shortcut::portal_answered(&app, PortalState::Registered { trigger });
    listen(&app, activated, changed).await;
    // The portal and its session lived until here: dropping them ends the shortcut.
    drop((portal, session));
    (Unavailable::Ended, String::new())
}

/// A session bus connection registered as Little Joan's app (when the portal's host registry
/// can), the GlobalShortcuts portal on it, and a session.
async fn open(
    app_id: &str,
) -> Result<(GlobalShortcuts, Session<GlobalShortcuts>), (Unavailable, String)> {
    let session_bus = || async {
        zbus::Connection::session()
            .await
            .map_err(|err| (Unavailable::NoBus, err.to_string()))
    };
    let connection = session_bus().await?;
    // Before any other portal call on this connection (the registry's rule). GNOME's portal wants
    // an app id for global shortcuts, and finds none for an app started from a terminal.
    let registered = match app_id.parse::<AppID>() {
        Ok(id) => ashpd::register_host_app_with_connection(connection.clone(), id)
            .await
            .map_err(|err| err.to_string()),
        Err(err) => Err(err.to_string()),
    };
    let connection = match registered {
        Ok(()) => {
            smoke::log(backend::HOST, "registered");
            connection
        }
        Err(err) => {
            eprintln!("little-joan: the portal's host registry didn't take {app_id}: {err}");
            smoke::log(backend::HOST, &format!("unregistered {err}"));
            // A fresh connection, which the portal identifies by itself (from the app's systemd
            // unit, when the desktop started it).
            session_bus().await?
        }
    };
    let portal = GlobalShortcuts::with_connection(connection)
        .await
        .map_err(|err| failure(&err))?;
    let session = portal
        .create_session(CreateSessionOptions::default())
        .await
        .map_err(|err| failure(&err))?;
    Ok((portal, session))
}

/// Toggles the window on each activation of Little Joan's shortcut, and follows the desktop's
/// changes of its trigger, until either signal stream ends (the portal or the bus went away).
async fn listen<R: Runtime>(
    app: &AppHandle<R>,
    activated: impl Stream<Item = Activated>,
    changed: impl Stream<Item = ShortcutsChanged>,
) {
    enum Event {
        Activated(Activated),
        Changed(ShortcutsChanged),
    }
    let mut activated = pin!(activated);
    let mut changed = pin!(changed);
    loop {
        let event = poll_fn(|cx| {
            if let Poll::Ready(item) = activated.as_mut().poll_next(cx) {
                return Poll::Ready(item.map(Event::Activated));
            }
            if let Poll::Ready(item) = changed.as_mut().poll_next(cx) {
                return Poll::Ready(item.map(Event::Changed));
            }
            Poll::Pending
        })
        .await;
        match event {
            None => return,
            Some(Event::Activated(signal)) => {
                if signal.shortcut_id() == backend::SHORTCUT_ID {
                    joan::dispatch(app, Command::Toggle);
                }
            }
            Some(Event::Changed(signal)) => {
                if let Some(ours) = signal
                    .shortcuts()
                    .iter()
                    .find(|s| s.id() == backend::SHORTCUT_ID)
                {
                    shortcut::portal_changed(
                        app,
                        backend::shown_trigger(ours.trigger_description()),
                    );
                }
            }
        }
    }
}

/// `work`, or `None` when it took longer than `limit`.
async fn within<T>(limit: Duration, work: impl Future<Output = T>) -> Option<T> {
    let mut work = pin!(work);
    // No timer of our own: a sleeping thread of the blocking pool is one.
    let mut timer = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(limit));
    poll_fn(|cx| {
        if let Poll::Ready(value) = work.as_mut().poll(cx) {
            return Poll::Ready(Some(value));
        }
        Pin::new(&mut timer).poll(cx).map(|_| None)
    })
    .await
}

/// Why `err` means the portal can't take the shortcut, with its text.
fn failure(err: &ashpd::Error) -> (Unavailable, String) {
    let why = match err {
        ashpd::Error::PortalNotFound(_) => Unavailable::NoPortal,
        ashpd::Error::Response(ResponseError::Cancelled)
        | ashpd::Error::Portal(PortalError::Cancelled(_)) => Unavailable::Cancelled,
        ashpd::Error::Response(_) => Unavailable::Refused,
        ashpd::Error::Portal(PortalError::ZBus(err)) | ashpd::Error::Zbus(err) => bus_failure(err),
        ashpd::Error::Portal(_) => Unavailable::Refused,
        _ => Unavailable::Failed,
    };
    (why, err.to_string())
}

/// A D-Bus error: no portal (no such service, interface or method), the portal's refusal, or
/// something else.
fn bus_failure(err: &zbus::Error) -> Unavailable {
    match err {
        zbus::Error::MethodError(name, _, _) if NO_PORTAL.contains(&name.as_str()) => {
            Unavailable::NoPortal
        }
        zbus::Error::FDO(fdo)
            if matches!(
                **fdo,
                zbus::fdo::Error::ServiceUnknown(_)
                    | zbus::fdo::Error::UnknownInterface(_)
                    | zbus::fdo::Error::UnknownMethod(_)
                    | zbus::fdo::Error::UnknownObject(_)
            ) =>
        {
            Unavailable::NoPortal
        }
        zbus::Error::MethodError(..) | zbus::Error::FDO(_) => Unavailable::Refused,
        _ => Unavailable::Failed,
    }
}
