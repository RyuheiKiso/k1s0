/// テンプレートレンダリング結果のスナップショットテスト。
///
/// `insta` crate を使用して、テンプレートエンジンが生成するファイル一覧の
/// スナップショットを取得・比較する。代表6パターンをカバーする。
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

fn render_server(
    lang: &str,
    api_style: &str,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, "server")
        .api_style(api_style);

    if has_database {
        builder = builder.with_database(database_type);
    }
    if has_kafka {
        builder = builder.with_kafka();
    }
    if has_redis {
        builder = builder.with_redis();
    }

    let ctx = builder.build();
    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    (tmp, names)
}

// =========================================================================
// スナップショットテスト: 代表6パターン
// =========================================================================

/// パターン1: Go REST + PostgreSQL + Kafka + Redis (フルスタック)
#[test]
fn test_snapshot_go_rest_full_stack() {
    let (_, names) = render_server("go", "rest", true, "postgresql", true, true);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("go_rest_full_stack", sorted);
}

/// パターン2: Go gRPC (最小)
#[test]
fn test_snapshot_go_grpc_minimal() {
    let (_, names) = render_server("go", "grpc", false, "", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("go_grpc_minimal", sorted);
}

/// パターン3: Go GraphQL (最小)
#[test]
fn test_snapshot_go_graphql_minimal() {
    let (_, names) = render_server("go", "graphql", false, "", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("go_graphql_minimal", sorted);
}

/// パターン4: Rust REST + PostgreSQL (DB のみ)
#[test]
fn test_snapshot_rust_rest_db_only() {
    let (_, names) = render_server("rust", "rest", true, "postgresql", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("rust_rest_db_only", sorted);
}

/// パターン5: Rust gRPC (最小)
#[test]
fn test_snapshot_rust_grpc_minimal() {
    let (_, names) = render_server("rust", "grpc", false, "", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("rust_grpc_minimal", sorted);
}

/// パターン6: Rust GraphQL (最小)
#[test]
fn test_snapshot_rust_graphql_minimal() {
    let (_, names) = render_server("rust", "graphql", false, "", false, false);
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("rust_graphql_minimal", sorted);
}
