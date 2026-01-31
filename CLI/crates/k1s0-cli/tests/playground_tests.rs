//! Playground コマンドの統合テスト
//!
//! テンプレートレンダリングの正確性、マルチパスオーバーレイのマージ、
//! ローカル/Docker モードの分岐を検証する。

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// k1s0 CLI コマンドを取得する
#[allow(deprecated)]
fn k1s0() -> Command {
    Command::cargo_bin("k1s0").unwrap()
}

/// テスト用の Tera コンテキストを構築する
fn build_context(
    name: &str,
    with_db: bool,
    with_grpc: bool,
    with_cache: bool,
    port_offset: u16,
    mode: &str,
) -> k1s0_generator::Context {
    let feature_name_snake = name.replace('-', "_");
    let feature_name_pascal = name
        .split('-')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<String>();

    let mut ctx = k1s0_generator::Context::new();
    ctx.insert("feature_name", name);
    ctx.insert("service_name", name);
    ctx.insert("feature_name_snake", &feature_name_snake);
    ctx.insert("feature_name_pascal", &feature_name_pascal);
    ctx.insert("with_grpc", &with_grpc);
    ctx.insert("with_rest", &true);
    ctx.insert("with_db", &with_db);
    ctx.insert("with_cache", &with_cache);
    ctx.insert("rest_port", &(8080u16 + port_offset));
    ctx.insert("grpc_port", &(50051u16 + port_offset));
    ctx.insert("db_port", &(5432u16 + port_offset));
    ctx.insert("redis_port", &(6379u16 + port_offset));
    ctx.insert("is_playground", &true);
    ctx.insert("mode", mode);
    // Variables required by base feature templates (Pass 1)
    ctx.insert("now", "2026-01-31T00:00:00Z");
    ctx.insert("docker_context_levels", ".");
    ctx.insert("feature_relative_path", ".");
    ctx.insert("feature_name_kebab", name);
    ctx.insert("domain_name", "");
    ctx.insert("domain_name_snake", "");
    ctx.insert("has_domain", &false);
    // feature_name_title: "My Cool App" style
    let feature_name_title = name
        .split('-')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    ctx.insert("feature_name_title", &feature_name_title);
    ctx
}

/// CLI/templates/ ディレクトリのパスを返す
fn templates_dir() -> PathBuf {
    // テスト実行時のカレントディレクトリは CLI/crates/k1s0-cli/ なので上に辿る
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent() // crates/
        .unwrap()
        .parent() // CLI/
        .unwrap()
        .join("templates")
}

/// 単一パスのレンダリングを実行する（playground.rs の render_single_pass と同等）
fn render_pass(template_dir: &Path, output_dir: &Path, context: &k1s0_generator::Context) {
    if !template_dir.is_dir() {
        return;
    }
    let renderer =
        k1s0_generator::template::TemplateRenderer::new(template_dir).expect("renderer creation");
    renderer
        .render_directory(output_dir, context)
        .expect("render_directory");
}

/// Docker モードのマルチパスレンダリング（3 段階: base -> playground -> common）
fn render_docker_mode(
    output_dir: &Path,
    template_type: &str,
    context: &k1s0_generator::Context,
) {
    let base = templates_dir();

    // Pass 1: ベーステンプレート
    render_pass(&base.join(template_type).join("feature"), output_dir, context);

    // Pass 2: playground オーバーレイ
    render_pass(&base.join("playground").join(template_type), output_dir, context);

    // Pass 3: 共通オーバーレイ
    render_pass(&base.join("playground").join("common"), output_dir, context);
}

/// Local モードのマルチパスレンダリング（4 段階: base -> playground -> local -> common）
fn render_local_mode(
    output_dir: &Path,
    template_type: &str,
    context: &k1s0_generator::Context,
) {
    let base = templates_dir();

    // Pass 1: ベーステンプレート
    render_pass(&base.join(template_type).join("feature"), output_dir, context);

    // Pass 2: playground オーバーレイ
    render_pass(&base.join("playground").join(template_type), output_dir, context);

    // Pass 3: ローカルオーバーレイ
    render_pass(
        &base.join("playground").join(format!("{template_type}-local")),
        output_dir,
        context,
    );

    // Pass 4: 共通オーバーレイ
    render_pass(&base.join("playground").join("common"), output_dir, context);
}

