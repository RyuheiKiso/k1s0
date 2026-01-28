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

#[test]
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

#[test]
fn test_new_feature_backend_go_generates_files() {
    let temp_dir = tempfile::tempdir().unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let output_path = temp_dir.path().join("test-go-service");

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-feature",
            "-t",
            "backend-go",
            "-n",
            "test-go-service",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("生成しました"));

    // manifest.json が作成されたことを確認
    let manifest_file = output_path.join(".k1s0").join("manifest.json");
    assert!(manifest_file.exists(), "manifest.json が存在すること");

    // go.mod が作成されたことを確認
    let go_mod = output_path.join("go.mod");
    assert!(go_mod.exists(), "go.mod が存在すること");

    // go.mod の内容を確認
    let content = std::fs::read_to_string(&go_mod).unwrap();
    assert!(
        content.contains("test_go_service"),
        "モジュール名にsnake_caseのサービス名が含まれること"
    );

    // main.go が作成されたことを確認
    let main_go = output_path.join("main.go");
    assert!(main_go.exists(), "main.go が存在すること");

    // ディレクトリ構造の確認
    assert!(
        output_path.join("internal").join("domain").exists(),
        "internal/domain ディレクトリが存在すること"
    );
    assert!(
        output_path.join("internal").join("application").exists(),
        "internal/application ディレクトリが存在すること"
    );
    assert!(
        output_path.join("internal").join("presentation").exists(),
        "internal/presentation ディレクトリが存在すること"
    );
}

#[test]
fn test_new_feature_frontend_flutter_generates_files() {
    let temp_dir = tempfile::tempdir().unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let output_path = temp_dir.path().join("test-flutter-app");

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-feature",
            "-t",
            "frontend-flutter",
            "-n",
            "test-flutter-app",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("生成しました"));

    // manifest.json が作成されたことを確認
    let manifest_file = output_path.join(".k1s0").join("manifest.json");
    assert!(manifest_file.exists(), "manifest.json が存在すること");

    // pubspec.yaml が作成されたことを確認
    let pubspec = output_path.join("pubspec.yaml");
    assert!(pubspec.exists(), "pubspec.yaml が存在すること");

    // lib/main.dart が作成されたことを確認
    let main_dart = output_path.join("lib").join("main.dart");
    assert!(main_dart.exists(), "lib/main.dart が存在すること");

    // ディレクトリ構造の確認
    assert!(
        output_path.join("lib").join("src").join("domain").exists(),
        "lib/src/domain ディレクトリが存在すること"
    );
    assert!(
        output_path
            .join("lib")
            .join("src")
            .join("presentation")
            .exists(),
        "lib/src/presentation ディレクトリが存在すること"
    );
    assert!(
        output_path
            .join("lib")
            .join("src")
            .join("application")
            .exists(),
        "lib/src/application ディレクトリが存在すること"
    );
}

// =============================================================================
// new-domain コマンドのテスト
// =============================================================================

#[test]
fn test_new_domain_help() {
    k1s0()
        .args(["new-domain", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-domain"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--name"));
}

#[test]
fn test_new_domain_invalid_name_uppercase() {
    k1s0()
        .args(["new-domain", "-t", "backend-rust", "-n", "Production"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("kebab-case"));
}

#[test]
fn test_new_domain_invalid_name_underscore() {
    k1s0()
        .args(["new-domain", "-t", "backend-rust", "-n", "user_management"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("kebab-case"));
}

#[test]
fn test_new_domain_reserved_name() {
    k1s0()
        .args(["new-domain", "-t", "backend-rust", "-n", "framework"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("予約語"));
}

#[test]
fn test_new_domain_generates_files() {
    let temp_dir = tempfile::tempdir().unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let output_path = temp_dir.path().join("test-domain");

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-domain",
            "-t",
            "backend-rust",
            "-n",
            "test-domain",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("作成しました"));

    // manifest.json が作成されたことを確認
    let manifest_file = output_path.join(".k1s0").join("manifest.json");
    assert!(manifest_file.exists(), "manifest.json が存在すること");

    // manifest.json の内容を確認
    let content = std::fs::read_to_string(&manifest_file).unwrap();
    assert!(content.contains("\"layer\""), "layer フィールドが存在すること");
    assert!(content.contains("\"domain\""), "layer が domain であること");
    assert!(content.contains("\"version\""), "version フィールドが存在すること");

    // Cargo.toml が作成されたことを確認
    let cargo_toml = output_path.join("Cargo.toml");
    assert!(cargo_toml.exists(), "Cargo.toml が存在すること");

    // ディレクトリ構造の確認
    assert!(
        output_path.join("src").join("domain").exists(),
        "src/domain ディレクトリが存在すること"
    );
    assert!(
        output_path.join("src").join("application").exists(),
        "src/application ディレクトリが存在すること"
    );
    assert!(
        output_path.join("src").join("infrastructure").exists(),
        "src/infrastructure ディレクトリが存在すること"
    );

    // README.md が作成されたことを確認
    let readme = output_path.join("README.md");
    assert!(readme.exists(), "README.md が存在すること");

    // CHANGELOG.md が作成されたことを確認
    let changelog = output_path.join("CHANGELOG.md");
    assert!(changelog.exists(), "CHANGELOG.md が存在すること");
}

#[test]
fn test_new_domain_directory_already_exists_without_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("existing-domain");
    std::fs::create_dir_all(&output_path).unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-domain",
            "-t",
            "backend-rust",
            "-n",
            "existing-domain",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("既に存在"));
}

