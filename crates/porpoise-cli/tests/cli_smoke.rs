use assert_cmd::Command;
use predicates::prelude::*;

// Helper: build a `porpoise` command with a temp HOME so daemon
// doesn't collide with a running instance.
fn porpoise_cmd() -> Command {
    let mut cmd = Command::cargo_bin("porpoise").expect("binary exists");
    // Ensure no real daemon is contacted
    cmd.env("PORPOISE_HOME", tempfile::tempdir().unwrap().path());
    cmd
}

#[test]
fn version_flag_prints_version() {
    porpoise_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("porpoise"));
}

#[test]
fn help_flag_prints_usage() {
    porpoise_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("porpoise"));
}

#[test]
fn status_subcommand_returns_json() {
    porpoise_cmd()
        .args(["--json", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("running"));
}

#[test]
fn invalid_subcommand_fails() {
    porpoise_cmd()
        .arg("nonexistent-command")
        .assert()
        .failure();
}

#[test]
fn worktree_list_without_daemon_fails_gracefully() {
    // Without a running daemon, worktree list should error (not panic)
    porpoise_cmd()
        .args(["worktree", "list"])
        .assert()
        .failure();
}

#[test]
fn agent_list_without_daemon_fails_gracefully() {
    porpoise_cmd()
        .args(["agent", "list"])
        .assert()
        .failure();
}
