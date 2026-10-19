//! Linux. Tauri packs a window's webviews into a vertical GtkBox (so bounds don't apply; GTK
//! lays them out), which this module uses for the bar: in Toolbar mode it is packed at a fixed
//! height and the page takes the rest; in Overlay mode it fills the box and the page is hidden.
//! The bar sits in a frame whose natural height doesn't follow the bar's web content (see
//! `install_bar`).
//! Page keys go through the GTK window's `key-press-event`, which runs before WebKitGTK receives
//! the key (handlers connected with `connect_*` run before the class handler). Element
//! fullscreen is followed through WebKitGTK's `enter-fullscreen` and `leave-fullscreen`
//! signals, to hide the bar meanwhile — and switched off on WebKitGTK older than 2.52, which
//! aborts the process as a page enters it (LJ14.1, `fullscreen::needs_fullscreen_off`).
//! The page's downloads are handled here, through WebKitGTK itself (`take_downloads`, LJ15).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib::Propagation;
use gtk::glib::prelude::*;
use gtk::prelude::*;
use tauri::{AppHandle, Manager, Runtime, Webview, Window};
use url::Url;
use webkit2gtk::{DownloadExt, SettingsExt, URIRequestExt, WebContextExt, WebViewExt};

use crate::bar;
use crate::bar::download::Outcome;
use crate::fullscreen;
use crate::keys::{Key, Mods, Platform};
use crate::layout::BAR_HEIGHT;
use crate::page_actions;
use crate::phone;
use crate::smoke;
use crate::state;

