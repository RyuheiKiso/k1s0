//! Docker template rendering tests
//!
//! Tests for Dockerfile.tera and compose.yaml.tera across all 6 templates.

use std::path::PathBuf;
use tera::{Context, Tera};

/// テンプレートディレクトリを取得
fn template_dir(template_name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // CLI/
        .join("templates")
        .join(template_name)
        .join("feature")
}

/// ベースコンテキストを作成
fn create_base_context() -> Context {
    let mut ctx = Context::new();
    ctx.insert("feature_name", "test-service");
    ctx.insert("feature_name_snake", "test_service");
    ctx.insert("feature_name_pascal", "TestService");
    ctx.insert("feature_name_kebab", "test-service");
    ctx.insert("feature_name_title", "Test Service");
    ctx.insert("service_name", "test-service");
    ctx.insert("language", "rust");
    ctx.insert("service_type", "backend");
    ctx.insert("layer", "feature");
    ctx.insert("k1s0_version", "0.1.0");
    ctx.insert("with_grpc", &false);
    ctx.insert("with_rest", &true);
    ctx.insert("with_db", &false);
    ctx.insert("with_cache", &false);
    ctx.insert("with_docker", &true);
    ctx.insert("has_domain", &false);
    ctx.insert("domain_name", "");
    ctx
}

/// テンプレートをレンダリング
fn render_template(template_name: &str, file: &str, ctx: &Context) -> String {
    let dir = template_dir(template_name);
    let template_path = dir.join(file);
    let content = std::fs::read_to_string(&template_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", template_path.display(), e));

    let mut tera = Tera::default();
    tera.add_raw_template(file, &content).unwrap();
    tera.render(file, ctx).unwrap()
}

// =============================================================================
// Dockerfile.tera tests - backend-rust
// =============================================================================

#[test]
fn test_rust_dockerfile_basic() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "Dockerfile.tera", &ctx);

    assert!(output.contains("FROM rust:"), "Should use rust base image");
    assert!(output.contains("EXPOSE 8080"), "Should expose port 8080");
    assert!(output.contains("appuser"), "Should use non-root user");
    assert!(output.contains("HEALTHCHECK"), "Should have healthcheck");
    assert!(output.contains("ENTRYPOINT"), "Should have entrypoint");
    assert!(output.contains("test_service"), "Should contain snake_case name");
    assert!(output.contains("ARG HTTP_PROXY"), "Should have proxy support");
    assert!(output.contains("ARG HTTPS_PROXY"), "Should have HTTPS proxy support");
    assert!(output.contains("ARG NO_PROXY"), "Should have NO_PROXY support");
}

#[test]
fn test_rust_dockerfile_with_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("with_grpc", &true);
    let output = render_template("backend-rust", "Dockerfile.tera", &ctx);

    assert!(output.contains("EXPOSE 50051"), "Should expose gRPC port");
    assert!(output.contains("proto/"), "Should copy proto directory");
    assert!(output.contains("buf.yaml"), "Should copy buf.yaml");
    assert!(output.contains("buf.gen.yaml"), "Should copy buf.gen.yaml");
}

#[test]
fn test_rust_dockerfile_without_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("with_grpc", &false);
    let output = render_template("backend-rust", "Dockerfile.tera", &ctx);

    assert!(!output.contains("EXPOSE 50051"), "Should not expose gRPC port");
    assert!(!output.contains("proto/"), "Should not copy proto directory");
}

#[test]
fn test_rust_dockerfile_multistage() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "Dockerfile.tera", &ctx);

    assert!(output.contains("AS build"), "Should have build stage");
    assert!(output.contains("AS runtime"), "Should have runtime stage");
    assert!(output.contains("COPY --from=build"), "Should copy from build stage");
    assert!(output.contains("debian:bookworm-slim"), "Should use slim runtime image");
}

#[test]
fn test_rust_dockerfile_dependency_caching() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "Dockerfile.tera", &ctx);

    assert!(output.contains("COPY Cargo.toml"), "Should copy manifests first");
    assert!(output.contains("Create dummy src"), "Should use dummy src for caching");
    assert!(output.contains("cargo build --release"), "Should build twice for layer caching");
}

