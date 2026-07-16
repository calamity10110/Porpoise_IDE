use std::path::Path;

use porpoise_agent::AgentPool;
use porpoise_core::error::Result;

pub async fn handle_list(pool: &AgentPool) -> Result<serde_json::Value> {
    let agents = pool.list().await;
    let list: Vec<serde_json::Value> = agents
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id.to_string(),
                "kind": a.kind.to_string(),
                "pid": a.pid,
                "status": format!("{:?}", a.status),
                "worktree": a.worktree_path.as_ref().map(|p| p.display().to_string()),
            })
        })
        .collect();
    Ok(serde_json::json!({ "agents": list }))
}

pub async fn handle_detect() -> Result<serde_json::Value> {
    let manifests = porpoise_agent::AgentDetector::detect_all();
    let list: Vec<serde_json::Value> = manifests
        .iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "binary": m.binary,
                "kind": m.kind.to_string(),
                "detected": m.detected,
                "version": m.version,
            })
        })
        .collect();
    Ok(serde_json::json!({ "agents": list }))
}

pub async fn handle_run(pool: &AgentPool, kind: &str, worktree: &str, prompt: &str) -> Result<serde_json::Value> {
    let agent_kind = match kind {
        "claude" | "claude-code" => porpoise_agent::types::AgentKind::ClaudeCode,
        "codex" => porpoise_agent::types::AgentKind::Codex,
        "gemini" => porpoise_agent::types::AgentKind::Gemini,
        "opencode" => porpoise_agent::types::AgentKind::OpenCode,
        "z" | "zai" => porpoise_agent::types::AgentKind::ZAI,
        "openai" => porpoise_agent::types::AgentKind::OpenAI,
        "grok" => porpoise_agent::types::AgentKind::Grok,
        "openrouter" => porpoise_agent::types::AgentKind::OpenRouter,
        other => porpoise_agent::types::AgentKind::Custom(other.to_string()),
    };

    let worktree_path = Path::new(worktree);
    let id = pool.spawn(agent_kind, worktree_path).await?;
    Ok(serde_json::json!({
        "id": id.to_string(),
        "kind": kind,
        "worktree": worktree,
        "prompt": prompt,
        "status": "started"
    }))
}

pub async fn handle_stop(pool: &AgentPool, id_str: &str) -> Result<serde_json::Value> {
    let id: porpoise_core::types::id::AgentId = id_str
        .parse()
        .map_err(|_| porpoise_core::error::PorpoiseError::Agent(format!("invalid agent id: {id_str}")))?;
    pool.shutdown(id).await?;
    Ok(serde_json::json!({ "id": id_str, "status": "stopped" }))
}

pub async fn handle_logs(pool: &AgentPool, _id_str: &str, _lines: usize) -> Result<serde_json::Value> {
    let agents = pool.list().await;
    let logs: Vec<serde_json::Value> = agents
        .iter()
        .filter(|a| a.id.to_string().starts_with(_id_str))
        .map(|a| {
            serde_json::json!({
                "id": a.id.to_string(),
                "kind": a.kind.to_string(),
                "status": format!("{:?}", a.status),
            })
        })
        .collect();
    Ok(serde_json::json!({ "logs": logs }))
}
