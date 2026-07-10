use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub id: String,
    pub session_id: String,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub worktree_count: usize,
    pub agent_count: usize,
    pub terminal_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub default_agent: String,
    pub max_concurrent_agents: u32,
    pub terminal_font: String,
    pub terminal_font_size: u32,
    pub terminal_shell: String,
    pub enable_notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            default_agent: "claude".into(),
            max_concurrent_agents: 5,
            terminal_font: "JetBrains Mono".into(),
            terminal_font_size: 14,
            terminal_shell: "bash".into(),
            enable_notifications: true,
        }
    }
}

#[tauri::command]
pub async fn list_worktrees() -> Result<Vec<WorktreeInfo>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn create_worktree(name: String, _agent: Option<String>) -> Result<WorktreeInfo, String> {
    Ok(WorktreeInfo {
        name,
        path: String::new(),
        branch: "main".into(),
    })
}

#[tauri::command]
pub async fn remove_worktree(_name: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn list_agents() -> Result<Vec<AgentInfo>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn list_terminals() -> Result<Vec<TerminalInfo>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn get_status() -> Result<SystemStatus, String> {
    Ok(SystemStatus {
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_seconds: 0,
        worktree_count: 0,
        agent_count: 0,
        terminal_count: 0,
    })
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    Ok(Settings::default())
}

#[tauri::command]
pub async fn save_settings(_settings: Settings) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn open_terminal_panel(app: tauri::AppHandle, worktree_id: String) -> Result<(), String> {
    let _ = app;
    let _ = worktree_id;
    Ok(())
}

#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    tauri::WebviewWindowBuilder::new(&app, "settings", tauri::WebviewUrl::App("index.html#settings".into()))
        .title("Settings")
        .inner_size(600.0, 400.0)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}