// =============================================================================
// Dockerfile.tera tests - backend-go
// =============================================================================

#[test]
fn test_go_dockerfile_basic() {
    let mut ctx = create_base_context();
    ctx.insert("language", "go");
    let output = render_template("backend-go", "Dockerfile.tera", &ctx);

    assert!(output.contains("FROM golang:"), "Should use golang base image");
    assert!(output.contains("distroless"), "Should use distroless runtime");
    assert!(output.contains("CGO_ENABLED=0"), "Should disable CGO");
    assert!(output.contains("EXPOSE 8080"), "Should expose port 8080");
    assert!(!output.contains("HEALTHCHECK"), "Distroless should not have HEALTHCHECK");
    assert!(output.contains("ARG HTTP_PROXY"), "Should have proxy support");
}

#[test]
fn test_go_dockerfile_with_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("language", "go");
    ctx.insert("with_grpc", &true);
    let output = render_template("backend-go", "Dockerfile.tera", &ctx);

    assert!(output.contains("EXPOSE 50051"), "Should expose gRPC port");
    assert!(output.contains("proto/"), "Should copy proto directory");
}

#[test]
fn test_go_dockerfile_static_binary() {
    let mut ctx = create_base_context();
    ctx.insert("language", "go");
    let output = render_template("backend-go", "Dockerfile.tera", &ctx);

    assert!(output.contains("GOOS=linux"), "Should build for Linux");
    assert!(output.contains("-ldflags=\"-s -w\""), "Should strip symbols");
    assert!(output.contains("nonroot:nonroot"), "Should use nonroot user");
}

// =============================================================================
// Dockerfile.tera tests - backend-csharp
// =============================================================================

#[test]
fn test_csharp_dockerfile_basic() {
    let mut ctx = create_base_context();
    ctx.insert("language", "csharp");
    let output = render_template("backend-csharp", "Dockerfile.tera", &ctx);

    assert!(output.contains("dotnet"), "Should use dotnet");
    assert!(output.contains("HEALTHCHECK"), "Should have healthcheck");
    assert!(output.contains("health/live"), "Should check /health/live");
    assert!(output.contains("ARG HTTP_PROXY"), "Should have proxy support");
    assert!(output.contains("EXPOSE 8080"), "Should expose port 8080");
}

#[test]
fn test_csharp_dockerfile_with_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("language", "csharp");
    ctx.insert("with_grpc", &true);
    let output = render_template("backend-csharp", "Dockerfile.tera", &ctx);

    assert!(output.contains("EXPOSE 50051"), "Should expose gRPC port");
}

// =============================================================================
// Dockerfile.tera tests - backend-python
// =============================================================================

#[test]
fn test_python_dockerfile_basic() {
    let mut ctx = create_base_context();
    ctx.insert("language", "python");
    let output = render_template("backend-python", "Dockerfile.tera", &ctx);

    assert!(output.contains("python:"), "Should use python base image");
    assert!(output.contains("HEALTHCHECK"), "Should have healthcheck");
    assert!(output.contains("health/live"), "Should check /health/live");
    assert!(output.contains("ARG HTTP_PROXY"), "Should have proxy support");
    assert!(output.contains("EXPOSE 8080"), "Should expose port 8080");
}

#[test]
fn test_python_dockerfile_with_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("language", "python");
    ctx.insert("with_grpc", &true);
    let output = render_template("backend-python", "Dockerfile.tera", &ctx);

    assert!(output.contains("EXPOSE 50051"), "Should expose gRPC port");
}

// =============================================================================
// Dockerfile.tera tests - frontend-react
// =============================================================================

#[test]
fn test_react_dockerfile_basic() {
    let mut ctx = create_base_context();
    ctx.insert("language", "typescript");
    ctx.insert("service_type", "frontend");
    let output = render_template("frontend-react", "Dockerfile.tera", &ctx);

    assert!(output.contains("node:"), "Should use node base image");
    assert!(output.contains("nginx:"), "Should use nginx runtime");
    assert!(output.contains("EXPOSE 80"), "Should expose port 80");
    assert!(output.contains("HEALTHCHECK"), "Should have healthcheck");
    assert!(output.contains("ARG HTTP_PROXY"), "Should have proxy support");
}

