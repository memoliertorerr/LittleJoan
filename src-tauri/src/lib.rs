//! Little Joan: a tiny pop-up browser that hibernates until a global shortcut calls it.
//!
//! At rest the process has a tray icon and nothing else — no window and no webview. See
//! docs/architecture.md for the lifecycle.

pub mod autostart;
pub mod bar;
pub mod cli;
pub mod commands;
pub mod fullscreen;
pub mod input;
pub mod joan;
pub mod keys;
pub mod layout;
pub mod navigation;
pub mod page_actions;
pub mod phone;
pub mod phone_mode;
pub mod placement;
pub mod settings;
pub mod settings_window;
pub mod shortcut;
pub mod smoke;
pub mod state;
pub mod toggle;
pub mod translate;

#[cfg(target_os = "macos")]
mod menu;
mod platform;
mod tray;

use tauri::RunEvent;

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
            if let Some(command) = args.forwarded_command() {
                joan::dispatch(app, command);
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
            commands::get_settings,
            commands::save_settings,
            commands::suspend_shortcut,
            commands::set_start_at_login
        ])
        .on_window_event(|window, event| {
            joan::on_window_event(window, event);
            settings_window::on_window_event(window, event);
        })
        .setup(|app| {
            smoke::log("started", "");

            // A menu-bar utility: no Dock icon on macOS.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();
            #[cfg(target_os = "macos")]
            {
                app.set_menu(menu::create(handle)?)?;
                app.on_menu_event(menu::on_menu_event);
            }
            state::init(handle)?;
            tray::create(handle)?;
            shortcut::register_from_settings(handle);

            // A first launch with flags (e.g. `--show <url>`, `--start-at-login on`) acts on
            // them; without flags Little Joan starts hibernating.
            let args = cli::parse_args(&std::env::args().collect::<Vec<_>>());
            if let Some(enabled) = args.start_at_login {
                let _ = autostart::set(handle, enabled);
            }
            if let Some(command) = args.command {
                joan::dispatch(handle, command);
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
