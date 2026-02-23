/// BFF テンプレートのレンダリング統合テスト。
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

fn read_bff_output(tmp: &TempDir, path: &str) -> String {
    fs::read_to_string(tmp.path().join("output").join(path)).unwrap()
}

// =========================================================================
// BFF テンプレート: 内容検証
// =========================================================================

#[test]
fn test_go_bff_resolver_has_query_mutation() {
    let (tmp, _) = render_bff("go");
    let content = read_bff_output(&tmp, "internal/handler/graphql_resolver.go");

    assert!(
        content.contains("func (r *Resolver) Query()"),
        "Go BFF resolver should have Query method"
    );
    assert!(
        content.contains("func (r *Resolver) Mutation()"),
        "Go BFF resolver should have Mutation method"
    );
    assert!(
        content.contains("QueryResolver"),
        "Go BFF resolver should have QueryResolver type"
    );
    assert!(
        content.contains("MutationResolver"),
        "Go BFF resolver should have MutationResolver type"
    );
    assert!(
        content.contains("OrderApi"),
        "Go BFF resolver should use PascalCase service name"
    );
}

#[test]
fn test_go_bff_has_upstream_client() {
    let (_, names) = render_bff("go");

    assert!(
        names
            .iter()
            .any(|n| n.contains("internal/client/upstream.go")),
        "Go BFF should generate upstream client file. Files: {names:?}"
    );
}

#[test]
fn test_go_bff_upstream_client_content() {
    let (tmp, _) = render_bff("go");
    let content = read_bff_output(&tmp, "internal/client/upstream.go");

    assert!(
        content.contains("type UpstreamClient struct"),
        "upstream client should have UpstreamClient struct"
    );
    assert!(
        content.contains("func NewUpstreamClient("),
        "upstream client should have constructor"
    );
    assert!(
        content.contains("func (c *UpstreamClient) Get("),
        "upstream client should have Get method"
    );
    assert!(
        content.contains("func (c *UpstreamClient) GetList("),
        "upstream client should have GetList method"
    );
    assert!(
        content.contains("func (c *UpstreamClient) Post("),
        "upstream client should have Post method"
    );
}

#[test]
fn test_go_bff_has_resolver_test() {
    let (_, names) = render_bff("go");

    assert!(
        names.iter().any(|n| n.contains("graphql_resolver_test.go")),
        "Go BFF should generate resolver test file. Files: {names:?}"
    );
}

#[test]
fn test_go_bff_resolver_test_content() {
    let (tmp, _) = render_bff("go");
    let content = read_bff_output(&tmp, "internal/handler/graphql_resolver_test.go");

    assert!(
        content.contains("TestNewResolver"),
        "resolver test should have TestNewResolver"
    );
    assert!(
        content.contains("TestResolverQuery"),
        "resolver test should have TestResolverQuery"
    );
    assert!(
        content.contains("TestResolverMutation"),
        "resolver test should have TestResolverMutation"
    );
}

#[test]
fn test_rust_bff_has_async_graphql_schema() {
    let (tmp, _) = render_bff("rust");
    let content = read_bff_output(&tmp, "src/handler/graphql.rs");

    assert!(
        content.contains("async_graphql"),
        "Rust BFF should use async_graphql"
    );
    assert!(
        content.contains("Schema"),
        "Rust BFF should have Schema type"
    );
    assert!(
        content.contains("pub struct QueryRoot"),
        "Rust BFF should have QueryRoot"
    );
    assert!(
        content.contains("pub struct MutationRoot"),
        "Rust BFF should have MutationRoot"
    );
    assert!(
        content.contains("pub fn build_schema("),
        "Rust BFF should have build_schema function"
    );
    assert!(
        content.contains("OrderApiBffSchema"),
        "Rust BFF should have typed schema alias"
    );
}

#[test]
fn test_rust_bff_has_upstream_client() {
    let (_, names) = render_bff("rust");

    assert!(
        names.iter().any(|n| n.contains("src/client/upstream.rs")),
        "Rust BFF should generate upstream client file. Files: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("src/client/mod.rs")),
        "Rust BFF should generate client/mod.rs file. Files: {names:?}"
    );
}

