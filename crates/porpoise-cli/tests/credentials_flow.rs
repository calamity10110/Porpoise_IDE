//! Credential flow integration test: drives the real `porpoise` binary
//! through a full store/retrieve/update/list/delete cycle against an
//! isolated vault, plus the failure paths (missing master key, wrong key,
//! unknown name).

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn porpoise(data_dir: &TempDir, master_key: &str) -> Command {
    let mut cmd = Command::cargo_bin("porpoise").expect("porpoise binary");
    cmd.env("PORPOISE_DATA_DIR", data_dir.path())
        .env("PORPOISE_MASTER_KEY", master_key);
    cmd
}

#[test]
fn credential_set_get_update_list_delete_roundtrip() {
    let dir = TempDir::new().unwrap();

    // Store.
    porpoise(&dir, "correct-horse")
        .args(["credential", "set", "github-token", "--value", "ghp_secret123"])
        .assert()
        .success()
        .stdout(predicates::str::contains("github-token"));

    // Retrieve: the exact secret comes back.
    porpoise(&dir, "correct-horse")
        .args(["credential", "get", "github-token"])
        .assert()
        .success()
        .stdout(predicates::str::contains("ghp_secret123"));

    // Update (upsert): no duplicate entries left behind.
    porpoise(&dir, "correct-horse")
        .args(["credential", "set", "github-token", "--value", "ghp_rotated456"])
        .assert()
        .success();
    porpoise(&dir, "correct-horse")
        .args(["credential", "get", "github-token"])
        .assert()
        .success()
        .stdout(predicates::str::contains("ghp_rotated456"));

    // List shows name + type but NEVER the secret.
    porpoise(&dir, "correct-horse")
        .args(["credential", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("github-token").and(predicates::str::contains("ghp").not()));

    // Delete, then get must fail with a clean error (exit 1).
    porpoise(&dir, "correct-horse")
        .args(["credential", "delete", "github-token"])
        .assert()
        .success()
        .stdout(predicates::str::contains("deleted"));
    porpoise(&dir, "correct-horse")
        .args(["credential", "get", "github-token"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not found"));
}

#[test]
fn missing_master_key_is_refused_honestly() {
    let dir = TempDir::new().unwrap();
    porpoise(&dir, "")
        .args(["credential", "set", "x", "--value", "y"])
        .env_remove("PORPOISE_MASTER_KEY")
        .assert()
        .failure()
        .stderr(predicates::str::contains("PORPOISE_MASTER_KEY"));
}

#[test]
fn wrong_master_key_cannot_decrypt() {
    let dir = TempDir::new().unwrap();
    porpoise(&dir, "right-key")
        .args(["credential", "set", "svc", "--value", "s3cret"])
        .assert()
        .success();

    // A different key opens the vault but must fail to decrypt, cleanly.
    porpoise(&dir, "wrong-key")
        .args(["credential", "get", "svc"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("decrypt"));
}

#[test]
fn vault_file_never_stores_plaintext() {
    let dir = TempDir::new().unwrap();
    let secret = "sk-plaintext-probe-1234567890";
    porpoise(&dir, "some-key")
        .args(["credential", "set", "probe", "--value", secret])
        .assert()
        .success();

    let vault = std::fs::read_to_string(dir.path().join("credentials.json")).unwrap();
    assert!(
        !vault.contains(secret),
        "the vault file must never contain the plaintext secret"
    );
}
