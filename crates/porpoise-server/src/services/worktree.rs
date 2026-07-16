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

pub async fn handle_remove(state: &AppState, name: &str) -> Result<serde_json::Value> {
    let mut wts = state.worktrees_mut().await;
    let id_to_remove = wts.iter().find(|(_, wt)| wt.name == name).map(|(id, _)| *id);
    if let Some(id) = id_to_remove {
        wts.remove(&id);
        Ok(serde_json::json!({ "name": name, "status": "removed" }))
    } else {
        Ok(serde_json::json!({ "name": name, "status": "not_found" }))
    }
}

pub async fn handle_prune(state: &AppState, dry_run: bool) -> Result<serde_json::Value> {
    let mut wts = state.worktrees_mut().await;
    let stale: Vec<String> = wts
        .iter()
        .filter(|(_, wt)| matches!(wt.status, porpoise_core::types::event::WorktreeStatus::Idle))
        .map(|(_, wt)| wt.name.clone())
        .collect();
    if !dry_run {
        let to_remove: Vec<WorktreeId> = wts
            .iter()
            .filter(|(_, wt)| matches!(wt.status, porpoise_core::types::event::WorktreeStatus::Idle))
            .map(|(id, _)| *id)
            .collect();
        for id in to_remove {
            wts.remove(&id);
        }
    }
    Ok(serde_json::json!({ "pruned": stale, "dry_run": dry_run }))
}