#[test]
fn test_new_domain_directory_already_exists_with_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("existing-domain");
    std::fs::create_dir_all(&output_path).unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-domain",
            "-t",
            "backend-rust",
            "-n",
            "existing-domain",
            "-o",
            output_path.to_str().unwrap(),
            "-f",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("作成しました"));
}

// =============================================================================
// domain-list コマンドのテスト
// =============================================================================

#[test]
fn test_domain_list_help() {
    k1s0()
        .args(["domain-list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("domain"));
}

#[test]
fn test_domain_list_no_domains() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["domain-list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("見つかりませんでした"));
}

// =============================================================================
// domain-version コマンドのテスト
// =============================================================================

#[test]
fn test_domain_version_help() {
    k1s0()
        .args(["domain-version", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--bump"));
}

#[test]
fn test_domain_version_domain_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["domain-version", "-n", "nonexistent", "-b", "patch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("見つかりません"));
}

#[test]
fn test_domain_version_bump_patch() {
    let temp_dir = tempfile::tempdir().unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // まず domain を作成（親ディレクトリは作成せず、-o で直接パスを指定）
    let domain_path = temp_dir.path().join("domain/backend/rust/test-version");

    k1s0()
        .current_dir(repo_root)
        .args([
            "new-domain",
            "-t",
            "backend-rust",
            "-n",
            "test-version",
            "-o",
            domain_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // バージョンをバンプ
    k1s0()
        .current_dir(temp_dir.path())
        .args(["domain-version", "-n", "test-version", "-b", "patch"])
        .assert()
        .success()
        .stderr(predicate::str::contains("更新しました"));

    // CHANGELOG.md が更新されたことを確認
    let changelog = domain_path.join("CHANGELOG.md");
    let content = std::fs::read_to_string(&changelog).unwrap();
    assert!(
        content.contains("0.1.1"),
        "CHANGELOG.md に新しいバージョンが含まれること"
    );
}

// =============================================================================
// domain-dependents コマンドのテスト
// =============================================================================

#[test]
fn test_domain_dependents_help() {
    k1s0()
        .args(["domain-dependents", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dependents"))
        .stdout(predicate::str::contains("--name"));
}

#[test]
fn test_domain_dependents_domain_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["domain-dependents", "-n", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("見つかりません"));
}

// =============================================================================
// domain-impact コマンドのテスト
// =============================================================================

#[test]
fn test_domain_impact_help() {
    k1s0()
        .args(["domain-impact", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("impact"))
        .stdout(predicate::str::contains("--name"));
}

#[test]
fn test_domain_impact_domain_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["domain-impact", "-n", "nonexistent", "--to", "1.0.0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("見つかりません"));
}

// =============================================================================
// 対話モード関連テスト
// =============================================================================

#[test]
fn test_new_feature_interactive_flag_exists() {
    // --interactive / -i フラグがヘルプに存在することを確認
    k1s0()
        .args(["new-feature", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--interactive"))
        .stdout(predicate::str::contains("-i"));
}

#[test]
fn test_new_feature_missing_type_no_tty_fails() {
    // 非対話環境で必須引数（type）不足時にエラー
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["new-feature", "--name", "test-service"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("必須引数が不足"));
}

#[test]
fn test_new_feature_missing_name_no_tty_fails() {
    // 非対話環境で必須引数（name）不足時にエラー
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["new-feature", "--type", "backend-rust"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("必須引数が不足"));
}

#[test]
fn test_new_feature_all_args_provided_succeeds() {
    // 全ての必須引数が提供されている場合は成功
    let temp_dir = tempfile::tempdir().unwrap();

    // k1s0 リポジトリのルートを基準に実行
    k1s0()
        .args([
            "new-feature",
            "--type", "backend-rust",
            "--name", "test-interactive",
            "--output", temp_dir.path().join("test-output").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("サービス 'test-interactive' を生成しました"));
}

#[test]
fn test_new_feature_interactive_flag_no_tty_fails() {
    // --interactive フラグが指定されているが TTY がない場合はエラー
    let temp_dir = tempfile::tempdir().unwrap();

    k1s0()
        .current_dir(temp_dir.path())
        .args(["new-feature", "--interactive"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("TTY"));
}
