use std::sync::Arc;

use porpoise_core::{
    bus::EventBus,
    types::event::{SystemEvent, TerminalEvent},
};
use porpoise_relay::RelayClient;
use porpoise_runtime::PtyManager;
use tauri::{
    Emitter, Manager,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder},
};

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
            let data_dir =
                porpoise_core::config::AppConfig::default_data_dir().unwrap_or_else(|_| std::env::temp_dir());
            let socket_path = data_dir.join("porpoise.sock");
            let token_path = data_dir.join("ipc-token");

            if let Some(client) = connect_to_daemon(&rt, &socket_path, &token_path) {
                app.manage(client);
                commands::log_event("connection", "daemon connected");
            } else {
                tracing::error!("daemon unreachable — RPC features disabled until restart");
                commands::log_event("connection", "daemon UNREACHABLE");
            }
            commands::log_event("app", "started");

            let event_bus = EventBus::new(4096);
            let pty_manager = Arc::new(PtyManager::new(event_bus.clone()));
            app.manage(pty_manager.clone());
            app.manage(Arc::new(std::sync::Mutex::new(Vec::<String>::new())));
            spawn_terminal_forwarder(app.handle().clone(), event_bus.clone());
            spawn_system_telemetry(event_bus);

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
            commands::terminal_new,
            commands::terminal_input,
            commands::terminal_resize,
            commands::terminal_close,
            commands::agent_run,
            commands::agent_stop,
            commands::telemetry_log,
            commands::get_telemetry,
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

fn spawn_terminal_forwarder(app_handle: tauri::AppHandle, event_bus: EventBus) {
    let mut rx = event_bus.subscribe();
    tauri::async_runtime::spawn(async move {
        tracing::info!("terminal output forwarder started");
        // PTY chunks can split multi-byte UTF-8 sequences mid-boundary;
        // per-chunk from_utf8_lossy would corrupt them into U+FFFD. Each
        // terminal keeps its own incomplete tail until more bytes arrive.
        let mut accumulators: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
        let mut total_bytes: u64 = 0;
        let mut total_events: u64 = 0;
        loop {
            match rx.recv().await {
                Ok(SystemEvent::Terminal(TerminalEvent::Output { id, data, .. })) => {
                    total_bytes += data.len() as u64;
                    total_events += 1;
                    if total_events % 500 == 0 {
                        commands::log_event(
                            "data_flow:terminal",
                            &format!("{} events, {} bytes", total_events, total_bytes),
                        );
                    }
                    let key = id.to_string();
                    let buf = accumulators.entry(key.clone()).or_default();
                    buf.extend_from_slice(&data);
                    let text = match std::str::from_utf8(buf) {
                        Ok(s) => {
                            let out = s.to_string();
                            buf.clear();
                            out
                        }
                        Err(e) => {
                            let valid = e.valid_up_to();
                            let out = String::from_utf8_lossy(&buf[..valid]).to_string();
                            if e.error_len().is_none() {
                                let tail = buf[valid..].to_vec();
                                *buf = tail;
                            } else {
                                commands::log_event("warning:utf8", &format!("invalid utf8 at {}", valid));
                                buf.clear();
                            }
                            out
                        }
                    };
                    let payload = serde_json::json!({
                        "id": key,
                        "data": text,
                    });
                    if let Err(e) = app_handle.emit("terminal-output", payload) {
                        tracing::warn!("emit terminal-output: {e}");
                        commands::log_event("error:emit", &e.to_string());
                    }
                }
                Ok(SystemEvent::Terminal(TerminalEvent::Bell { id })) => {
                    commands::log_event("data_flow:bell", &id.to_string());
                }
                Ok(SystemEvent::Agent(ev)) => {
                    commands::log_event("data_flow:agent", &format!("{:?}", ev));
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("terminal forwarder lagged, dropped {n} events");
                    commands::log_event("warning:lagged", &format!("dropped {n} events"));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn spawn_system_telemetry(event_bus: EventBus) {
    let mut rx = event_bus.subscribe();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        let mut event_count: u64 = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    commands::log_event("system:usage", &format!("tick events={}", event_count));
                }
                ev = rx.recv() => {
                    match ev {
                        Ok(e) => {
                            event_count += 1;
                            if event_count % 1000 == 0 {
                                commands::log_event("system:usage", &format!("1000 events reached total={}", event_count));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            commands::log_event("warning:system_lagged", &format!("{n}"));
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });
}

/// Connect to the daemon, spawning it first if it is not running.
/// The server requires an auth handshake (it sends one on accept and validates
/// the session token), so plain `RelayClient::connect` would leave the
/// connection in a corrupted protocol state — always use `connect_with_auth`.
fn connect_to_daemon(
    rt: &tokio::runtime::Runtime,
    socket_path: &std::path::Path,
    token_path: &std::path::Path,
) -> Option<RelayClient> {
    for attempt in 1..=5 {
        // The daemon writes ipc-token on first boot — re-read every attempt;
        // a token captured before the daemon spawned is empty and rejected.
        let token = if token_path.exists() {
            porpoise_relay::auth::load_token(token_path).unwrap_or_default()
        } else {
            String::new()
        };

        match rt.block_on(RelayClient::connect_with_auth(socket_path, &token)) {
            Ok(client) => {
                tracing::info!("connected to daemon (attempt {attempt})");
                return Some(client);
            }
            Err(e) => {
                tracing::warn!("daemon connect attempt {attempt} failed: {e}");
                if attempt == 1 || attempt == 3 {
                    spawn_daemon();
                }
                std::thread::sleep(std::time::Duration::from_millis(750));
            }
        }
    }
    None
}

fn spawn_daemon() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let daemon_name = if cfg!(windows) {
        "porpoise-server.exe"
    } else {
        "porpoise-server"
    };
    let daemon = exe_dir.join(daemon_name);
    if daemon.exists() {
        match std::process::Command::new(&daemon)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => tracing::info!("spawned daemon (pid {})", child.id()),
            Err(e) => tracing::error!("failed to spawn daemon {:?}: {e}", daemon),
        }
    } else {
        tracing::warn!("daemon binary not found at {:?}", daemon);
    }
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

    let check_updates = MenuItemBuilder::with_id("check-updates", "Check for Updates…").build(app)?;

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
