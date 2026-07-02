use std::sync::Arc;
use porpoise_relay::Router;

pub fn register_all(router: &Router) {
    let s = router.state();

    router.register("worktree_create", Arc::new(move |req, _| {
        let name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed");
        let repo = req.params.get("repo").and_then(|v| v.as_str());
        let s = s.clone();
        Box::pin(async move { super::worktree::handle_create(&s, name, repo).await })
    }));

    router.register("worktree_list", Arc::new(move |_, _| {
        let s = s.clone();
        Box::pin(async move { super::worktree::handle_list(&s).await })
    }));

    router.register("terminal_create", Arc::new(move |_, _| {
        let s = s.clone();
        Box::pin(async move { super::terminal::handle_create(&s).await })
    }));

    router.register("agent_list", Arc::new(move |_, _| {
        Box::pin(async move { super::agent::handle_list().await })
    }));

    router.register("agent_detect", Arc::new(move |_, _| {
        Box::pin(async move { super::agent::handle_detect().await })
    }));

    router.register("config_get", Arc::new(move |req, _| {
        let key = req.params.get("key").and_then(|v| v.as_str()).unwrap_or("");
        let s = s.clone();
        Box::pin(async move { super::config::handle_get(&s, key).await })
    }));

    router.register("git_status", Arc::new(move |req, _| {
        let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        Box::pin(async move { super::git::handle_status(path).await })
    }));

    router.register("git_clone", Arc::new(move |req, _| {
        let url = req.params.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        Box::pin(async move { super::git::handle_clone(url, path).await })
    }));
}