/// ファイルの内容を読み取る
fn read_file(dir: &Path, relative: &str) -> String {
    let path = dir.join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// ファイルが存在することをアサートする
fn assert_file_exists(dir: &Path, relative: &str) {
    let path = dir.join(relative);
    assert!(path.exists(), "expected file to exist: {}", path.display());
}

/// ディレクトリが存在することをアサートする
fn assert_dir_exists(dir: &Path, relative: &str) {
    let path = dir.join(relative);
    assert!(path.is_dir(), "expected directory to exist: {}", path.display());
}

// ===========================================================================
// CLI ヘルプ テスト
// ===========================================================================

#[test]
fn test_playground_help() {
    k1s0()
        .args(["playground", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("playground"));
}

#[test]
fn test_playground_start_help() {
    k1s0()
        .args(["playground", "start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--type"));
}

#[test]
fn test_playground_list_help() {
    k1s0()
        .args(["playground", "list", "--help"])
        .assert()
        .success();
}

#[test]
fn test_playground_status_help() {
    k1s0()
        .args(["playground", "status", "--help"])
        .assert()
        .success();
}

#[test]
fn test_playground_stop_help() {
    k1s0()
        .args(["playground", "stop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"));
}

// ===========================================================================
// backend-rust Docker モード テンプレートレンダリング テスト
// ===========================================================================

#[test]
fn test_backend_rust_docker_mode_with_db_true_grpc_true() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("my-app", true, true, false, 0, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    // 3-stage merge: base -> playground -> common の結果を検証

    // playground オーバーレイで上書きされた Cargo.toml
    let cargo_toml = read_file(out, "Cargo.toml");
    assert!(
        cargo_toml.contains("name = \"my-app\""),
        "Cargo.toml should contain feature_name"
    );
    assert!(
        cargo_toml.contains("sqlx"),
        "with_db=true: Cargo.toml should contain sqlx"
    );

    // docker-compose.yml: ポート反映と DB/gRPC 分岐
    let compose = read_file(out, "docker-compose.yml");
    assert!(
        compose.contains("8080:8080"),
        "docker-compose should map REST port"
    );
    assert!(
        compose.contains("50051:50051"),
        "with_grpc=true: docker-compose should map gRPC port"
    );
    assert!(
        compose.contains("postgres:16-alpine"),
        "with_db=true: docker-compose should include postgres service"
    );

    // config/default.yaml: Docker モードでは postgres 設定
    let config = read_file(out, "config/default.yaml");
    assert!(
        config.contains("driver: postgres"),
        "Docker mode should use postgres driver"
    );

    // 共通オーバーレイの PLAYGROUND_README.md
    assert_file_exists(out, "PLAYGROUND_README.md");
    let readme = read_file(out, "PLAYGROUND_README.md");
    assert!(readme.contains("my-app"), "README should contain feature_name");
    assert!(readme.contains("docker"), "README should contain mode");

    // ディレクトリ構造
    assert_dir_exists(out, "src");
    assert_dir_exists(out, "src/domain");
    assert_dir_exists(out, "src/application");
    assert_dir_exists(out, "src/presentation");
    assert_dir_exists(out, "src/infrastructure");
    assert_dir_exists(out, "config");
}

#[test]
fn test_backend_rust_docker_mode_with_db_false_grpc_false() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("test-svc", false, false, false, 0, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    let cargo_toml = read_file(out, "Cargo.toml");
    assert!(
        !cargo_toml.contains("sqlx"),
        "with_db=false: Cargo.toml should NOT contain sqlx"
    );

    let compose = read_file(out, "docker-compose.yml");
    assert!(
        !compose.contains("50051"),
        "with_grpc=false: docker-compose should NOT map gRPC port"
    );
    assert!(
        !compose.contains("postgres"),
        "with_db=false: docker-compose should NOT include postgres"
    );

    let config = read_file(out, "config/default.yaml");
    assert!(
        !config.contains("database"),
        "with_db=false: config should NOT contain database section"
    );
}

#[test]
fn test_backend_rust_docker_mode_port_offset() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let offset: u16 = 100;
    let ctx = build_context("offset-app", true, true, true, offset, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    let compose = read_file(out, "docker-compose.yml");
    assert!(
        compose.contains("8180:8080"),
        "port offset 100: REST should map 8180:8080"
    );
    assert!(
        compose.contains("50151:50051"),
        "port offset 100: gRPC should map 50151:50051"
    );
    assert!(
        compose.contains("5532:5432"),
        "port offset 100: DB should map 5532:5432"
    );
    assert!(
        compose.contains("6479:6379"),
        "port offset 100: Redis should map 6479:6379"
    );
}

#[test]
fn test_backend_rust_docker_mode_with_cache() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("cache-app", false, false, true, 0, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    let compose = read_file(out, "docker-compose.yml");
    assert!(
        compose.contains("redis:7-alpine"),
        "with_cache=true: docker-compose should include redis service"
    );
    assert!(
        compose.contains("6379:6379"),
        "with_cache=true: docker-compose should map redis port"
    );
}

// ===========================================================================
// backend-rust Local モード テンプレートレンダリング テスト
// ===========================================================================

#[test]
fn test_backend_rust_local_mode_4_stage_merge() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("local-app", true, false, false, 0, "local");

    render_local_mode(out, "backend-rust", &ctx);

    // Pass 3 (local overlay) が config/default.yaml を上書きしている
    let config = read_file(out, "config/default.yaml");
    assert!(
        config.contains("driver: sqlite"),
        "Local mode should use sqlite driver"
    );
    assert!(
        config.contains("./data/playground.db"),
        "Local mode should reference sqlite file path"
    );
    assert!(
        !config.contains("driver: postgres"),
        "Local mode should NOT use postgres driver"
    );

    // REST ポートが Tera 変数で展開されている
    assert!(
        config.contains("port: 8080"),
        "Local mode config should contain the rendered port"
    );
}

#[test]
fn test_backend_rust_local_mode_with_db_false() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("local-nodb", false, false, false, 0, "local");

    render_local_mode(out, "backend-rust", &ctx);

    let config = read_file(out, "config/default.yaml");
    assert!(
        !config.contains("database"),
        "with_db=false in local mode should NOT contain database section"
    );
    assert!(
        !config.contains("sqlite"),
        "with_db=false in local mode should NOT contain sqlite"
    );
}

