use porpoise_core::{error::Result, state::AppState, types::id::WorktreeId};

pub async fn handle_create(state: &AppState, name: &str, repo: Option<&str>) -> Result<serde_json::Value> {
    let id = WorktreeId::new();
    {
        let mut wts = state.worktrees_mut().await;
        wts.insert(
            id,
            porpoise_core::state::WorktreeState {
                id,
                path: std::path::PathBuf::from(name),
                name: name.into(),
                status: porpoise_core::types::event::WorktreeStatus::Idle,
                agent_id: None,
            },
        );
    }
    Ok(serde_json::json!({ "id": id.to_string(), "name": name, "repo": repo, "status": "created" }))
}

pub async fn handle_list(state: &AppState) -> Result<serde_json::Value> {
    let wts = state.worktrees().await;
    let list: Vec<serde_json::Value> = wts
        .values()
        .map(|wt| {
            serde_json::json!({
                "id": wt.id.to_string(),
                "name": wt.name,
                "status": format!("{:?}", wt.status),
            })
        })
        .collect();
    Ok(serde_json::json!({ "worktrees": list }))
}
