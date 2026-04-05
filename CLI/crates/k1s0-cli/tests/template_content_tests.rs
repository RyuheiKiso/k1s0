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
/// - Go REST サーバー（最小）: go.mod / cmd/main.go / `internal/adapter/handler/rest_handler.go`
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

/// `render_and_collect` に渡すテンプレートレンダリングオプション。
///
/// 引数が多くなるのを防ぐために関連するパラメータをまとめた構造体。
struct RenderOptions<'a> {
    /// 生成対象言語（例: "rust", "go"）
    lang: &'a str,
    /// コンポーネント種別（例: "server", "library"）
    kind: &'a str,
    /// API スタイル（例: "rest", "grpc"）
    api_style: &'a str,
    /// データベースを有効化するかどうか
    has_database: bool,
    /// データベース種別（例: "postgresql"）
    database_type: &'a str,
    /// Kafka を有効化するかどうか
    has_kafka: bool,
    /// Redis を有効化するかどうか
    has_redis: bool,
}

/// テンプレートをレンダリングし、`target_files` に指定したファイルの内容を返す。
///
/// 返すマップのキーはリポジトリルート相対パス（スラッシュ区切り）。
/// ファイルが生成されなかった場合はマップに含まれない。
fn render_and_collect(
    opts: &RenderOptions<'_>,
    target_files: &[&str],
) -> (TempDir, BTreeMap<String, String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("task-api", "service", opts.lang, opts.kind)
        .api_style(opts.api_style);

    if opts.has_database {
        builder = builder.with_database(opts.database_type);
    }
    if opts.has_kafka {
        builder = builder.with_kafka();
    }
    if opts.has_redis {
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
    let opts = RenderOptions {
        lang: "rust",
        kind: "server",
        api_style: "rest",
        has_database: false,
        database_type: "",
        has_kafka: false,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["Cargo.toml"]);
    let content = contents
        .get("Cargo.toml")
        .expect("Cargo.toml not generated");
    insta::assert_snapshot!("rust_rest_minimal__cargo_toml", content);
}

/// Rust REST サーバー（最小）— src/main.rs の内容が意図通りに生成されることを検証する。
///
/// エントリーポイントのブートストラップパターン変更はここで検知される。
#[test]
fn test_content_rust_rest_minimal_main_rs() {
    let opts = RenderOptions {
        lang: "rust",
        kind: "server",
        api_style: "rest",
        has_database: false,
        database_type: "",
        has_kafka: false,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["src/main.rs"]);
    let content = contents
        .get("src/main.rs")
        .expect("src/main.rs not generated");
    insta::assert_snapshot!("rust_rest_minimal__main_rs", content);
}

/// Rust REST サーバー（最小）— src/adapter/handler/rest.rs の内容を検証する。
///
/// REST ハンドラーのコードパターン変更はここで検知される。
#[test]
fn test_content_rust_rest_minimal_handler_rs() {
    let opts = RenderOptions {
        lang: "rust",
        kind: "server",
        api_style: "rest",
        has_database: false,
        database_type: "",
        has_kafka: false,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["src/adapter/handler/rest.rs"]);
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
    let opts = RenderOptions {
        lang: "rust",
        kind: "server",
        api_style: "rest",
        has_database: true,
        database_type: "postgresql",
        has_kafka: true,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["Cargo.toml"]);
    let content = contents
        .get("Cargo.toml")
        .expect("Cargo.toml not generated");
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
    let opts = RenderOptions {
        lang: "go",
        kind: "server",
        api_style: "rest",
        has_database: false,
        database_type: "",
        has_kafka: false,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["go.mod"]);
    let content = contents.get("go.mod").expect("go.mod not generated");
    insta::assert_snapshot!("go_rest_minimal__go_mod", content);
}

/// Go REST サーバー（最小）— cmd/main.go の内容を検証する。
///
/// Go エントリーポイントのブートストラップパターン変更はここで検知される。
#[test]
fn test_content_go_rest_minimal_main_go() {
    let opts = RenderOptions {
        lang: "go",
        kind: "server",
        api_style: "rest",
        has_database: false,
        database_type: "",
        has_kafka: false,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["cmd/main.go"]);
    let content = contents
        .get("cmd/main.go")
        .expect("cmd/main.go not generated");
    insta::assert_snapshot!("go_rest_minimal__main_go", content);
}

/// Go REST サーバー（最小）— `internal/adapter/handler/rest_handler.go` の内容を検証する。
///
/// REST ハンドラーのコードパターン変更はここで検知される。
#[test]
fn test_content_go_rest_minimal_handler_go() {
    let opts = RenderOptions {
        lang: "go",
        kind: "server",
        api_style: "rest",
        has_database: false,
        database_type: "",
        has_kafka: false,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["internal/adapter/handler/rest_handler.go"]);
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
    let opts = RenderOptions {
        lang: "go",
        kind: "server",
        api_style: "rest",
        has_database: true,
        database_type: "postgresql",
        has_kafka: true,
        has_redis: false,
    };
    let (_tmp, contents) = render_and_collect(&opts, &["internal/infra/config/config.go"]);
    let content = contents
        .get("internal/infra/config/config.go")
        .expect("internal/infra/config/config.go not generated");
    insta::assert_snapshot!("go_rest_db_kafka__config_go", content);
}
