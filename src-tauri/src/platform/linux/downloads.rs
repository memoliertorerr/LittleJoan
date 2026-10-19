//! The page's downloads on WebKitGTK (LJ15), through WebKitGTK itself rather than Tauri's
//! `on_download`. wry registers its download handler on the **web context** — shared by both
//! webviews of every window (and Settings in installed builds), and kept by Tauri for the app's
//! lifetime on Linux — once per webview built, each connecting a `decide-destination` handler on
//! every download, and WebKitGTK's `decide-destination` ends at the first handler that returns
//! TRUE (wry's always do). So the first webview's built-in "save anywhere" would decide the page's
//! downloads, and each window would add one more `finished` report with a failure flag that never
//! resets.
//!
//! So Little Joan decides in an **emission hook** on `WebKitDownload::decide-destination`: it runs
//! for every download in the process, before any handler, whatever the context and whenever the
//! signal comes (CI's Ubuntu 22.04 — WebKitGTK 2.50.4 — never ran a handler connected in the
//! context's `download-started`; 2.52 did). For a download from the open window's page webview it
//! disconnects that download's other `decide-destination` handlers (wry's: a handler disconnected
//! during an emission is skipped), sets the destination itself (bar/download.rs decides; a
//! `file://` URI, which WebKitGTK 4.1's legacy API wants on 2.50), or cancels a refused one; the
//! class handler then keeps that destination. Downloads under way are cancelled when the window
//! closes (`cancel`).

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_uint};
use std::rc::Rc;
use std::sync::Once;

use gtk::glib::gobject_ffi;
use gtk::glib::prelude::*;
use gtk::glib::translate::{IntoGlib, from_glib_none};
use tauri::{AppHandle, Runtime};
use url::Url;
use webkit2gtk::{DownloadExt, URIRequestExt};

use crate::bar;
use crate::bar::download::Outcome;
use crate::smoke;

/// What decides a page download, given the download and the engine's suggested name.
type Decide = Box<dyn Fn(&webkit2gtk::Download, Option<&str>)>;

thread_local! {
    /// The open window's page webview (GTK main thread only): whose downloads are Little Joan's.
    static PAGE_VIEW: RefCell<Option<gtk::glib::WeakRef<webkit2gtk::WebView>>> =
        const { RefCell::new(None) };
    /// The page's downloads under way, cancelled when its window closes.
    static UNDER_WAY: RefCell<Vec<webkit2gtk::Download>> = const { RefCell::new(Vec::new()) };
    /// What the emission hook calls (it needs the app handle, which an extern fn can't hold).
    static DECIDE: RefCell<Option<Decide>> = const { RefCell::new(None) };
}

/// The emission hook is added once per process.
static HOOK: Once = Once::new();

/// The page webview of a new window takes its downloads (GTK main thread: `install_page`).
pub fn take<R: Runtime>(app: &AppHandle<R>, view: &webkit2gtk::WebView) {
    PAGE_VIEW.with(|page| *page.borrow_mut() = Some(view.downgrade()));
    DECIDE.with(|decide| {
        if decide.borrow().is_none() {
            let app = app.clone();
            let decides: Decide = Box::new(
                move |download: &webkit2gtk::Download, suggested: Option<&str>| {
                    decide_destination(&app, download, suggested);
                },
            );
            *decide.borrow_mut() = Some(decides);
        }
    });
    HOOK.call_once(|| {
        // SAFETY: the signal is looked up on WebKitDownload's own (registered) type, and the hook
        // is a plain function that lives as long as the process; no data is passed to it.
        let hooked = unsafe {
            let signal = gobject_ffi::g_signal_lookup(
                c"decide-destination".as_ptr(),
                webkit2gtk::Download::static_type().into_glib(),
            );
            signal != 0
                && gobject_ffi::g_signal_add_emission_hook(
                    signal,
                    0,
                    Some(on_decide_destination),
                    std::ptr::null_mut(),
                    None,
                ) != 0
        };
        if !hooked {
            eprintln!("little-joan: no hook on WebKitGTK's downloads; they are WebKitGTK's own");
        }
    });
}

