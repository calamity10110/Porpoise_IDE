use std::{path::PathBuf, sync::Arc};

use porpoise_core::types::id::TerminalId;
use porpoise_relay::RelayClient;
use porpoise_runtime::PtyManager;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub id: String,
    pub name: String,
    pub status: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            default_agent: "opencode".into(),
            max_concurrent_agents: 5,
            terminal_font: "JetBrains Mono".into(),
            terminal_font_size: 14,
            terminal_shell: default_shell().into(),
            enable_notifications: true,
        }
    }
}

pub fn default_shell() -> &'static str {
    if cfg!(windows) { "powershell" } else { "bash" }
}

fn settings_path() -> PathBuf {
    let data_dir = porpoise_core::config::AppConfig::default_data_dir().unwrap_or_else(|_| std::env::temp_dir());
    data_dir.join("settings.json")
}

fn load_settings_from_disk() -> Settings {
    let path = settings_path();
    if let Ok(raw) = std::fs::read_to_string(&path)
        && let Ok(settings) = serde_json::from_str::<Settings>(&raw)
    {
        return settings;
    }
    Settings::default()
}

// ── Worktrees (via daemon relay) ─────────────────────────────────

#[tauri::command]
pub async fn list_worktrees(relay: State<'_, RelayClient>) -> Result<Vec<WorktreeInfo>, String> {
    let body = relay
        .call("worktree_list", serde_json::json!({}))
        .await
        .map_err(|e| e.to_string())?;
    let list = body.get("worktrees").cloned().unwrap_or(serde_json::json!([]));
    serde_json::from_value(list).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_worktree(
    name: String,
    agent: Option<String>,
    relay: State<'_, RelayClient>,
) -> Result<WorktreeInfo, String> {
    let _ = agent;
    let body = relay
        .call("worktree_create", serde_json::json!({"name": name}))
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_value(body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_worktree(name: String, relay: State<'_, RelayClient>) -> Result<(), String> {
    relay
        .call("worktree_rm", serde_json::json!({"name": name}))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Agents / status (via daemon relay) ───────────────────────────

#[tauri::command]
pub async fn list_agents(relay: State<'_, RelayClient>) -> Result<Vec<AgentInfo>, String> {
    let body = relay
        .call("agent_list", serde_json::json!({}))
        .await
        .map_err(|e| e.to_string())?;
    let list = body.get("agents").cloned().unwrap_or(serde_json::json!([]));
    serde_json::from_value(list).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_terminals(relay: State<'_, RelayClient>) -> Result<Vec<TerminalInfo>, String> {
    let _ = relay;
    Ok(Vec::new())
}

#[tauri::command]
pub async fn get_status(relay: State<'_, RelayClient>) -> Result<SystemStatus, String> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let body = relay
        .call("health", serde_json::json!({}))
        .await
        .map_err(|e| e.to_string())?;
    let uptime = body.get("uptime_seconds").and_then(|v| v.as_u64()).unwrap_or(0);
    let agents = body.get("agents").and_then(|v| v.as_u64()).unwrap_or(0);
    Ok(SystemStatus {
        version,
        uptime_seconds: uptime,
        worktree_count: 0,
        agent_count: agents as usize,
        terminal_count: 0,
    })
}

// ── Settings (persisted to data_dir/settings.json) ───────────────

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    Ok(load_settings_from_disk())
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    let path = settings_path();
    let raw = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| format!("write {}: {e}", path.display()))
}

// ── Terminals (local PTY, VS Code architecture) ──────────────────

#[tauri::command]
pub async fn terminal_new(
    rows: Option<u16>,
    cols: Option<u16>,
    shell: Option<String>,
    active: State<'_, Arc<std::sync::Mutex<Vec<String>>>>,
    pty: State<'_, Arc<PtyManager>>,
) -> Result<String, String> {
    let settings = load_settings_from_disk();
    let shell = shell.unwrap_or_else(|| {
        if settings.terminal_shell.is_empty() {
            default_shell().to_string()
        } else {
            settings.terminal_shell.clone()
        }
    });
    let rows = rows.unwrap_or(24);
    let cols = cols.unwrap_or(80);
    let id = match pty.alloc(rows, cols, &shell).await {
        Ok(id) => id,
        Err(e) => {
            let fallback = default_shell();
            tracing::warn!("spawn '{shell}' failed ({e}); falling back to '{fallback}'");
            pty.alloc(rows, cols, fallback)
                .await
                .map_err(|e2| format!("spawn '{shell}': {e}; fallback '{fallback}': {e2}"))?
        }
    };
    let id_str = id.to_string();
    active.lock().map_err(|e| e.to_string())?.push(id_str.clone());
    Ok(id_str)
}

#[tauri::command]
pub async fn terminal_input(id: String, data: String, pty: State<'_, Arc<PtyManager>>) -> Result<(), String> {
    let tid: TerminalId = id.parse().map_err(|_| "invalid terminal id".to_string())?;
    pty.write(tid, data.as_bytes()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn terminal_resize(id: String, rows: u16, cols: u16, pty: State<'_, Arc<PtyManager>>) -> Result<(), String> {
    let tid: TerminalId = id.parse().map_err(|_| "invalid terminal id".to_string())?;
    pty.resize(tid, rows, cols).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn terminal_close(
    id: String,
    active: State<'_, Arc<std::sync::Mutex<Vec<String>>>>,
    pty: State<'_, Arc<PtyManager>>,
) -> Result<(), String> {
    let tid: TerminalId = id.parse().map_err(|_| "invalid terminal id".to_string())?;
    pty.close(tid).await.map_err(|e| e.to_string())?;
    if let Ok(mut list) = active.lock() {
        list.retain(|x| x != &id);
    }
    Ok(())
}

// ── Agents (via daemon relay) ────────────────────────────────────

#[tauri::command]
pub async fn agent_run(
    kind: Option<String>,
    worktree: Option<String>,
    prompt: Option<String>,
    relay: State<'_, RelayClient>,
) -> Result<serde_json::Value, String> {
    let settings = load_settings_from_disk();
    let kind = kind.unwrap_or_else(|| settings.default_agent.clone());
    let worktree = worktree.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".into())
    });
    relay
        .call(
            "agent_run",
            serde_json::json!({"kind": kind, "worktree": worktree, "prompt": prompt.unwrap_or_default()}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_stop(id: String, relay: State<'_, RelayClient>) -> Result<(), String> {
    relay
        .call("agent_stop", serde_json::json!({"id": id}))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Telemetry (local JSONL, user-reviewable) ─────────────────────

fn telemetry_path() -> PathBuf {
    let data_dir = porpoise_core::config::AppConfig::default_data_dir().unwrap_or_else(|_| std::env::temp_dir());
    data_dir.join("telemetry.log")
}

static TELEMETRY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn log_event(kind: &str, detail: &str) {
    let event = serde_json::json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "kind": kind,
        "detail": detail,
    });
    // serialize appends: concurrent writers interleave and corrupt JSONL lines
    if let Ok(_guard) = TELEMETRY_LOCK.lock() {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(telemetry_path())
        {
            use std::io::Write;
            let _ = writeln!(f, "{event}");
        }
    }
}

#[tauri::command]
pub async fn telemetry_log(kind: String, detail: String) -> Result<(), String> {
    log_event(&kind, &detail);
    Ok(())
}

#[tauri::command]
pub async fn get_telemetry(limit: Option<usize>) -> Result<serde_json::Value, String> {
    let path = telemetry_path();
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = raw.lines().collect();
    let limit = limit.unwrap_or(200).min(lines.len());
    let recent: Vec<serde_json::Value> = lines[lines.len() - limit..]
        .iter()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    let mut counts: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for l in raw.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(l)
            && let Some(kind) = v.get("kind").and_then(|k| k.as_str())
        {
            *counts.entry(kind.to_string()).or_default() += 1;
        }
    }

    let errors: Vec<&serde_json::Value> = recent
        .iter()
        .filter(|e| {
            e.get("detail")
                .and_then(|d| d.as_str())
                .map_or(false, |d| d.starts_with("err"))
        })
        .collect();

    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "total_events": lines.len(),
        "counts": counts,
        "error_count_recent": errors.len(),
        "recent": recent,
    }))
}

// ── Window helpers ───────────────────────────────────────────────

#[tauri::command]
pub async fn open_terminal_panel(
    _app: tauri::AppHandle,
    worktree_id: String,
    relay: State<'_, RelayClient>,
) -> Result<(), String> {
    relay
        .call("terminal_create", serde_json::json!({"worktree_id": worktree_id}))
        .await
        .map_err(|e| e.to_string())?;
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