#[test]
fn test_backend_rust_local_mode_port_offset_in_config() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("local-offset", true, false, false, 50, "local");

    render_local_mode(out, "backend-rust", &ctx);

    let config = read_file(out, "config/default.yaml");
    assert!(
        config.contains("port: 8130"),
        "Local mode config should reflect port offset (8080+50=8130)"
    );
}

// ===========================================================================
// Local モード テンプレート生成 -> ディレクトリ構造の検証 (統合テスト)
// ===========================================================================

#[test]
fn test_local_mode_directory_structure_backend_rust() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("struct-test", true, true, true, 0, "local");

    render_local_mode(out, "backend-rust", &ctx);

    // ファイルの存在チェック
    assert_file_exists(out, "Cargo.toml");
    assert_file_exists(out, "config/default.yaml");
    assert_file_exists(out, "PLAYGROUND_README.md");
    assert_file_exists(out, "src/main.rs");
    assert_file_exists(out, "src/domain/entities/item.rs");
    assert_file_exists(out, "src/application/usecases/item_usecase.rs");
    assert_file_exists(out, "src/infrastructure/repositories/item_repository.rs");
    assert_file_exists(out, "src/presentation/rest/mod.rs");

    // ディレクトリの存在チェック
    assert_dir_exists(out, "src/domain");
    assert_dir_exists(out, "src/domain/entities");
    assert_dir_exists(out, "src/application");
    assert_dir_exists(out, "src/application/usecases");
    assert_dir_exists(out, "src/infrastructure");
    assert_dir_exists(out, "src/infrastructure/repositories");
    assert_dir_exists(out, "src/presentation");
    assert_dir_exists(out, "src/presentation/rest");
    assert_dir_exists(out, "config");
}

#[test]
fn test_local_mode_directory_structure_backend_go() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("go-test", false, false, false, 0, "local");

    // Go playground テンプレートが存在する場合のみレンダリング
    let base = templates_dir();
    let go_playground = base.join("playground").join("backend-go");
    if !go_playground.is_dir() {
        // テンプレートが存在しない場合はスキップ
        return;
    }

    render_local_mode(out, "backend-go", &ctx);

    assert_file_exists(out, "go.mod");
    assert_file_exists(out, "config/default.yaml");
    assert_file_exists(out, "PLAYGROUND_README.md");
}

// ===========================================================================
// Overlay マージの正確性テスト
// ===========================================================================