/// `decide-destination`'s emission hook: (the download, its suggested file name).
unsafe extern "C" fn on_decide_destination(
    _hint: *mut gobject_ffi::GSignalInvocationHint,
    n_values: c_uint,
    values: *const gobject_ffi::GValue,
    _data: gtk::glib::ffi::gpointer,
) -> gtk::glib::ffi::gboolean {
    if n_values >= 1 && !values.is_null() {
        // SAFETY: GLib passes the instance first, then the signal's parameters — here one string
        // (the suggested file name) — as `n_values` initialised GValues.
        let (instance, suggested) = unsafe {
            let instance = gobject_ffi::g_value_get_object(values);
            let suggested = if n_values >= 2 {
                let name = gobject_ffi::g_value_get_string(values.add(1));
                (!name.is_null()).then(|| CStr::from_ptr(name).to_string_lossy().into_owned())
            } else {
                None
            };
            (instance, suggested)
        };
        if !instance.is_null() {
            // SAFETY: the instance of a WebKitDownload signal is a WebKitDownload; a new
            // reference is taken for the call.
            let download: webkit2gtk::Download =
                unsafe { from_glib_none(instance.cast::<webkit2gtk::ffi::WebKitDownload>()) };
            DECIDE.with(|decide| {
                if let Some(decide) = decide.borrow().as_ref() {
                    decide(&download, suggested.as_deref());
                }
            });
        }
    }
    // Stay hooked.
    gtk::glib::ffi::GTRUE
}

/// Whether `download` comes from the open window's page webview.
fn from_page(download: &webkit2gtk::Download) -> bool {
    let Some(from) = download.web_view() else {
        return false;
    };
    PAGE_VIEW.with(|page| {
        page.borrow()
            .as_ref()
            .and_then(gtk::glib::WeakRef::upgrade)
            .is_some_and(|page| page == from)
    })
}

fn decide_destination<R: Runtime>(
    app: &AppHandle<R>,
    download: &webkit2gtk::Download,
    suggested: Option<&str>,
) {
    if !from_page(download) {
        smoke::log("download-decide", "other");
        return;
    }
    let Some(url) = download
        .request()
        .and_then(|request| request.uri())
        .and_then(|uri| Url::parse(&uri).ok())
    else {
        smoke::log("download-decide", "no-url");
        download.cancel();
        return;
    };
    take_destination(download);
    // Whether it is being saved (not refused), and whether it failed: per download.
    let saving = Rc::new(Cell::new(false));
    let failed = Rc::new(Cell::new(false));
    {
        let failed = failed.clone();
        download.connect_failed(move |_, _| failed.set(true));
    }
    {
        let (app, url, saving) = (app.clone(), url.clone(), saving.clone());
        download.connect_finished(move |download| {
            UNDER_WAY.with(|all| all.borrow_mut().retain(|open| open != download));
            if saving.get() {
                let outcome = if failed.get() {
                    Outcome::Failed
                } else {
                    Outcome::Saved
                };
                bar::download::finished(&app, &url, outcome);
            }
        });
    }
    let suggested = suggested.filter(|name| !name.is_empty());
    match bar::download::requested(app, &url, suggested) {
        Some(path) => match gtk::glib::filename_to_uri(&path, None) {
            Ok(uri) => {
                download.set_destination(&uri);
                saving.set(true);
                UNDER_WAY.with(|all| all.borrow_mut().push(download.clone()));
            }
            Err(err) => {
                eprintln!("little-joan: no URI for {}: {err}", path.display());
                bar::download::finished(app, &url, Outcome::Failed);
                download.cancel();
            }
        },
        None => download.cancel(),
    }
}

/// Disconnects `download`'s `decide-destination` handlers — wry's, one per webview built on its
/// context — so they don't decide after the hook (a handler disconnected during an emission isn't
/// run).
fn take_destination(download: &webkit2gtk::Download) {
    // SAFETY: `download` is a live GObject for the duration of the call; the signal is looked up
    // on its own type, and matching only by signal id disconnects that signal's handlers on this
    // one instance.
    let removed = unsafe {
        let signal = gobject_ffi::g_signal_lookup(
            c"decide-destination".as_ptr(),
            webkit2gtk::Download::static_type().into_glib(),
        );
        if signal == 0 {
            0
        } else {
            gobject_ffi::g_signal_handlers_disconnect_matched(
                download.as_ptr().cast(),
                gobject_ffi::G_SIGNAL_MATCH_ID,
                signal,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }
    };
    smoke::log("download-deciders-removed", &removed.to_string());
}

/// The window is closing: its page's downloads under way are cancelled (on the main thread).
pub fn cancel<R: Runtime>(app: &AppHandle<R>) {
    let result = app.run_on_main_thread(|| {
        let open = UNDER_WAY.with(|all| std::mem::take(&mut *all.borrow_mut()));
        for download in open {
            download.cancel();
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not cancel the downloads: {err}");
    }
}
