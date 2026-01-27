//! CLI 統合テスト
//!
//! k1s0 CLI の動作を検証する統合テスト

use assert_cmd::Command;
use predicates::prelude::*;

/// k1s0 コマンドを取得
#[allow(deprecated)]
fn k1s0() -> Command {
    Command::cargo_bin("k1s0").unwrap()
}

#[test]
fn test_version_flag() {
    k1s0()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("k1s0"));
}

#[test]
fn test_help_flag() {
    k1s0()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("k1s0"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_init_help() {
    k1s0()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"));
}

#[test]
fn test_lint_help() {
    k1s0()
        .args(["lint", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lint"));
}

#[test]
fn test_upgrade_help() {
    k1s0()
        .args(["upgrade", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("upgrade"));
}

#[test]
fn test_new_feature_help() {
    k1s0()
        .args(["new-feature", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-feature"));
}

#[test]
fn test_new_screen_help() {
    k1s0()
        .args(["new-screen", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-screen"));
}

#[test]
fn test_registry_help() {
    k1s0()
        .args(["registry", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("registry"));
}

#[test]
fn test_completions_help() {
    k1s0()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn test_lint_without_manifest() {
    // manifest.json が存在しないディレクトリで lint を実行
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .arg("lint")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("K001").or(predicate::str::contains("lint")));
}

#[test]
fn test_invalid_subcommand() {
    k1s0()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_completions_bash() {
    k1s0()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_completions_zsh() {
    k1s0()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef"));
}

#[test]
fn test_completions_fish() {
    k1s0()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_completions_powershell() {
    // clap_complete では "power-shell" という名前で登録されている
    k1s0()
        .args(["completions", "power-shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}
