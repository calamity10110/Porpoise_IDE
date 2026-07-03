// porpoise-app crate - Tauri desktop application

/// Initializes and runs the Tauri desktop application.
///
/// This is the entry point called from `main.rs`. It sets up the Tauri builder
/// with plugins and IPC handlers. The Porpoise server daemon is started as a
/// sidecar or embedded task.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

