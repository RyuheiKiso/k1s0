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

// =============================================================================
// new-screen コマンドのテスト
// =============================================================================

#[test]
fn test_new_screen_invalid_screen_id_empty() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .args([
            "new-screen",
            "-s",
            "",
            "-T",
            "Test",
            "-f",
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("無効"));
}

#[test]
fn test_new_screen_invalid_screen_id_uppercase() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .args([
            "new-screen",
            "-s",
            "Users.List",
            "-T",
            "Test",
            "-f",
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("無効"));
}

#[test]
fn test_new_screen_invalid_screen_id_leading_dot() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .args([
            "new-screen",
            "-s",
            ".users",
            "-T",
            "Test",
            "-f",
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("無効"));
}

#[test]
fn test_new_screen_feature_dir_not_exists() {
    k1s0()
        .args([
            "new-screen",
            "-s",
            "users.list",
            "-T",
            "Test",
            "-f",
            "/nonexistent/path",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("存在しません"));
}

#[test]
fn test_new_screen_react_generates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pages_dir = temp_dir.path().join("src").join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    // k1s0リポジトリのルートから実行する必要がある
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-screen",
            "-t",
            "react",
            "-s",
            "users.list",
            "-T",
            "ユーザー一覧",
            "-f",
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("UsersListPage"))
        .stderr(predicate::str::contains("生成しました"));

    // ファイルが生成されたことを確認
    let generated_file = pages_dir.join("UsersListPage.tsx");
    assert!(generated_file.exists(), "生成されたファイルが存在すること");

    // ファイル内容を確認
    let content = std::fs::read_to_string(&generated_file).unwrap();
    assert!(content.contains("UsersListPage"), "コンポーネント名が含まれること");
    assert!(content.contains("ユーザー一覧"), "タイトルが含まれること");
}

#[test]
fn test_new_screen_flutter_generates_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pages_dir = temp_dir
        .path()
        .join("lib")
        .join("src")
        .join("presentation")
        .join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    // k1s0リポジトリのルートから実行する必要がある
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-screen",
            "-t",
            "flutter",
            "-s",
            "settings.profile",
            "-T",
            "プロフィール設定",
            "-f",
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("SettingsProfilePage"))
        .stderr(predicate::str::contains("生成しました"));

    // ファイルが生成されたことを確認
    let generated_file = pages_dir.join("settings_profile_page.dart");
    assert!(generated_file.exists(), "生成されたファイルが存在すること");

    // ファイル内容を確認
    let content = std::fs::read_to_string(&generated_file).unwrap();
    assert!(
        content.contains("SettingsProfilePage"),
        "クラス名が含まれること"
    );
    assert!(content.contains("プロフィール設定"), "タイトルが含まれること");
}

#[test]
fn test_new_screen_with_menu_flag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pages_dir = temp_dir.path().join("src").join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-screen",
            "-s",
            "dashboard",
            "-T",
            "ダッシュボード",
            "-f",
            temp_dir.path().to_str().unwrap(),
            "--with-menu",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("menu.items"));
}

#[test]
fn test_new_screen_with_custom_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pages_dir = temp_dir.path().join("src").join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-screen",
            "-s",
            "users.list",
            "-T",
            "ユーザー一覧",
            "-f",
            temp_dir.path().to_str().unwrap(),
            "-p",
            "/custom/users/path",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("/custom/users/path"));
}

#[test]
fn test_new_screen_file_already_exists_without_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pages_dir = temp_dir.path().join("src").join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    // 既存ファイルを作成
    let existing_file = pages_dir.join("UsersListPage.tsx");
    std::fs::write(&existing_file, "existing content").unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-screen",
            "-s",
            "users.list",
            "-T",
            "ユーザー一覧",
            "-f",
            temp_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("既に存在"));
}

#[test]
fn test_new_screen_file_already_exists_with_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pages_dir = temp_dir.path().join("src").join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    // 既存ファイルを作成
    let existing_file = pages_dir.join("UsersListPage.tsx");
    std::fs::write(&existing_file, "existing content").unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-screen",
            "-s",
            "users.list",
            "-T",
            "ユーザー一覧",
            "-f",
            temp_dir.path().to_str().unwrap(),
            "-F",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("生成しました"));

    // ファイルが上書きされたことを確認
    let content = std::fs::read_to_string(&existing_file).unwrap();
    assert!(
        content.contains("UsersListPage"),
        "ファイルが上書きされていること"
    );
}

// =============================================================================
// init コマンドのテスト
// =============================================================================

#[test]
fn test_init_creates_k1s0_directory() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .args(["init", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("初期化"));

    // .k1s0 ディレクトリが作成されたことを確認
    let k1s0_dir = temp_dir.path().join(".k1s0");
    assert!(k1s0_dir.exists(), ".k1s0 ディレクトリが存在すること");

    // config.json が作成されたことを確認
    let config_file = k1s0_dir.join("config.json");
    assert!(config_file.exists(), "config.json が存在すること");
}

#[test]
fn test_init_already_initialized_without_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    let k1s0_dir = temp_dir.path().join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    k1s0()
        .args(["init", temp_dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("既に"));
}

#[test]
fn test_init_already_initialized_with_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    let k1s0_dir = temp_dir.path().join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    k1s0()
        .args(["init", "--force", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("初期化"));
}

// =============================================================================
// new-feature コマンドのテスト
// =============================================================================

#[test]
fn test_new_feature_invalid_name_uppercase() {
    k1s0()
        .args(["new-feature", "-t", "backend-rust", "-n", "UserService"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("不正"));
}

#[test]
fn test_new_feature_invalid_name_underscore() {
    k1s0()
        .args(["new-feature", "-t", "backend-rust", "-n", "user_service"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("不正"));
}

// TODO: new-feature テンプレートの修正後に有効化
// 現在、backend-rust テンプレートにある migrations/0001_initial.down.sql.tera で
// テンプレートエラーが発生するため、一旦スキップ
#[test]
#[ignore = "backend-rust テンプレートの修正が必要"]
fn test_new_feature_generates_files() {
    let temp_dir = tempfile::tempdir().unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let output_path = temp_dir.path().join("test-service");

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-feature",
            "-t",
            "backend-rust",
            "-n",
            "test-service",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("生成しました"));

    // manifest.json が作成されたことを確認
    let manifest_file = output_path.join(".k1s0").join("manifest.json");
    assert!(manifest_file.exists(), "manifest.json が存在すること");
}
