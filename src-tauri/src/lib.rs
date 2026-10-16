//! Little Joan: a tiny pop-up browser that hibernates until a global shortcut calls it.
//!
//! At rest the process has a tray icon and nothing else — no window and no webview. See
//! docs/architecture.md for the lifecycle.

pub mod cli;
pub mod commands;
pub mod fullscreen;
pub mod input;
pub mod joan;
pub mod keys;
pub mod navigation;
pub mod page_actions;
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

use crate::toggle::Command;

/// Builds and runs the app. Returns only when the user quits.
pub fn run() {
    let app = tauri::Builder::default()
        // First: a second launch hands its arguments to the running instance and exits.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            joan::dispatch(app, cli::parse_args(&argv).unwrap_or(Command::Show(None)));
        }))
        .plugin(shortcut::plugin())
        .invoke_handler(tauri::generate_handler![
            commands::open_input,
            commands::hibernate,
            commands::start_info,
            commands::get_settings,
            commands::save_settings,
            commands::suspend_shortcut
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

            // A first launch with flags (e.g. `--show <url>`) acts on them; without flags
            // Little Joan starts hibernating.
            let args: Vec<String> = std::env::args().collect();
            if let Some(command) = cli::parse_args(&args) {
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