#[test]
fn test_overlay_merge_later_pass_overwrites_earlier() {
    // Docker モード: playground overlay (pass 2) がベース (pass 1) を上書きすることを確認
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("merge-test", true, false, false, 0, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    // playground overlay の Cargo.toml にはサンプルコード用の依存が含まれている
    let cargo_toml = read_file(out, "Cargo.toml");
    // playground overlay は chrono や tower-http などの追加依存を持つ
    assert!(
        cargo_toml.contains("tower-http"),
        "Playground overlay should add tower-http dependency"
    );
}

#[test]
fn test_common_overlay_adds_readme() {
    // 共通オーバーレイが全モードで PLAYGROUND_README.md を追加することを確認
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("readme-test", false, false, false, 0, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    assert_file_exists(out, "PLAYGROUND_README.md");
    let readme = read_file(out, "PLAYGROUND_README.md");
    assert!(readme.contains("readme-test"));
}

#[test]
fn test_common_overlay_adds_seed_sql() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("seed-test", true, false, false, 0, "docker");

    render_docker_mode(out, "backend-rust", &ctx);

    assert_file_exists(out, "seed.sql");
}

// ===========================================================================
// feature_name 変換テスト
// ===========================================================================

#[test]
fn test_feature_name_pascal_case_in_context() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("my-cool-app", false, false, false, 0, "docker");

    // backend-csharp テンプレートが存在する場合、Pascal case を使用する
    let base = templates_dir();
    let csharp_playground = base.join("playground").join("backend-csharp");
    if !csharp_playground.is_dir() {
        return;
    }

    render_docker_mode(out, "backend-csharp", &ctx);

    // C# テンプレートは Tera 変数をディレクトリ名に使用する。
    // TemplateRenderer はディレクトリ名のテンプレート展開を行わないため、
    // リテラル "{{ feature_name_pascal }}" が含まれるディレクトリが生成される。
    // ここでは C# 用のテンプレートファイルの中身に PascalCase が反映されることを確認する。
    let item_cs = out
        .join("src")
        .join("{{ feature_name_pascal }}.Domain")
        .join("Entities")
        .join("Item.cs");
    if item_cs.exists() {
        let content = fs::read_to_string(&item_cs).unwrap();
        assert!(
            content.contains("MyCoolApp"),
            "C# Item.cs namespace should contain PascalCase feature name"
        );
    }
}

// ===========================================================================
// Docker テスト (feature-gated, 通常は ignore)
// ===========================================================================

#[cfg_attr(not(feature = "docker-tests"), ignore)]
#[test]
fn test_docker_mode_lifecycle_start_status_stop() {
    // Docker が必要なため、通常のテストではスキップされる。
    // `cargo test --features docker-tests` で実行可能。
    //
    // このテストは以下のライフサイクルを検証する:
    //   1. playground start --type backend-rust --mode docker --yes
    //   2. playground status --json
    //   3. playground stop --yes --volumes
    //
    // TODO: Docker 環境が CI で利用可能になったら実装を完成させる。

    // スタブ: Docker が利用可能であることだけ確認
    let output = std::process::Command::new("docker")
        .arg("--version")
        .output();
    assert!(
        output.is_ok_and(|o| o.status.success()),
        "Docker must be available for this test"
    );
}

// ===========================================================================
// backend-python テンプレートレンダリングテスト
// ===========================================================================

#[test]
fn test_backend_python_docker_mode_rendering() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("py-app", true, false, false, 0, "docker");

    let base = templates_dir();
    if !base.join("playground").join("backend-python").is_dir() {
        return;
    }

    render_docker_mode(out, "backend-python", &ctx);

    assert_file_exists(out, "pyproject.toml");
    assert_file_exists(out, "config/default.yaml");
    assert_file_exists(out, "PLAYGROUND_README.md");

    let pyproject = read_file(out, "pyproject.toml");
    assert!(pyproject.contains("py-app") || pyproject.contains("py_app"));
}

#[test]
fn test_backend_python_local_mode_sqlite_config() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path();
    let ctx = build_context("py-local", true, false, false, 0, "local");

    let base = templates_dir();
    if !base.join("playground").join("backend-python-local").is_dir() {
        return;
    }

    render_local_mode(out, "backend-python", &ctx);

    let config = read_file(out, "config/default.yaml");
    assert!(
        config.contains("sqlite"),
        "Python local mode should use sqlite"
    );
}
