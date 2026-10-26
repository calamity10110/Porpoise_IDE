//! Sandbox lifecycle integration test: drives the real relay Router with
//! `register_all` through the full SANDBOX_SPEC v1 flow — provision via
//! RPC, files land in the configured data dir, start fails honestly with
//! missing prerequisites (no KVM/QEMU/images on the test host), list,
//! clean, detect. Exercises the exact handlers the CLI/app will call.

use porpoise_core::{bus::EventBus, config::AppConfig, state::AppState};
use porpoise_relay::{Request, Router, StatusCode};
use porpoise_server::services::register_all;

fn router_with_temp_data_dir(data_dir: &std::path::Path) -> Router {
    let mut config = AppConfig::default();
    config.core.data_dir = Some(data_dir.to_path_buf());
    let bus = EventBus::new(64);
    let state = AppState::new(config, bus.clone());
    let pty = std::sync::Arc::new(porpoise_runtime::PtyManager::new(bus));
    let agents = std::sync::Arc::new(porpoise_agent::AgentPool::new(2));
    let mut router = Router::new(state.clone());
    register_all(
        &mut router,
        chrono::Utc::now(),
        pty,
        agents,
        state,
        "test-token".into(),
        None,
    );
    router
}

fn dispatch(router: &Router, method: &str, params: serde_json::Value) -> porpoise_relay::message::Response {
    block_on(router.dispatch(Request::new(method, params)))
}

// The relay handlers are async; run them on a tiny current-thread runtime
// without pulling tokio::test into every assert.
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(fut)
}

#[test]
fn sandbox_provision_start_fail_clean_flow_over_the_router() {
    let dir = tempfile::tempdir().unwrap();
    let router = router_with_temp_data_dir(dir.path());

    // Provision through the router.
    let resp = dispatch(&router, "sandbox_provision", serde_json::json!({ "id": "demo" }));
    assert_eq!(resp.status, StatusCode::Ok, "body: {}", resp.body);
    assert_eq!(resp.body["state"], "stopped");
    assert_eq!(resp.body["cpus"], 1, "spec default 1 vCPU");
    assert_eq!(resp.body["memory_mb"], 512, "spec default 512 MB");

    // Spec file layout, under the CONFIGURED data dir.
    let project = dir.path().join("sandboxes").join("demo");
    assert!(project.join("workspace").is_dir());
    let vm_toml = std::fs::read_to_string(project.join("vm.toml")).unwrap();
    assert!(vm_toml.contains("backend = \"microvm\""));
    assert!(!vm_toml.contains("network_policy")); // policy lives in microvm.json
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.join("microvm.json")).unwrap()).unwrap();
    assert_eq!(json["network_policy"], "deny");
    assert_eq!(json["toolchain_mount"], "ro");
    assert!(
        !project.join("overlay.qcow2").exists(),
        "overlay is created at start, never at provision"
    );

    // Start fails honestly: imageless host => missing items listed.
    let resp = dispatch(&router, "sandbox_start", serde_json::json!({ "id": "demo" }));
    assert_eq!(resp.status, StatusCode::InternalError);
    let msg = resp.error.expect("error body").message;
    assert!(msg.contains("missing"), "got: {msg}");
    assert!(msg.contains("base image"), "got: {msg}");
    assert!(msg.contains("guest kernel"), "got: {msg}");

    // Simulate runtime leftovers, then clean through the router: overlay
    // and pid removed, workspace + config kept.
    std::fs::write(project.join("overlay.qcow2"), "qcow2").unwrap();
    std::fs::write(project.join("microvm.pid"), "4242").unwrap();
    let resp = dispatch(&router, "sandbox_clean", serde_json::json!({ "id": "demo" }));
    assert_eq!(resp.status, StatusCode::Ok);
    assert!(!project.join("overlay.qcow2").exists());
    assert!(!project.join("microvm.pid").exists());
    assert!(project.join("workspace").is_dir());
    assert!(project.join("vm.toml").exists());

    // Status after clean.
    let resp = dispatch(&router, "sandbox_status", serde_json::json!({ "id": "demo" }));
    assert_eq!(resp.status, StatusCode::Ok);
    assert_eq!(resp.body["state"], "stopped");
}

#[test]
fn sandbox_list_and_detect_over_the_router() {
    let dir = tempfile::tempdir().unwrap();
    let router = router_with_temp_data_dir(dir.path());

    dispatch(&router, "sandbox_provision", serde_json::json!({ "id": "b" }));
    dispatch(&router, "sandbox_provision", serde_json::json!({ "id": "a" }));

    let resp = dispatch(&router, "sandbox_list", serde_json::json!({}));
    assert_eq!(resp.status, StatusCode::Ok);
    let ids: Vec<&str> = resp.body["sandboxes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["a", "b"]);

    let resp = dispatch(&router, "sandbox_detect", serde_json::json!({}));
    assert_eq!(resp.status, StatusCode::Ok);
    let names: Vec<&str> = resp.body["backends"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"microvm"), "backends: {names:?}");
    assert!(names.contains(&"process"));
}

#[test]
fn sandbox_errors_are_clean_over_the_router() {
    let dir = tempfile::tempdir().unwrap();
    let router = router_with_temp_data_dir(dir.path());

    // Bad id rejected before touching the filesystem.
    let resp = dispatch(&router, "sandbox_provision", serde_json::json!({ "id": "../escape" }));
    assert_eq!(resp.status, StatusCode::InternalError);
    assert!(resp.error.unwrap().message.contains("invalid sandbox id"));

    // Unknown status -> clean NotFound-ish error, not a panic.
    let resp = dispatch(&router, "sandbox_status", serde_json::json!({ "id": "ghost" }));
    assert_eq!(resp.status, StatusCode::InternalError);
    assert!(resp.error.unwrap().message.contains("not found"));
}