/// The bar: a fixed height at the top of the window's box instead of an equal share.
///
/// A WebKitGTK view's natural height is its page's contents size, which is never smaller than
/// the view itself. Once the command bar's overlay had made the bar as tall as the window, the
/// box kept giving it that natural height after the overlay closed (natural sizes are honoured
/// before `expand`), and the page got 1 px (CI, LJ22). So the bar is moved into a frame — a
/// scrolled window with external scrolling, which doesn't pass its child's natural size on, and a
/// viewport that sizes the child to the frame — and the frame is what the box packs: 40 px in
/// Toolbar mode, the whole box in Overlay mode.
pub fn install_bar<R: Runtime>(bar: &Webview<R>) {
    // with_webview runs on the GTK main thread, where GTK objects may be touched.
    let result = bar.with_webview(|webview| {
        let view = webview.inner();
        let Some(container) = view
            .parent()
            .and_then(|parent| parent.downcast::<gtk::Box>().ok())
        else {
            eprintln!("little-joan: the bar is not in a GtkBox; layout is GTK's");
            return;
        };
        let viewport = gtk::Viewport::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        viewport.set_shadow_type(gtk::ShadowType::None);
        viewport.set_hscroll_policy(gtk::ScrollablePolicy::Minimum);
        viewport.set_vscroll_policy(gtk::ScrollablePolicy::Minimum);
        let frame = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        frame.set_policy(gtk::PolicyType::External, gtk::PolicyType::External);
        frame.set_propagate_natural_height(false);
        frame.set_propagate_natural_width(false);
        frame.set_shadow_type(gtk::ShadowType::None);
        frame.set_kinetic_scrolling(false);
        frame.set_overlay_scrolling(false);
        // BAR_HEIGHT is a small constant (40), so the cast is exact.
        frame.set_size_request(-1, BAR_HEIGHT as i32);
        container.remove(&view);
        viewport.add(&view);
        frame.add(&viewport);
        container.pack_start(&frame, false, false, 0);
        container.reorder_child(&frame, 0);
        // Element fullscreen hides the bar (Webview::hide hides the view itself): the frame
        // follows, so the page gets the whole window.
        let shown = frame.clone();
        view.connect_show(move |_| shown.show());
        let hidden = frame.clone();
        view.connect_hide(move |_| hidden.hide());
        viewport.show();
        if view.is_visible() {
            frame.show();
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the bar: {err}");
    }
}

/// The bar's place in the window's box: the box, and the box's child that holds the bar (the
/// frame `install_bar` put it in).
fn bar_slot(view: &webkit2gtk::WebView) -> Option<(gtk::Box, gtk::Widget)> {
    let mut child: gtk::Widget = view.clone().upcast();
    loop {
        match child.parent()?.downcast::<gtk::Box>() {
            Ok(container) => return Some((container, child)),
            Err(parent) => child = parent,
        }
    }
}

/// The command bar's overlay. GTK lays out the box, so instead of bounds: the bar's frame is
/// packed to fill the window (its 40 px request stays as a minimum, so the window can still
/// shrink) and the page widget is hidden; closing it undoes both. tao shows a window with `show_all()`, which
/// would bring a hidden page back, so the page is kept out of it (`no_show_all`) meanwhile.
pub fn set_overlay<R: Runtime>(bar: &Webview<R>, page: &Webview<R>, on: bool) {
    let result = page.with_webview(move |webview| {
        let view = webview.inner();
        view.set_no_show_all(on);
        if on {
            view.hide();
        } else {
            view.show_all();
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the page: {err}");
    }
    let result = bar.with_webview(move |webview| match bar_slot(&webview.inner()) {
        Some((container, slot)) => {
            container.set_child_packing(&slot, on, on, 0, gtk::PackType::Start);
        }
        None => eprintln!("little-joan: the bar is not in a GtkBox; layout is GTK's"),
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the bar: {err}");
    }
}

/// WebKitGTK's version at run time: the library actually loaded (the system's for the `.deb`,
/// the bundled one in the AppImage), not the one the app was built against.
fn webkit_version() -> (u32, u32, u32) {
    // SAFETY: argument-less getters of numbers compiled into the loaded libwebkit2gtk.
    unsafe {
        (
            webkit2gtk::ffi::webkit_get_major_version(),
            webkit2gtk::ffi::webkit_get_minor_version(),
            webkit2gtk::ffi::webkit_get_micro_version(),
        )
    }
}

/// WebKitGTK older than 2.52 aborts as a page enters element fullscreen, whatever
/// `enter-fullscreen` answers (LJ14's experiment), so the page's fullscreen is switched off there:
/// its request is refused, and nothing crashes (smoke log `fullscreen-off <version>`).
fn fullscreen_off_where_it_aborts(view: &webkit2gtk::WebView) {
    let (major, minor, micro) = webkit_version();
    if !fullscreen::needs_fullscreen_off(major, minor) {
        return;
    }
    match WebViewExt::settings(view) {
        Some(settings) => {
            settings.set_enable_fullscreen(false);
            smoke::log("fullscreen-off", &format!("{major}.{minor}.{micro}"));
        }
        None => eprintln!(
            "little-joan: the webview has no settings; element fullscreen stays on (WebKitGTK {major}.{minor})"
        ),
    }
}

/// The page: page keys for the whole window, element fullscreen, and its downloads.
pub fn install_page<R: Runtime>(page: &Webview<R>) {
    let app = page.app_handle().clone();
    let result = page.with_webview(move |webview| {
        let view = webview.inner();
        fullscreen_off_where_it_aborts(&view);
        take_downloads(&app, &view);
        for (signal, on) in [("enter-fullscreen", true), ("leave-fullscreen", false)] {
            let app = app.clone();
            view.connect_local(signal, false, move |_| {
                // Hide or show the bar after WebKitGTK has handled the signal.
                let app = app.clone();
                tauri::async_runtime::spawn(async move { bar::set_element_fullscreen(&app, on) });
                // FALSE: let WebKitGTK go on with its own fullscreen handling.
                Some(false.to_value())
            });
        }
        let Some(toplevel) = view.toplevel() else {
            eprintln!("little-joan: the webview has no toplevel window; page keys are off");
            return;
        };
        toplevel.connect_key_press_event(move |_, event| {
            let state = event.state();
            let mods = Mods {
                ctrl: state.contains(gtk::gdk::ModifierType::CONTROL_MASK),
                shift: state.contains(gtk::gdk::ModifierType::SHIFT_MASK),
                alt: state.contains(gtk::gdk::ModifierType::MOD1_MASK),
                meta: state.contains(gtk::gdk::ModifierType::SUPER_MASK)
                    || state.contains(gtk::gdk::ModifierType::META_MASK),
            };
            let key = event
                .keyval()
                .name()
                .map_or(Key::Other, |name| Key::from_gdk_name(name.as_str()));
            match state::with(&app, |s| s.keymap.action_for(key, mods)) {
                Some(action) => {
                    page_actions::perform(&app, action.page_action());
                    Propagation::Stop
                }
                None => Propagation::Proceed,
            }
        });
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the webview: {err}");
    }
}

thread_local! {
    /// The open window's page webview (GTK main thread only): whose downloads are Little Joan's.
    static PAGE_VIEW: RefCell<Option<gtk::glib::WeakRef<webkit2gtk::WebView>>> =
        const { RefCell::new(None) };
    /// The page's downloads under way, cancelled when its window closes.
    static UNDER_WAY: RefCell<Vec<webkit2gtk::Download>> = const { RefCell::new(Vec::new()) };
    /// The web contexts whose downloads come here: Tauri keeps one per data folder for the app's
    /// lifetime on Linux, so each is hooked once.
    static HOOKED: RefCell<Vec<webkit2gtk::WebContext>> = const { RefCell::new(Vec::new()) };
}

/// The page's downloads, through WebKitGTK itself rather than Tauri's `on_download`: wry
/// registers its download handler on the web context — shared by both webviews of every window
/// (and Settings) — once per webview built, and the first handler registered decides every
/// download's destination (WebKitGTK's `decide-destination` ends at the first that returns TRUE,
/// and wry's always do). So the bar's built-in "save anywhere" would decide the page's downloads,
/// and every later window would add another `finished` report (with a failure flag that sticks).
/// Here: one hook per context, for downloads that come from the page webview only; it removes the
/// deciders wry connected before it and decides itself (bar/download.rs, downloads.rs).
fn take_downloads<R: Runtime>(app: &AppHandle<R>, view: &webkit2gtk::WebView) {
    PAGE_VIEW.with(|page| *page.borrow_mut() = Some(view.downgrade()));
    let Some(context) = WebViewExt::context(view) else {
        eprintln!("little-joan: the page has no web context; its downloads are WebKitGTK's");
        return;
    };
    let new = HOOKED.with(|hooked| {
        let mut hooked = hooked.borrow_mut();
        if hooked.contains(&context) {
            false
        } else {
            hooked.push(context.clone());
            true
        }
    });
    if new {
        let app = app.clone();
        context.connect_download_started(move |_, download| download_started(&app, download));
    }
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

fn download_started<R: Runtime>(app: &AppHandle<R>, download: &webkit2gtk::Download) {
    if !from_page(download) {
        return;
    }
    let Some(url) = download
        .request()
        .and_then(|request| request.uri())
        .and_then(|uri| Url::parse(&uri).ok())
    else {
        download.cancel();
        return;
    };
    take_destination(download);
    // Whether it is being saved (not refused), and whether it failed: per download.
    let saving = Rc::new(Cell::new(false));
    let failed = Rc::new(Cell::new(false));
    {
        let (app, url, saving) = (app.clone(), url.clone(), saving.clone());
        download.connect_decide_destination(move |download, suggested| {
            let suggested = Some(suggested).filter(|name| !name.is_empty());
            match bar::download::requested(&app, &url, suggested) {
                Some(path) => match gtk::glib::filename_to_uri(&path, None) {
                    Ok(uri) => {
                        download.set_destination(&uri);
                        saving.set(true);
                        UNDER_WAY.with(|all| all.borrow_mut().push(download.clone()));
                    }
                    Err(err) => {
                        eprintln!("little-joan: no URI for {}: {err}", path.display());
                        bar::download::finished(&app, &url, Outcome::Failed);
                        download.cancel();
                    }
                },
                None => download.cancel(),
            }
            // Decided: the signal ends here (later handlers, wry's too, don't run).
            true
        });
    }
    {
        let failed = failed.clone();
        download.connect_failed(move |_, _| failed.set(true));
    }
    let app = app.clone();
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

/// Removes the `decide-destination` handlers connected on `download` so far — wry's, one per
/// webview built on this context before the page's hook — so the page's own decides. (Handlers
/// connected after it run after it, and it ends the signal.)
fn take_destination(download: &webkit2gtk::Download) {
    use gtk::glib::gobject_ffi;
    use gtk::glib::translate::IntoGlib;
    // SAFETY: `download` is a live GObject for the duration of the call; the signal is looked up
    // on its own type, and matching only by signal id disconnects every handler of that signal on
    // this one instance (none of them is running: this is `download-started`).
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
    crate::smoke::log("download-deciders-removed", &removed.to_string());
}

/// The window is closing: its page's downloads under way are cancelled (on the main thread).
pub fn cancel_downloads<R: Runtime>(app: &AppHandle<R>) {
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

/// The GDK window state's fullscreen bit (set by WebKitGTK's own fullscreen, by tao, or by the
/// window manager). Main thread only.
pub fn is_fullscreen<R: Runtime>(window: &Window<R>) -> bool {
    window
        .gtk_window()
        .ok()
        .and_then(|gtk_window| gtk_window.window())
        .is_some_and(|gdk_window| {
            gdk_window
                .state()
                .contains(gtk::gdk::WindowState::FULLSCREEN)
        })
}

/// WebKitGTK: the user agent in the webview's WebKitSettings (unset = the engine's own). The
/// view's `settings` GObject property is write-only, so this goes through
/// `webkit_web_view_get_settings` (the webkit2gtk crate, as wry does).
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    let result = page.with_webview(move |webview| {
        let user_agent = mobile.then(|| phone::mobile_user_agent(Platform::Linux, None));
        match WebViewExt::settings(&webview.inner()) {
            Some(settings) => settings.set_user_agent(user_agent.as_deref()),
            None => eprintln!("little-joan: the webview has no settings; user agent unchanged"),
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not change the user agent: {err}");
    }
}

/// Puts `text` on the clipboard (and asks GTK to keep it after Little Joan quits). Main thread
/// only.
pub fn write_clipboard(text: &str) -> Result<(), String> {
    let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
    clipboard.set_text(text);
    clipboard.store();
    Ok(())
}
