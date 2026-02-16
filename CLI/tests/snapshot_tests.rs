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

// =========================================================================
// ヘルパー関数: Client
// =========================================================================

fn render_client(framework: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("order-app", "service", framework, "client")
        .framework(framework)
        .build();
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
// ヘルパー関数: Library
// =========================================================================

fn render_library(lang: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("order-lib", "system", lang, "library")
        .build();
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
// ヘルパー関数: Database
// =========================================================================

fn render_database(db_type: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("order-db", "service", db_type, "database")
        .with_database(db_type)
        .build();
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
// ヘルパー関数: Helm
// =========================================================================

fn render_helm(
    api_style: &str,
    has_database: bool,
    database_type: &str,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", "go", "helm")
        .api_style(api_style);

    if has_database {
        builder = builder.with_database(database_type);
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
// ヘルパー関数: CICD
// =========================================================================

fn render_cicd(
    lang: &str,
    kind_for_ctx: &str,
    api_style: &str,
    has_database: bool,
    database_type: &str,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, kind_for_ctx)
        .api_style(api_style);

    if has_database {
        builder = builder.with_database(database_type);
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
// スナップショットテスト: Client
// =========================================================================

/// Client: React
#[test]
fn test_snapshot_client_react() {
    let (_, names) = render_client("react");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("client_react", sorted);
}

/// Client: Flutter
#[test]
fn test_snapshot_client_flutter() {
    let (_, names) = render_client("flutter");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("client_flutter", sorted);
}

// =========================================================================
// スナップショットテスト: Library
// =========================================================================

/// Library: Go
#[test]
fn test_snapshot_library_go() {
    let (_, names) = render_library("go");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_go", sorted);
}

/// Library: Rust
#[test]
fn test_snapshot_library_rust() {
    let (_, names) = render_library("rust");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_rust", sorted);
}

/// Library: TypeScript
#[test]
fn test_snapshot_library_typescript() {
    let (_, names) = render_library("typescript");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_typescript", sorted);
}

/// Library: Dart
#[test]
fn test_snapshot_library_dart() {
    let (_, names) = render_library("dart");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("library_dart", sorted);
}

// =========================================================================
// スナップショットテスト: Database
// =========================================================================

/// Database: PostgreSQL
#[test]
fn test_snapshot_database_postgresql() {
    let (_, names) = render_database("postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("database_postgresql", sorted);
}

/// Database: MySQL
#[test]
fn test_snapshot_database_mysql() {
    let (_, names) = render_database("mysql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("database_mysql", sorted);
}

/// Database: SQLite
#[test]
fn test_snapshot_database_sqlite() {
    let (_, names) = render_database("sqlite");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("database_sqlite", sorted);
}

// =========================================================================
// ヘルパー関数: BFF
// =========================================================================

fn render_bff(lang: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("order-api", "service", lang, "bff")
        .api_style("graphql")
        .build();
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
// スナップショットテスト: BFF
// =========================================================================

/// BFF: Go
#[test]
fn test_snapshot_bff_go() {
    let (_, names) = render_bff("go");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("bff_go", sorted);
}

/// BFF: Rust
#[test]
fn test_snapshot_bff_rust() {
    let (_, names) = render_bff("rust");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("bff_rust", sorted);
}

// =========================================================================
// スナップショットテスト: Helm
// =========================================================================

/// Helm: REST + PostgreSQL
#[test]
fn test_snapshot_helm_rest_postgresql() {
    let (_, names) = render_helm("rest", true, "postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_rest_postgresql", sorted);
}

// =========================================================================
// スナップショットテスト: CICD
// =========================================================================

/// CICD: Go REST + PostgreSQL
#[test]
fn test_snapshot_cicd_go_rest_postgresql() {
    let (_, names) = render_cicd("go", "cicd", "rest", true, "postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("cicd_go_rest_postgresql", sorted);
}

/// CICD: Rust gRPC (DB なし)
#[test]
fn test_snapshot_cicd_rust_grpc() {
    let (_, names) = render_cicd("rust", "cicd", "grpc", false, "");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("cicd_rust_grpc", sorted);
}

// =========================================================================
// ヘルパー関数: 複数APIスタイル対応 Server
// =========================================================================

fn render_server_multi(
    lang: &str,
    api_styles: Vec<&str>,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let styles: Vec<String> = api_styles.iter().map(|s| s.to_string()).collect();

    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, "server")
        .api_styles(styles);

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
// スナップショットテスト: 複数APIスタイル (REST+gRPC)
// =========================================================================

/// パターン: Go REST+gRPC + PostgreSQL
#[test]
fn test_snapshot_go_rest_grpc_postgresql() {
    let (_, names) = render_server_multi(
        "go",
        vec!["rest", "grpc"],
        true,
        "postgresql",
        false,
        false,
    );
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("go_rest_grpc_postgresql", sorted);
}

/// パターン: Rust REST+gRPC (最小)
#[test]
fn test_snapshot_rust_rest_grpc_minimal() {
    let (_, names) = render_server_multi(
        "rust",
        vec!["rest", "grpc"],
        false,
        "",
        false,
        false,
    );
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("rust_rest_grpc_minimal", sorted);
}

// =========================================================================
// ヘルパー関数: 複数APIスタイル対応 Helm
// =========================================================================

fn render_helm_multi(
    api_styles: Vec<&str>,
    has_database: bool,
    database_type: &str,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let styles: Vec<String> = api_styles.iter().map(|s| s.to_string()).collect();

    let mut builder = TemplateContextBuilder::new("order-api", "service", "go", "helm")
        .api_styles(styles);

    if has_database {
        builder = builder.with_database(database_type);
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
// スナップショットテスト: Helm 追加パターン
// =========================================================================

/// パターン #23: Helm gRPC
#[test]
fn test_snapshot_helm_grpc() {
    let (_, names) = render_helm_multi(vec!["grpc"], false, "");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_grpc", sorted);
}

/// パターン #24: Helm GraphQL
#[test]
fn test_snapshot_helm_graphql() {
    let (_, names) = render_helm_multi(vec!["graphql"], false, "");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_graphql", sorted);
}

/// パターン #25: Helm REST+gRPC
#[test]
fn test_snapshot_helm_rest_grpc() {
    let (_, names) = render_helm_multi(vec!["rest", "grpc"], true, "postgresql");
    let mut sorted = names.clone();
    sorted.sort();
    insta::assert_yaml_snapshot!("helm_rest_grpc", sorted);
}

/// 複数APIスタイル時にREST・gRPC両方のハンドラが含まれることを検証
#[test]
fn test_go_rest_grpc_includes_both_handlers() {
    let (_, files) = render_server_multi(
        "go",
        vec!["rest", "grpc"],
        true,
        "postgresql",
        false,
        false,
    );
    assert!(files.iter().any(|f| f.contains("rest")), "REST handler should be included");
    assert!(files.iter().any(|f| f.contains("grpc")), "gRPC handler should be included");
}
