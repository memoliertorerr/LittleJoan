fn main() {
    // App commands are allowed only where a capability grants them (capabilities/*.json), so a
    // window can call exactly the commands it needs and remote pages none.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "open_input",
            "hibernate",
            "start_info",
            "get_settings",
            "save_settings",
            "suspend_shortcut",
        ]),
    ))
    .expect("failed to run tauri-build");
}
