use std::sync::Arc;

use chrono::{DateTime, Utc};
use porpoise_relay::Router;
use porpoise_runtime::PtyManager;

pub fn register_all(router: &mut Router, start_time: DateTime<Utc>, pty_manager: Arc<PtyManager>) {
    let state = router.state().clone();

    let s = state.clone();
    router.register(
        "worktree_create",
        Arc::new(move |req, _| {
            let name = req.params.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "unnamed".into());
            let repo = req.params.get("repo").and_then(|v| v.as_str()).map(|s| s.to_string());
            let s = s.clone();
            Box::pin(async move { super::worktree::handle_create(&s, &name, repo.as_deref()).await })
        }),
    );

    let s = state.clone();
    router.register(
        "worktree_list",
        Arc::new(move |_, _| {
            let s = s.clone();
            Box::pin(async move { super::worktree::handle_list(&s).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_create",
        Arc::new(move |req, _| {
            let rows = req.params.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;
            let cols = req.params.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
            let shell = req.params.get("shell").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "bash".into());
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_create_pty(&pty, rows, cols, &shell).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_send",
        Arc::new(move |req, _| {
            let id = req.params.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let data = req.params.get("data").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_send_pty(&pty, &id, &data).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_read",
        Arc::new(move |req, _| {
            let id = req.params.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let max = req.params.get("max_bytes").and_then(|v| v.as_u64()).unwrap_or(1024) as usize;
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_read_pty(&pty, &id, max).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_resize",
        Arc::new(move |req, _| {
            let id = req.params.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let rows = req.params.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;
            let cols = req.params.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_resize_pty(&pty, &id, rows, cols).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_close",
        Arc::new(move |req, _| {
            let id = req.params.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_close_pty(&pty, &id).await })
        }),
    );

    router.register(
        "agent_list",
        Arc::new(move |_, _| Box::pin(async move { super::agent::handle_list().await })),
    );

    router.register(
        "agent_detect",
        Arc::new(move |_, _| Box::pin(async move { super::agent::handle_detect().await })),
    );

    let s = state.clone();
    router.register(
        "config_get",
        Arc::new(move |req, _| {
            let key = req.params.get("key").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let s = s.clone();
            Box::pin(async move { super::config::handle_get(&s, &key).await })
        }),
    );

    router.register(
        "git_status",
        Arc::new(move |req, _| {
            let path = req.params.get("path").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| ".".into());
            Box::pin(async move { super::git::handle_status(&path).await })
        }),
    );

    router.register(
        "git_clone",
        Arc::new(move |req, _| {
            let url = req.params.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            let path = req.params.get("path").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| ".".into());
            Box::pin(async move { super::git::handle_clone(&url, &path).await })
        }),
    );

    router.register(
        "ssh_connect",
        Arc::new(move |req, _| {
            let host = req.params.get("host").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "localhost".into());
            let port = req.params.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
            let user = req.params.get("user").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "root".into());
            Box::pin(async move { super::ssh_service::handle_connect(&host, port, &user).await })
        }),
    );

    router.register(
        "browser_open",
        Arc::new(move |req, _| {
            let url = req.params.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            Box::pin(async move { super::browser_service::handle_open(&url).await })
        }),
    );

    router.register(
        "skill_list",
        Arc::new(move |_, _| Box::pin(async move { super::skills_service::handle_list().await })),
    );

    let st = start_time;
    router.register(
        "health",
        Arc::new(move |_, _| Box::pin(async move { super::health::handle_health(&st, 0).await })),
    );
}
