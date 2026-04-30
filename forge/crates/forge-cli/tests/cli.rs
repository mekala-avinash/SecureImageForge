//! End-to-end CLI integration tests. They exercise everything that does not
//! require the external buildkit/trivy/syft/cosign/opa daemons — i.e. the
//! parts of the CLI that talk only to local state.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn forge(data_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("forge").unwrap();
    cmd.env("HOME", data_dir.path())
        .env("FORGE_DATA_DIR", data_dir.path().join("data"));
    cmd
}

#[test]
fn help_prints_subcommands() {
    let mut cmd = Command::cargo_bin("forge").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("logs"));
}

#[test]
fn version_prints() {
    let mut cmd = Command::cargo_bin("forge").unwrap();
    cmd.arg("--version").assert().success();
}

#[test]
fn list_on_empty_db_succeeds() {
    let dir = TempDir::new().unwrap();
    forge(&dir).arg("list").assert().success();
}

#[test]
fn stats_on_empty_db_shows_zero_total() {
    let dir = TempDir::new().unwrap();
    forge(&dir)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("total:     0"));
}

#[test]
fn stats_json_output_shape() {
    let dir = TempDir::new().unwrap();
    forge(&dir)
        .args(["--output", "json", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\""))
        .stdout(predicate::str::contains("\"succeeded\""));
}

#[test]
fn scan_for_invalid_uuid_errors_clearly() {
    let dir = TempDir::new().unwrap();
    forge(&dir)
        .args(["scan", "not-a-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid build id"));
}

#[test]
fn logs_for_missing_build_prints_friendly_message() {
    let dir = TempDir::new().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    forge(&dir)
        .args(["logs", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("no log file"));
}

#[test]
fn completions_bash_emits_script() {
    let mut cmd = Command::cargo_bin("forge").unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_forge()"));
}

#[test]
fn doctor_prints_resolution_summary() {
    let dir = TempDir::new().unwrap();
    forge(&dir)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("vendor prefix"));
}
