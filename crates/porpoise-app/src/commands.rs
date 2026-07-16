use porpoise_relay::RelayClient;
use serde::{Deserialize, Serialize};
use tauri::State;

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
pub async fn list_worktrees(relay: State<'_, RelayClient>) -> Result<Vec<WorktreeInfo>, String> {
    let body = relay.call("worktree_list", serde_json::json!({})).await.map_err(|e| e.to_string())?;
    serde_json::from_value(body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_worktree(name: String, agent: Option<String>, relay: State<'_, RelayClient>) -> Result<WorktreeInfo, String> {
    let body = relay.call("worktree_create", serde_json::json!({"name": name, "agent": agent})).await.map_err(|e| e.to_string())?;
    serde_json::from_value(body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_worktree(name: String, relay: State<'_, RelayClient>) -> Result<(), String> {
    relay.call("worktree_rm", serde_json::json!({"name": name})).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_agents(relay: State<'_, RelayClient>) -> Result<Vec<AgentInfo>, String> {
    let body = relay.call("agent_list", serde_json::json!({})).await.map_err(|e| e.to_string())?;
    serde_json::from_value(body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_terminals(relay: State<'_, RelayClient>) -> Result<Vec<TerminalInfo>, String> {
    let body = relay.call("terminal_list", serde_json::json!({})).await.map_err(|e| e.to_string())?;
    serde_json::from_value(body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_status(relay: State<'_, RelayClient>) -> Result<SystemStatus, String> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let body = relay.call("health", serde_json::json!({})).await.map_err(|e| e.to_string())?;
    let uptime = body.get("uptime_seconds").and_then(|v| v.as_u64()).unwrap_or(0);
    let process_count = body.get("process_count").and_then(|v| v.as_u64()).unwrap_or(0);
    Ok(SystemStatus {
        version,
        uptime_seconds: uptime,
        worktree_count: 0,
        agent_count: process_count as usize,
        terminal_count: 0,
    })
}

#[tauri::command]
pub async fn get_settings(relay: State<'_, RelayClient>) -> Result<Settings, String> {
    let body = relay.call("config_get", serde_json::json!({"key": "app"})).await.map_err(|e| e.to_string())?;
    let theme = body.get("theme").and_then(|v| v.as_str()).unwrap_or("dark").to_string();
    let default_agent = body.get("default_agent").and_then(|v| v.as_str()).unwrap_or("claude").to_string();
    Ok(Settings { theme, default_agent, ..Settings::default() })
}

#[tauri::command]
pub async fn save_settings(settings: Settings, relay: State<'_, RelayClient>) -> Result<(), String> {
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    relay.call("config_set", serde_json::json!({"key": "app", "value": value})).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_terminal_panel(_app: tauri::AppHandle, worktree_id: String, relay: State<'_, RelayClient>) -> Result<(), String> {
    relay.call("terminal_create", serde_json::json!({"worktree_id": worktree_id})).await.map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn open_workflow_editor(app: tauri::AppHandle) -> Result<(), String> {
    tauri::WebviewWindowBuilder::new(&app, "workflow", tauri::WebviewUrl::App("workflow.html".into()))
        .title("Workflow Editor")
        .inner_size(1200.0, 800.0)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}
