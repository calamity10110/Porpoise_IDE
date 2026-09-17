use std::sync::Arc;

use chrono::{DateTime, Utc};
use porpoise_agent::AgentPool;
use porpoise_core::state::AppState;
use porpoise_git::cache::GitStatusCache;
use porpoise_relay::Router;
use porpoise_runtime::PtyManager;

use crate::services::esp32_service;

pub fn register_all(
    router: &mut Router,
    start_time: DateTime<Utc>,
    pty_manager: Arc<PtyManager>,
    agent_pool: Arc<AgentPool>,
    state: AppState,
    session_token: String,
    tls_fingerprint: Option<String>,
) {
    let st = state.clone();
    router.register(
        "worktree_create",
        Arc::new(move |req, _| {
            let name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unnamed".into());
            let repo = req.params.get("repo").and_then(|v| v.as_str()).map(|s| s.to_string());
            let st = st.clone();
            Box::pin(async move { super::worktree::handle_create(&st, &name, repo.as_deref()).await })
        }),
    );

    let st = state.clone();
    router.register(
        "worktree_list",
        Arc::new(move |_, _| {
            let st = st.clone();
            Box::pin(async move { super::worktree::handle_list(&st).await })
        }),
    );

    let st = state.clone();
    router.register(
        "worktree_rm",
        Arc::new(move |req, _| {
            let name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let st = st.clone();
            Box::pin(async move { super::worktree::handle_remove(&st, &name).await })
        }),
    );

    let st = state.clone();
    router.register(
        "worktree_prune",
        Arc::new(move |req, _| {
            let dry_run = req.params.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);
            let st = st.clone();
            Box::pin(async move { super::worktree::handle_prune(&st, dry_run).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_create",
        Arc::new(move |req, _| {
            let rows = req.params.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;
            let cols = req.params.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;
            let shell = req
                .params
                .get("shell")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "bash".into());
            let worktree_id = req
                .params
                .get("worktree_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let pty = pty.clone();
            Box::pin(async move {
                super::terminal::handle_create_pty(&pty, rows, cols, &shell, worktree_id.as_deref()).await
            })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_list",
        Arc::new(move |_, _| {
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_list_terminals(&pty).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_send",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let data = req
                .params
                .get("data")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_send_pty(&pty, &id, &data).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_read",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let max = req.params.get("max_bytes").and_then(|v| v.as_u64()).unwrap_or(4096) as usize;
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_read_pty(&pty, &id, max).await })
        }),
    );

    let pty = pty_manager.clone();
    router.register(
        "terminal_resize",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
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
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let pty = pty.clone();
            Box::pin(async move { super::terminal::handle_close_pty(&pty, &id).await })
        }),
    );

    let pool = agent_pool.clone();
    router.register(
        "agent_list",
        Arc::new(move |_, _| {
            let pool = pool.clone();
            Box::pin(async move { super::agent::handle_list(&pool).await })
        }),
    );

    router.register(
        "agent_detect",
        Arc::new(move |_, _| Box::pin(async move { super::agent::handle_detect().await })),
    );

    let pool = agent_pool.clone();
    router.register(
        "agent_run",
        Arc::new(move |req, _| {
            let kind = req
                .params
                .get("kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "opencode".into());
            let worktree = req
                .params
                .get("worktree")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".into());
            let prompt = req
                .params
                .get("prompt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let pool = pool.clone();
            Box::pin(async move { super::agent::handle_run(&pool, &kind, &worktree, &prompt).await })
        }),
    );

    let pool = agent_pool.clone();
    router.register(
        "agent_stop",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let pool = pool.clone();
            Box::pin(async move { super::agent::handle_stop(&pool, &id).await })
        }),
    );

    let pool = agent_pool.clone();
    router.register(
        "agent_logs",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let lines = req.params.get("lines").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
            let pool = pool.clone();
            Box::pin(async move { super::agent::handle_logs(&pool, &id, lines).await })
        }),
    );

    let st = state.clone();
    router.register(
        "config_get",
        Arc::new(move |req, _| {
            let key = req
                .params
                .get("key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let st = st.clone();
            Box::pin(async move { super::config::handle_get(&st, &key).await })
        }),
    );

    let st = state.clone();
    router.register(
        "config_set",
        Arc::new(move |req, _| {
            let key = req
                .params
                .get("key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let value = req.params.get("value").cloned().unwrap_or(serde_json::Value::Null);
            let st = st.clone();
            Box::pin(async move { super::config::handle_set(&st, &key, value).await })
        }),
    );

    let st = state.clone();
    router.register(
        "config_list",
        Arc::new(move |_, _| {
            let st = st.clone();
            Box::pin(async move { super::config::handle_list(&st).await })
        }),
    );

    let status_cache = GitStatusCache::new();

    let cache = status_cache.clone();
    router.register(
        "git_status",
        Arc::new(move |req, _| {
            let path = req
                .params
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".into());
            let cache = cache.clone();
            Box::pin(async move { super::git::handle_status(&path, &cache).await })
        }),
    );

    router.register(
        "git_clone",
        Arc::new(move |req, _| {
            let url = req
                .params
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let path = req
                .params
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".into());
            Box::pin(async move { super::git::handle_clone(&url, &path).await })
        }),
    );

    let cache = status_cache.clone();
    router.register(
        "git_diff",
        Arc::new(move |req, _| {
            let path = req
                .params
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".into());
            let staged = req.params.get("staged").and_then(|v| v.as_bool()).unwrap_or(false);
            let cache = cache.clone();
            Box::pin(async move { super::git::handle_diff(&path, staged, &cache).await })
        }),
    );

    let cache = status_cache.clone();
    router.register(
        "git_log",
        Arc::new(move |req, _| {
            let path = req
                .params
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".into());
            let count = req.params.get("count").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            let cache = cache.clone();
            Box::pin(async move { super::git::handle_log(&path, count, &cache).await })
        }),
    );

    let cache = status_cache.clone();
    router.register(
        "git_branch",
        Arc::new(move |req, _| {
            let path = req
                .params
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| ".".into());
            let action = req
                .params
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("list")
                .to_string();
            let name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let cache = cache.clone();
            Box::pin(async move {
                match action.as_str() {
                    "create" => super::git::handle_branch_create(&path, &name).await,
                    "checkout" => super::git::handle_branch_checkout(&path, &name).await,
                    _ => super::git::handle_branch_list(&path, &cache).await,
                }
            })
        }),
    );

    router.register(
        "ssh_connect",
        Arc::new(move |req, _| {
            let host = req
                .params
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "localhost".into());
            let port = req.params.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
            let user = req
                .params
                .get("user")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "root".into());
            let password = req
                .params
                .get("password")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let key_path = req
                .params
                .get("key_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Box::pin(async move {
                super::ssh_service::handle_connect(&host, port, &user, password.as_deref(), key_path.as_deref()).await
            })
        }),
    );

    router.register(
        "ssh_worktree",
        Arc::new(move |req, _| {
            let host = req
                .params
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "localhost".into());
            let port = req.params.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
            let user = req
                .params
                .get("user")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "root".into());
            let repo = req
                .params
                .get("repo")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let branch = req
                .params
                .get("branch")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "main".into());
            Box::pin(async move { super::ssh_service::handle_worktree(&host, port, &user, &repo, &branch).await })
        }),
    );

    router.register(
        "ssh_port_forward",
        Arc::new(move |req, _| {
            let host = req
                .params
                .get("host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "localhost".into());
            let port = req.params.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
            let user = req
                .params
                .get("user")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "root".into());
            let target_host = req
                .params
                .get("target_host")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "localhost".into());
            let target_port = req.params.get("target_port").and_then(|v| v.as_u64()).unwrap_or(8080) as u16;
            Box::pin(async move {
                super::ssh_service::handle_port_forward(&host, port, &user, &target_host, target_port).await
            })
        }),
    );

    router.register(
        "browser_open",
        Arc::new(move |req, _| {
            let url = req
                .params
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            Box::pin(async move { super::browser_service::handle_open(&url).await })
        }),
    );

    router.register(
        "browser_snapshot",
        Arc::new(move |req, _| {
            let url = req
                .params
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            Box::pin(async move { super::browser_service::handle_snapshot(&url).await })
        }),
    );

    router.register(
        "skill_list",
        Arc::new(move |_, _| Box::pin(async move { super::skills_service::handle_list().await })),
    );

    router.register(
        "skill_search",
        Arc::new(move |req, _| {
            let query = req
                .params
                .get("query")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            Box::pin(async move { super::skills_service::handle_search(&query).await })
        }),
    );

    router.register(
        "skill_install",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            Box::pin(async move { super::skills_service::handle_install(&id).await })
        }),
    );

    router.register(
        "skill_uninstall",
        Arc::new(move |req, _| {
            let id = req
                .params
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            Box::pin(async move { super::skills_service::handle_uninstall(&id).await })
        }),
    );

    let pool = agent_pool.clone();
    router.register(
        "health",
        Arc::new(move |_, _| {
            let pool = pool.clone();
            Box::pin(async move {
                let count = pool.list().await.len();
                super::health::handle_health(&start_time, count).await
            })
        }),
    );

    router.register(
        "daemon_stop",
        Arc::new(|_, _| {
            Box::pin(async move {
                tracing::info!("daemon_stop requested via RPC");
                Ok(serde_json::json!({ "status": "stopping" }))
            })
        }),
    );