#[test]
fn test_react_dockerfile_build_process() {
    let mut ctx = create_base_context();
    ctx.insert("language", "typescript");
    ctx.insert("service_type", "frontend");
    let output = render_template("frontend-react", "Dockerfile.tera", &ctx);

    assert!(output.contains("pnpm"), "Should use pnpm");
    assert!(output.contains("pnpm install"), "Should install dependencies");
    assert!(output.contains("pnpm build"), "Should build for production");
}

// =============================================================================
// Dockerfile.tera tests - frontend-flutter
// =============================================================================

#[test]
fn test_flutter_dockerfile_basic() {
    let mut ctx = create_base_context();
    ctx.insert("language", "dart");
    ctx.insert("service_type", "frontend");
    let output = render_template("frontend-flutter", "Dockerfile.tera", &ctx);

    assert!(output.contains("flutter"), "Should use flutter base image");
    assert!(output.contains("nginx:"), "Should use nginx runtime");
    assert!(output.contains("EXPOSE 80"), "Should expose port 80");
    assert!(output.contains("HEALTHCHECK"), "Should have healthcheck");
}

#[test]
fn test_flutter_dockerfile_build_process() {
    let mut ctx = create_base_context();
    ctx.insert("language", "dart");
    ctx.insert("service_type", "frontend");
    let output = render_template("frontend-flutter", "Dockerfile.tera", &ctx);

    assert!(output.contains("flutter pub get"), "Should get dependencies");
    assert!(output.contains("flutter build web"), "Should build for web");
}

// =============================================================================
// compose.yaml.tera tests - backend-rust
// =============================================================================

#[test]
fn test_rust_compose_basic() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("services:"), "Should have services section");
    assert!(output.contains("test-service:"), "Should define service");
    assert!(output.contains("build:"), "Should have build config");
    assert!(output.contains("8080:8080"), "Should map HTTP port");
    assert!(output.contains("networks:"), "Should have networks");
    assert!(output.contains("test-service-network"), "Should have service network");
}

#[test]
fn test_rust_compose_with_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("with_grpc", &true);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("50051:50051"), "Should map gRPC port");
}

#[test]
fn test_rust_compose_without_grpc() {
    let mut ctx = create_base_context();
    ctx.insert("with_grpc", &false);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(!output.contains("50051:50051"), "Should not map gRPC port");
}

#[test]
fn test_rust_compose_with_db() {
    let mut ctx = create_base_context();
    ctx.insert("with_db", &true);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("postgres:"), "Should contain postgres service");
    assert!(output.contains("POSTGRES_PASSWORD_FILE"), "Should use password file");
    assert!(output.contains("K021"), "Should have K021 comment");
    assert!(output.contains("postgres-data"), "Should have postgres volume");
    assert!(output.contains("secrets:"), "Should have secrets section");
    assert!(output.contains("db_password"), "Should have db_password secret");
    assert!(output.contains("depends_on:"), "Should depend on postgres");
    assert!(output.contains("service_healthy"), "Should wait for postgres health");
}

#[test]
fn test_rust_compose_without_db() {
    let mut ctx = create_base_context();
    ctx.insert("with_db", &false);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(!output.contains("postgres:16"), "Should not contain postgres");
    assert!(!output.contains("POSTGRES_PASSWORD_FILE"), "Should not have password file");
    assert!(!output.contains("postgres-data"), "Should not have postgres volume");
}

#[test]
fn test_rust_compose_with_cache() {
    let mut ctx = create_base_context();
    ctx.insert("with_cache", &true);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("redis:"), "Should contain redis service");
    assert!(output.contains("redis-data"), "Should have redis volume");
    assert!(output.contains("redis-cli"), "Should have redis healthcheck");
}

#[test]
fn test_rust_compose_without_cache() {
    let mut ctx = create_base_context();
    ctx.insert("with_cache", &false);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(!output.contains("redis:7"), "Should not contain redis");
    assert!(!output.contains("redis-data"), "Should not have redis volume");
}

#[test]
fn test_compose_otel_observability_profile() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("otel-collector"), "Should have otel-collector");
    assert!(output.contains("observability"), "Should have observability profile");
    assert!(output.contains("4317:4317"), "Should expose gRPC OTLP port");
    assert!(output.contains("4318:4318"), "Should expose HTTP OTLP port");
    assert!(output.contains("profiles:"), "Should have profiles section");
}

