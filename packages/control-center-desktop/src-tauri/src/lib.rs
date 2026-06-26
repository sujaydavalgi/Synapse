//! Tauri desktop shell for Spanda Control Center.
//!

/// Start the Control Center desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Launch the Tauri desktop shell for Control Center.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Does not return on success (runs the event loop).
    //
    // Options:
    // Set `SPANDA_CONTROL_CENTER_URL` for the default API base exposed to the webview.
    //
    // Example:
    // spanda_control_center_desktop_lib::run();

    // Build the Tauri app with shell plugin and default API URL command.
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![default_api_base])
        .run(tauri::generate_context!())
        .expect("error while running Spanda Control Center desktop");
}

/// Return the default Control Center API URL for the webview.
#[tauri::command]
fn default_api_base() -> String {
    // Resolve the API base URL from the environment or local dev default.
    //
    // Parameters:
    // None (Tauri command).
    //
    // Returns:
    // Control Center REST base URL string.
    //
    // Options:
    // `SPANDA_CONTROL_CENTER_URL` overrides the default `http://127.0.0.1:8080`.
    //
    // Example:
    // SPANDA_CONTROL_CENTER_URL=http://fleet:8080 spanda-control-center-desktop

    // Prefer SPANDA_CONTROL_CENTER_URL; fall back to local dev server.
    std::env::var("SPANDA_CONTROL_CENTER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into())
}
