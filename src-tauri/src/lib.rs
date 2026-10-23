//! Little Joan: a tiny pop-up browser that hibernates until a global shortcut calls it.
//!
//! At rest the process has a tray icon and nothing else — no window and no webview. See
//! docs/architecture.md for the lifecycle.

pub mod autostart;
pub mod back_stack;
pub mod bar;
pub mod cli;
pub mod commands;
pub mod cookies;
pub mod downloads;
pub mod extensions;
pub mod fullscreen;
pub mod history;
pub mod history_store;
pub mod input;
pub mod joan;
pub mod keymap;
pub mod keys;
pub mod layout;
pub mod navigation;
pub mod page_actions;
pub mod page_address;
pub mod phone;
pub mod phone_mode;
pub mod placement;
pub mod settings;
pub mod settings_live;
pub mod settings_window;
pub mod shortcut;
pub mod smoke;
pub mod spaces;
pub mod state;
pub mod suggest;
pub mod summon_timing;
pub mod system_shortcuts;
pub mod toggle;
pub mod translate;
pub mod welcome;
pub mod zoom;

#[cfg(test)]
mod capability_tests;
#[cfg(target_os = "macos")]
mod menu;
mod platform;
mod tray;

use tauri::RunEvent;

use crate::summon_timing::Source;
use crate::toggle::Command;

/// Builds and runs the app. Returns only when the user quits.
pub fn run() {
    let app = tauri::Builder::default()
        // First: a second launch hands its arguments to the running instance and exits.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let args = cli::parse_args(&argv);
            if let Some(enabled) = args.start_at_login {
                // `set` logs the outcome; there is no window to show an error in.
                let _ = autostart::set(app, enabled);
            }
            if let Some(tab) = args.settings.filter(|_| !args.quit) {
                settings_window::open(app, Some(tab));
            }
            if args.quit {
                joan::quit(app);
            } else if let Some(command) = args.forwarded_command() {
                joan::dispatch(app, command, Source::Cli);
            }
        }))
        .plugin(shortcut::plugin())
        .plugin(autostart::plugin())
        .invoke_handler(tauri::generate_handler![
            commands::open_input,
            commands::hibernate,
            commands::start_info,
            commands::page_url,
            commands::focus_page,
            commands::set_overlay,
            commands::go_back,
            commands::open_in_browser,
            commands::get_settings,
            commands::save_settings,
            commands::suspend_shortcut,
            commands::set_start_at_login,
            commands::history_list,
            commands::suggest,
            commands::history_remove,
            commands::history_clear,
            commands::open_settings,
            commands::preview_theme,
            commands::set_theme_tokens,
            commands::extensions_list,
            commands::reload_extensions,
            commands::open_extensions_folder,
            commands::open_extension,
            commands::site::site_mode_info,
            commands::site::set_site_mode,
            commands::app::quit_app,
            commands::downloads::reveal_download,
            commands::quick::quick_state,
            commands::quick::quick_command,
            commands::keys::keymap_info,
            commands::timing::report_timing
        ])
        // Extension panels and their API (extensions/protocol.rs). Registered for every
        // webview; each request is checked (extensions/api.rs), and only the page webview's
        // are ever answered.
        .register_asynchronous_uri_scheme_protocol(
            extensions::urls::SCHEME,
            extensions::protocol::handle,
        )
        .on_window_event(|window, event| {
            joan::on_window_event(window, event);
            settings_window::on_window_event(window, event);
        })
        .setup(|app| {
            smoke::log("started", "");
            // Setup runs on the main thread (Windows: nothing may wait there for WebView2).
            platform::note_main_thread();

            // A menu-bar utility: no Dock icon on macOS.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();
            state::init(handle)?;
            #[cfg(target_os = "macos")]
            {
                // The page items' keys come from the keymap (state::init built it).
                let keymap = state::with(handle, |s| s.keymap.clone());
                app.set_menu(menu::create(handle, &keymap)?)?;
                app.on_menu_event(menu::on_menu_event);
            }
            tray::create(handle)?;
            // In a Linux Wayland session the XDG GlobalShortcuts portal is asked first; it answers
            // later (LJ17), and the icon's rule runs again then.
            shortcut::register_at_launch(handle);
            // Hidden only while the shortcut works (tray.rs, LJ41).
            tray::apply_visibility(handle);

            // A first launch with flags (e.g. `--show <url>`, `--start-at-login on`, `--settings`) acts on
            // them; without flags Little Joan starts hibernating — except on the very first run
            // (no settings.json yet), which opens the window once with the welcome.
            let args = cli::parse_args(&std::env::args().collect::<Vec<_>>());
            let first_run = state::with(handle, |s| {
                s.first_run
                    .then(|| (s.data_dir.clone(), s.settings.clone()))
            });
            let welcome = first_run.is_some_and(|(dir, settings)| {
                smoke::log("first-run", "");
                state::finish_first_run(&dir, &settings, &args)
            });
            state::with(handle, |s| s.first_run = welcome);
            if let Some(enabled) = args.start_at_login {
                let _ = autostart::set(handle, enabled);
            }
            if let Some(tab) = args.settings.filter(|_| !args.quit) {
                settings_window::open(handle, Some(tab));
            }
            if args.quit {
                // `little-joan --quit` with nothing running: nothing to quit, so this one ends.
                joan::quit(handle);
            } else if let Some(command) = args.command {
                joan::dispatch(handle, command, Source::Launch);
            } else if welcome && shortcut::welcome_now(handle) {
                // While the portal is asked, the welcome waits for its answer (shortcut.rs), so it
                // names the desktop's key — or explains `--toggle` when there is none.
                joan::dispatch(handle, Command::Show(None), Source::Launch);
            }
            Ok(())
        })
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(err) => {
            eprintln!("little-joan: failed to start: {err}");
            std::process::exit(1);
        }
    };

    app.run(|_app, event| {
        // Closing the last window must never quit: Little Joan goes back to hibernation.
        // Only an explicit exit (Quit in the tray menu) carries an exit code.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
