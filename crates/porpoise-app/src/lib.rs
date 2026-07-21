use porpoise_relay::RelayClient;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder};

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let menu = build_app_menu(app)?;
            app.set_menu(menu)?;

            let tray = build_tray_icon(app)?;
            app.manage(tray);

            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            let socket_path = porpoise_core::config::AppConfig::default_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("porpoise.sock");
            if let Ok(client) = rt.block_on(RelayClient::connect(&socket_path)) {
                app.manage(client);
            } else {
                tracing::warn!("Failed to connect to daemon at {:?}", socket_path);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_worktrees,
            commands::create_worktree,
            commands::remove_worktree,
            commands::list_agents,
            commands::list_terminals,
            commands::get_status,
            commands::get_settings,
            commands::save_settings,
            commands::open_terminal_panel,
            commands::open_settings_window,
            commands::open_workflow_editor,
        ])
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "check-updates"
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.eval("window.__porpoise_check_update && window.__porpoise_check_update()");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_app_menu(app: &mut tauri::App) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let new_worktree = MenuItemBuilder::with_id("new-worktree", "New Worktree")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;

    let new_terminal = MenuItemBuilder::with_id("new-terminal", "New Terminal")
        .accelerator("CmdOrCtrl+T")
        .build(app)?;

    let settings = MenuItemBuilder::with_id("settings", "Settings")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    let check_updates = MenuItemBuilder::with_id("check-updates", "Check for Updates…")
        .build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "Quit Porpoise")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    let about = PredefinedMenuItem::about(app, Some("About Porpoise"), None)?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&new_worktree)
        .item(&new_terminal)
        .separator()
        .item(&settings)
        .separator()
        .item(&check_updates)
        .separator()
        .item(&quit)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .fullscreen()
        .separator()
        .minimize()
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .separator()
        .close_window()
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help").item(&about).build()?;

    MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()
        .map_err(Into::into)
}

fn build_tray_icon(app: &mut tauri::App) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show Porpoise").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
    let quit = MenuItemBuilder::with_id("quit-tray", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&hide)
        .separator()
        .item(&quit)
        .build()?;

    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Porpoise")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit-tray" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
