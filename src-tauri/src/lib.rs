//! Little Joan: a tiny pop-up browser that hibernates until a global shortcut calls it.
//!
//! At rest the process has a tray icon and nothing else — no window and no webview. See
//! docs/architecture.md for the lifecycle.

mod tray;

use tauri::RunEvent;

/// Builds and runs the app. Returns only when the user quits.
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            // A menu-bar utility: no Dock icon on macOS.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::create(app.handle())?;
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