#[test]
fn test_compose_proxy_build_args() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("HTTP_PROXY"), "Should have HTTP_PROXY build arg");
    assert!(output.contains("HTTPS_PROXY"), "Should have HTTPS_PROXY build arg");
    assert!(output.contains("NO_PROXY"), "Should have NO_PROXY build arg");
    assert!(output.contains("${HTTP_PROXY:-}"), "Should use environment variable with default");
}

#[test]
fn test_compose_volumes_conditional() {
    let mut ctx = create_base_context();
    ctx.insert("with_db", &true);
    ctx.insert("with_cache", &true);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("volumes:"), "Should have volumes section");
    assert!(output.contains("postgres-data:"), "Should have postgres volume");
    assert!(output.contains("redis-data:"), "Should have redis volume");
}

// =============================================================================
// compose.yaml.tera tests - other backends
// =============================================================================

#[test]
fn test_go_compose_with_db_and_cache() {
    let mut ctx = create_base_context();
    ctx.insert("language", "go");
    ctx.insert("with_db", &true);
    ctx.insert("with_cache", &true);
    let output = render_template("backend-go", "compose.yaml.tera", &ctx);

    assert!(output.contains("postgres:"), "Should contain postgres");
    assert!(output.contains("redis:"), "Should contain redis");
    assert!(output.contains("otel-collector"), "Should have otel-collector");
}

#[test]
fn test_csharp_compose_basic() {
    let mut ctx = create_base_context();
    ctx.insert("language", "csharp");
    let output = render_template("backend-csharp", "compose.yaml.tera", &ctx);

    assert!(output.contains("test-service:"), "Should define service");
    assert!(output.contains("8080:8080"), "Should map HTTP port");
}

#[test]
fn test_python_compose_with_db() {
    let mut ctx = create_base_context();
    ctx.insert("language", "python");
    ctx.insert("with_db", &true);
    let output = render_template("backend-python", "compose.yaml.tera", &ctx);

    assert!(output.contains("postgres:"), "Should contain postgres");
    assert!(output.contains("K021"), "Should have K021 comment");
}

// =============================================================================
// compose.yaml.tera tests - frontend
// =============================================================================

#[test]
fn test_frontend_react_compose_no_db_no_cache() {
    let mut ctx = create_base_context();
    ctx.insert("service_type", "frontend");
    ctx.insert("language", "typescript");
    let output = render_template("frontend-react", "compose.yaml.tera", &ctx);

    assert!(!output.contains("postgres"), "Frontend should not have postgres");
    assert!(!output.contains("redis"), "Frontend should not have redis");
    assert!(output.contains("otel-collector"), "Should have otel-collector");
    assert!(output.contains("3000:80"), "Should map port 3000 to 80");
}

#[test]
fn test_frontend_flutter_compose_basic() {
    let mut ctx = create_base_context();
    ctx.insert("service_type", "frontend");
    ctx.insert("language", "dart");
    let output = render_template("frontend-flutter", "compose.yaml.tera", &ctx);

    assert!(!output.contains("postgres"), "Frontend should not have postgres");
    assert!(!output.contains("redis"), "Frontend should not have redis");
    assert!(output.contains("otel-collector"), "Should have otel-collector");
}

// =============================================================================
// Dockerfile.monorepo.tera tests
// =============================================================================

#[test]
fn test_rust_monorepo_dockerfile() {
    let ctx = create_base_context();
    let output = render_template("backend-rust", "Dockerfile.monorepo.tera", &ctx);

    assert!(output.contains("framework/backend/rust"), "Should copy framework");
    assert!(output.contains("feature/backend/rust/test-service"), "Should copy feature");
    assert!(output.contains("HEALTHCHECK"), "Should have healthcheck");
    assert!(output.contains("ARG HTTP_PROXY"), "Should have proxy support");
    assert!(output.contains("/workspace"), "Should use workspace dir");
}

