//! テストモジュール — `unwrap()` の使用を許可する
#![allow(clippy::unwrap_used)]
/// テンプレートファイルのコンテンツスナップショットテスト（L-05 監査対応）。
///
/// `snapshot_tests.rs` は生成ファイル一覧（パス）の変化のみを検知するが、
/// テンプレート内容（コードパターン・依存関係・設定値）の変化は検知できない。
/// 本テストはそのギャップを埋める：主要ファイルの実際の生成内容を `insta`
/// スナップショットとして保存し、テンプレート変更時の意図しないリグレッションを
/// 自動で検知する。
///
/// # カバレッジ
/// - Rust REST サーバー（最小): Cargo.toml / src/main.rs / src/adapter/handler/rest.rs
/// - Go REST サーバー（最小）: go.mod / cmd/main.go / internal/adapter/handler/rest_handler.go
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

// =========================================================================
// ヘルパー関数
// =========================================================================

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

/// テンプレートをレンダリングし、`target_files` に指定したファイルの内容を返す。
///
/// 返すマップのキーはリポジトリルート相対パス（スラッシュ区切り）。
/// ファイルが生成されなかった場合はマップに含まれない。
fn render_and_collect(
    lang: &str,
    kind: &str,
    api_style: &str,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
    target_files: &[&str],
) -> (TempDir, BTreeMap<String, String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder =
        TemplateContextBuilder::new("task-api", "service", lang, kind).api_style(api_style);

    if has_database {
        builder = builder.with_database(database_type);
    }
    if has_kafka {
        builder = builder.with_kafka();
    }
    if has_redis {
        builder = builder.with_redis();
    }

    let ctx = builder.try_build().unwrap();
    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    engine.render_to_dir(&ctx, &output_dir).unwrap();

    let mut contents = BTreeMap::new();
    for &file in target_files {
        // OS のパス区切りに変換して読み込む
        let rel: std::path::PathBuf = file.split('/').collect();
        let path = output_dir.join(&rel);
        if path.exists() {
            let raw = fs::read_to_string(&path).unwrap();
            // Windows の CRLF を LF に正規化してスナップショットの差分を防ぐ
            contents.insert(file.to_string(), raw.replace("\r\n", "\n"));
        }
    }

    (tmp, contents)
}

// =========================================================================
// Rust REST サーバー — キーファイル内容検証
// =========================================================================

/// Rust REST サーバー（最小）— Cargo.toml の内容が意図通りに生成されることを検証する。
///
/// 依存関係の追加・削除・バージョン変更はここで検知される。
#[test]
fn test_content_rust_rest_minimal_cargo_toml() {
    let (_tmp, contents) = render_and_collect(
        "rust",
        "server",
        "rest",
        false,
        "",
        false,
        false,
        &["Cargo.toml"],
    );
    let content = contents.get("Cargo.toml").expect("Cargo.toml not generated");
    insta::assert_snapshot!("rust_rest_minimal__cargo_toml", content);
}

/// Rust REST サーバー（最小）— src/main.rs の内容が意図通りに生成されることを検証する。
///
/// エントリーポイントのブートストラップパターン変更はここで検知される。
#[test]
fn test_content_rust_rest_minimal_main_rs() {
    let (_tmp, contents) = render_and_collect(
        "rust",
        "server",
        "rest",
        false,
        "",
        false,
        false,
        &["src/main.rs"],
    );
    let content = contents.get("src/main.rs").expect("src/main.rs not generated");
    insta::assert_snapshot!("rust_rest_minimal__main_rs", content);
}

/// Rust REST サーバー（最小）— src/adapter/handler/rest.rs の内容を検証する。
///
/// REST ハンドラーのコードパターン変更はここで検知される。
#[test]
fn test_content_rust_rest_minimal_handler_rs() {
    let (_tmp, contents) = render_and_collect(
        "rust",
        "server",
        "rest",
        false,
        "",
        false,
        false,
        &["src/adapter/handler/rest.rs"],
    );
    let content = contents
        .get("src/adapter/handler/rest.rs")
        .expect("src/adapter/handler/rest.rs not generated");
    insta::assert_snapshot!("rust_rest_minimal__handler_rest_rs", content);
}

/// Rust REST サーバー（DB + Kafka）— Cargo.toml の内容を検証する。
///
/// DB/Kafka 有効時の依存関係が正しく含まれることを検証する。
#[test]
fn test_content_rust_rest_db_kafka_cargo_toml() {
    let (_tmp, contents) = render_and_collect(
        "rust",
        "server",
        "rest",
        true,
        "postgresql",
        true,
        false,
        &["Cargo.toml"],
    );
    let content = contents.get("Cargo.toml").expect("Cargo.toml not generated");
    insta::assert_snapshot!("rust_rest_db_kafka__cargo_toml", content);
}

// =========================================================================
// Go REST サーバー — キーファイル内容検証
// =========================================================================

/// Go REST サーバー（最小）— go.mod の内容が意図通りに生成されることを検証する。
///
/// Go モジュール依存関係の変更はここで検知される。
#[test]
fn test_content_go_rest_minimal_go_mod() {
    let (_tmp, contents) = render_and_collect(
        "go",
        "server",
        "rest",
        false,
        "",
        false,
        false,
        &["go.mod"],
    );
    let content = contents.get("go.mod").expect("go.mod not generated");
    insta::assert_snapshot!("go_rest_minimal__go_mod", content);
}

/// Go REST サーバー（最小）— cmd/main.go の内容を検証する。
///
/// Go エントリーポイントのブートストラップパターン変更はここで検知される。
#[test]
fn test_content_go_rest_minimal_main_go() {
    let (_tmp, contents) = render_and_collect(
        "go",
        "server",
        "rest",
        false,
        "",
        false,
        false,
        &["cmd/main.go"],
    );
    let content = contents.get("cmd/main.go").expect("cmd/main.go not generated");
    insta::assert_snapshot!("go_rest_minimal__main_go", content);
}

/// Go REST サーバー（最小）— internal/adapter/handler/rest_handler.go の内容を検証する。
///
/// REST ハンドラーのコードパターン変更はここで検知される。
#[test]
fn test_content_go_rest_minimal_handler_go() {
    let (_tmp, contents) = render_and_collect(
        "go",
        "server",
        "rest",
        false,
        "",
        false,
        false,
        &["internal/adapter/handler/rest_handler.go"],
    );
    let content = contents
        .get("internal/adapter/handler/rest_handler.go")
        .expect("internal/adapter/handler/rest_handler.go not generated");
    insta::assert_snapshot!("go_rest_minimal__rest_handler_go", content);
}

/// Go REST サーバー（DB + Kafka）— internal/infra/config/config.go の内容を検証する。
///
/// DB/Kafka 有効時の設定構造体が正しく含まれることを検証する。
#[test]
fn test_content_go_rest_db_kafka_config_go() {
    let (_tmp, contents) = render_and_collect(
        "go",
        "server",
        "rest",
        true,
        "postgresql",
        true,
        false,
        &["internal/infra/config/config.go"],
    );
    let content = contents
        .get("internal/infra/config/config.go")
        .expect("internal/infra/config/config.go not generated");
    insta::assert_snapshot!("go_rest_db_kafka__config_go", content);
}