#[test]
fn test_rust_bff_upstream_client_content() {
    let (tmp, _) = render_bff("rust");
    let content = read_bff_output(&tmp, "src/client/upstream.rs");

    assert!(
        content.contains("pub struct UpstreamClient"),
        "Rust BFF upstream should have UpstreamClient struct"
    );
    assert!(
        content.contains("pub fn new("),
        "Rust BFF upstream should have constructor"
    );
    assert!(
        content.contains("pub async fn get("),
        "Rust BFF upstream should have get method"
    );
    assert!(
        content.contains("pub async fn get_list("),
        "Rust BFF upstream should have get_list method"
    );
    assert!(
        content.contains("pub async fn post("),
        "Rust BFF upstream should have post method"
    );
}

#[test]
fn test_rust_bff_has_handler_mod() {
    let (_, names) = render_bff("rust");

    assert!(
        names.iter().any(|n| n.contains("src/handler/mod.rs")),
        "Rust BFF should generate handler/mod.rs file. Files: {names:?}"
    );
}

#[test]
fn test_rust_bff_has_integration_test() {
    let (_, names) = render_bff("rust");

    assert!(
        names
            .iter()
            .any(|n| n.contains("tests/integration_test.rs")),
        "Rust BFF should generate integration test file. Files: {names:?}"
    );
}

#[test]
fn test_rust_bff_integration_test_content() {
    let (tmp, _) = render_bff("rust");
    let content = read_bff_output(&tmp, "tests/integration_test.rs");

    assert!(
        content.contains("build_schema"),
        "integration test should reference build_schema"
    );
    assert!(
        content.contains("UpstreamClient"),
        "integration test should reference UpstreamClient"
    );
    assert!(
        content.contains("test_schema_creation"),
        "integration test should have test_schema_creation"
    );
}

#[test]
fn test_bff_go_config_has_upstream_grpc() {
    let (tmp, _) = render_bff("go");
    let content = read_bff_output(&tmp, "config/config.yaml");

    assert!(
        content.contains("grpc_address:"),
        "Go BFF config should have upstream.grpc_address"
    );
    assert!(
        content.contains("http_url:"),
        "Go BFF config should have upstream.http_url"
    );
}

#[test]
fn test_bff_rust_config_has_upstream_grpc() {
    let (tmp, _) = render_bff("rust");
    let content = read_bff_output(&tmp, "config/config.yaml");

    assert!(
        content.contains("grpc_address:"),
        "Rust BFF config should have upstream.grpc_address"
    );
    assert!(
        content.contains("http_url:"),
        "Rust BFF config should have upstream.http_url"
    );
}

#[test]
fn test_go_bff_schema_has_query_mutation() {
    let (tmp, _) = render_bff("go");
    let content = read_bff_output(&tmp, "api/graphql/schema.graphql");

    assert!(
        content.contains("type Query"),
        "Go BFF schema should have Query type"
    );
    assert!(
        content.contains("type Mutation"),
        "Go BFF schema should have Mutation type"
    );
    assert!(
        content.contains("OrderApi"),
        "Go BFF schema should use PascalCase service name"
    );
    assert!(
        content.contains("CreateOrderApiInput"),
        "Go BFF schema should have create input type"
    );
}

#[test]
fn test_rust_bff_main_has_client_and_handler_mods() {
    let (tmp, _) = render_bff("rust");
    let content = read_bff_output(&tmp, "src/main.rs");

    assert!(
        content.contains("mod client;"),
        "Rust BFF main should have client module"
    );
    assert!(
        content.contains("mod handler;"),
        "Rust BFF main should have handler module"
    );
    assert!(
        content.contains("build_schema"),
        "Rust BFF main should call build_schema"
    );
    assert!(
        content.contains("UpstreamClient"),
        "Rust BFF main should use UpstreamClient"
    );
    assert!(
        content.contains("graphql_handler"),
        "Rust BFF main should have graphql_handler"
    );
}

#[test]
fn test_rust_bff_cargo_toml_has_reqwest() {
    let (tmp, _) = render_bff("rust");
    let content = read_bff_output(&tmp, "Cargo.toml");

    assert!(
        content.contains("reqwest"),
        "Rust BFF Cargo.toml should have reqwest dependency"
    );
    assert!(
        content.contains("anyhow"),
        "Rust BFF Cargo.toml should have anyhow dependency"
    );
    assert!(
        content.contains("async-graphql-actix-web"),
        "Rust BFF Cargo.toml should have async-graphql-actix-web"
    );
}