#[test]
fn test_rust_monorepo_dockerfile_with_domain() {
    let mut ctx = create_base_context();
    ctx.insert("has_domain", &true);
    ctx.insert("domain_name", "user-management");
    let output = render_template("backend-rust", "Dockerfile.monorepo.tera", &ctx);

    assert!(
        output.contains("domain/backend/rust/user-management"),
        "Should copy domain crate"
    );
}

#[test]
fn test_rust_monorepo_dockerfile_without_domain() {
    let mut ctx = create_base_context();
    ctx.insert("has_domain", &false);
    let output = render_template("backend-rust", "Dockerfile.monorepo.tera", &ctx);

    assert!(!output.contains("COPY domain/"), "Should not copy domain when not present");
}

#[test]
fn test_go_monorepo_dockerfile() {
    let mut ctx = create_base_context();
    ctx.insert("language", "go");
    let output = render_template("backend-go", "Dockerfile.monorepo.tera", &ctx);

    assert!(output.contains("framework/backend/go"), "Should copy framework");
    assert!(output.contains("feature/backend/go/test-service"), "Should copy feature");
    assert!(output.contains("distroless"), "Should use distroless");
}

#[test]
fn test_csharp_monorepo_dockerfile() {
    let mut ctx = create_base_context();
    ctx.insert("language", "csharp");
    let output = render_template("backend-csharp", "Dockerfile.monorepo.tera", &ctx);

    assert!(output.contains("framework/backend/csharp"), "Should copy framework");
    assert!(output.contains("feature/backend/csharp/test-service"), "Should copy feature");
}

#[test]
fn test_python_monorepo_dockerfile() {
    let mut ctx = create_base_context();
    ctx.insert("language", "python");
    let output = render_template("backend-python", "Dockerfile.monorepo.tera", &ctx);

    assert!(output.contains("framework/backend/python"), "Should copy framework");
    assert!(output.contains("feature/backend/python/test-service"), "Should copy feature");
}

#[test]
fn test_react_monorepo_dockerfile() {
    let mut ctx = create_base_context();
    ctx.insert("language", "typescript");
    ctx.insert("service_type", "frontend");
    let output = render_template("frontend-react", "Dockerfile.monorepo.tera", &ctx);

    assert!(output.contains("framework/frontend/react"), "Should copy framework");
    assert!(output.contains("feature/frontend/react/test-service"), "Should copy feature");
}

#[test]
fn test_flutter_monorepo_dockerfile() {
    let mut ctx = create_base_context();
    ctx.insert("language", "dart");
    ctx.insert("service_type", "frontend");
    let output = render_template("frontend-flutter", "Dockerfile.monorepo.tera", &ctx);

    assert!(output.contains("framework/frontend/flutter"), "Should copy framework");
    assert!(output.contains("feature/frontend/flutter/test-service"), "Should copy feature");
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_all_options_enabled() {
    let mut ctx = create_base_context();
    ctx.insert("with_grpc", &true);
    ctx.insert("with_db", &true);
    ctx.insert("with_cache", &true);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("50051:50051"), "Should have gRPC port");
    assert!(output.contains("postgres:"), "Should have postgres");
    assert!(output.contains("redis:"), "Should have redis");
    assert!(output.contains("otel-collector"), "Should have otel-collector");
    assert!(output.contains("postgres-data:"), "Should have postgres volume");
    assert!(output.contains("redis-data:"), "Should have redis volume");
}

#[test]
fn test_minimal_configuration() {
    let mut ctx = create_base_context();
    ctx.insert("with_grpc", &false);
    ctx.insert("with_db", &false);
    ctx.insert("with_cache", &false);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(!output.contains("50051:50051"), "Should not have gRPC port");
    assert!(!output.contains("postgres:16"), "Should not have postgres");
    assert!(!output.contains("redis:7"), "Should not have redis");
    assert!(output.contains("otel-collector"), "Should still have otel-collector");
}

#[test]
fn test_service_name_in_urls() {
    let mut ctx = create_base_context();
    ctx.insert("with_db", &true);
    let output = render_template("backend-rust", "compose.yaml.tera", &ctx);

    assert!(output.contains("test-service:"), "Service name should appear in YAML");
    assert!(output.contains("test-service-network"), "Network should use service name");
    assert!(output.contains("test-service-postgres"), "Postgres container should use service name");
}