router.register(
        "mobile/pairing_info",
        Arc::new(move |_, _| {
            let fp = tls_fingerprint.clone();
            let token = session_token.clone();
            Box::pin(async move {
                let port_str = std::env::var("PORPOISE_WS_PORT").unwrap_or_default();
                let tls_enabled = std::env::var("PORPOISE_WS_TLS").as_deref() == Ok("1");
                Ok(serde_json::json!({
                    "host": std::env::var("PORPOISE_WS_HOST").unwrap_or_else(|_| "localhost".to_string()),
                    "port": port_str,
                    "tls_enabled": tls_enabled,
                    "tls_fingerprint": fp.unwrap_or_default(),
                    "token": token,
                }))
            })
        }),
    );

    register_esp32(router);

    fn register_esp32(router: &mut Router) {
        let esp_registry = esp32_service::new_registry();

        {
            let reg = esp_registry.clone();
            router.register(
                "esp32/list",
                Arc::new(move |_, _| {
                    let reg = reg.clone();
                    Box::pin(async move { esp32_service::handle_list(&reg).await })
                }),
            );
        }
        {
            let reg = esp_registry.clone();
            router.register(
                "esp32/get",
                Arc::new(move |req, _| {
                    let reg = reg.clone();
                    Box::pin(async move {
                        let id = req.params.get("device_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        esp32_service::handle_get(&reg, id).await
                    })
                }),
            );
        }
        router.register(
            "esp32/command",
            Arc::new(|req, _| {
                Box::pin(async move {
                    let id = req.params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                    let target = req.params.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    let action = req.params.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let args = req.params.get("args").cloned().unwrap_or(serde_json::Value::Null);
                    esp32_service::handle_command(id, target, action, args).await
                })
            }),
        );
        router.register(
            "esp32/ota_push",
            Arc::new(|req, _| {
                Box::pin(async move {
                    let id = req.params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                    let url = req.params.get("firmware_url").and_then(|v| v.as_str()).unwrap_or("");
                    let ck = req.params.get("checksum").and_then(|v| v.as_str()).unwrap_or("");
                    esp32_service::handle_ota_push(id, url, ck).await
                })
            }),
        );
        router.register(
            "esp32/board_templates",
            Arc::new(|_, _| {
                Box::pin(async move { esp32_service::handle_board_templates().await })
            }),
        );
    }

     // ── ESP32-S3 Device Management ──────────────────────────────────
    let esp_registry = esp32_service::new_registry();

    {
        let reg = esp_registry.clone();
        router.register(
            "esp32/list",
            Arc::new(move |_, _| {
                let reg = reg.clone();
                Box::pin(async move { esp32_service::handle_list(&reg).await })
            }),
        );
    }
    {
        let reg = esp_registry.clone();
        router.register(
            "esp32/get",
            Arc::new(move |req, _| {
                let reg = reg.clone();
                Box::pin(async move {
                    let id = req.params.get("device_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    esp32_service::handle_get(&reg, id).await
                })
            }),
        );
    }
    router.register(
        "esp32/command",
        Arc::new(|req, _| {
            Box::pin(async move {
                let id = req.params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                let target = req.params.get("target").and_then(|v| v.as_str()).unwrap_or("");
                let action = req.params.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let args = req.params.get("args").cloned().unwrap_or(serde_json::Value::Null);
                esp32_service::handle_command(id, target, action, args).await
            })
        }),
    );
    router.register(
        "esp32/ota_push",
        Arc::new(|req, _| {
            Box::pin(async move {
                let id = req.params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                let url = req.params.get("firmware_url").and_then(|v| v.as_str()).unwrap_or("");
                let ck = req.params.get("checksum").and_then(|v| v.as_str()).unwrap_or("");
                esp32_service::handle_ota_push(id, url, ck).await
            })
        }),
    );
    router.register(
        "esp32/board_templates",
        Arc::new(|_, _| {
            Box::pin(async move { esp32_service::handle_board_templates().await })
        }),
    );
}
