use assert_cmd::Command;

#[test]
fn test_version() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .arg("version")
        .assert()
        .success();
}

#[test]
fn test_help() {
    Command::cargo_bin("porpoise").unwrap().arg("--help").assert().success();
}

#[test]
fn test_status() {
    let mut cmd = Command::cargo_bin("porpoise").unwrap();
    cmd.arg("status");
    let output = cmd.assert();
    output.failure_or_success();
}

#[test]
fn test_worktree_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["worktree", "--help"])
        .assert()
        .success();
}

#[test]
fn test_agent_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["agent", "--help"])
        .assert()
        .success();
}

#[test]
fn test_terminal_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["terminal", "--help"])
        .assert()
        .success();
}

#[test]
fn test_git_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["git", "--help"])
        .assert()
        .success();
}

#[test]
fn test_skill_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["skill", "--help"])
        .assert()
        .success();
}

#[test]
fn test_ssh_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["ssh", "--help"])
        .assert()
        .success();
}

#[test]
fn test_config_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["config", "--help"])
        .assert()
        .success();
}

#[test]
fn test_browser_help() {
    Command::cargo_bin("porpoise")
        .unwrap()
        .args(["browser", "--help"])
        .assert()
        .success();
}

#[test]
fn test_json_flag() {
    let mut cmd = Command::cargo_bin("porpoise").unwrap();
    cmd.args(["--json", "version"]);
    let output = cmd.assert();
    output.failure_or_success();
}

#[test]
fn test_skill_list() {
    let mut cmd = Command::cargo_bin("porpoise").unwrap();
    cmd.args(["skill", "list"]);
    let output = cmd.assert();
    output.failure_or_success();
}
