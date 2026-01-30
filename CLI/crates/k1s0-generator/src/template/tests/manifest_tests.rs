//! Manifest template rendering tests
//!
//! Tests for manifest.json.tera variable expansion across backend templates.

use std::path::PathBuf;
use tera::{Context, Tera};

/// Get the template directory for a given template name.
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

/// Create a base Tera context with all required variables.
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
    ctx.insert("template_version", "0.1.0");
    ctx.insert("fingerprint", "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
    ctx.insert("with_grpc", &false);
    ctx.insert("with_rest", &true);
    ctx.insert("with_db", &false);
    ctx.insert("with_cache", &false);
    ctx.insert("with_docker", &true);
    ctx.insert("has_domain", &false);
    ctx.insert("domain_name", "");
    ctx
}

/// Render a manifest template with date filter support.
fn render_manifest(template_name: &str, ctx: &Context) -> serde_json::Value {
    let dir = template_dir(template_name);
    let template_path = dir.join(".k1s0/manifest.json.tera");
    let content = std::fs::read_to_string(&template_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", template_path.display(), e));

    let mut tera = Tera::default();
    tera.add_raw_template("manifest.json.tera", &content)
        .unwrap();

    // Register a dummy `now` variable and `date` filter is built-in to Tera.
    let mut render_ctx = ctx.clone();
    render_ctx.insert("now", "2026-01-30T00:00:00Z");

    let rendered = tera.render("manifest.json.tera", &render_ctx).unwrap();
    serde_json::from_str(&rendered)
        .unwrap_or_else(|e| panic!("Failed to parse rendered manifest as JSON: {}\n{}", e, rendered))
}

// =============================================================================
// backend-rust manifest.json.tera tests
// =============================================================================

#[test]
fn test_manifest_with_cache_true() {
    let mut ctx = create_base_context();
    ctx.insert("with_cache", &true);
    let json = render_manifest("backend-rust", &ctx);

    let with_cache = json["service"]["features"]["with_cache"]
        .as_bool()
        .expect("with_cache should be a boolean");
    assert!(with_cache, "with_cache should be true");
}

#[test]
fn test_manifest_with_cache_false() {
    let mut ctx = create_base_context();
    ctx.insert("with_cache", &false);
    let json = render_manifest("backend-rust", &ctx);

    let with_cache = json["service"]["features"]["with_cache"]
        .as_bool()
        .expect("with_cache should be a boolean");
    assert!(!with_cache, "with_cache should be false");
}

#[test]
fn test_manifest_with_docker_managed_paths() {
    let mut ctx = create_base_context();
    ctx.insert("with_docker", &true);
    let json = render_manifest("backend-rust", &ctx);

    let managed_paths = json["managed_paths"]
        .as_array()
        .expect("managed_paths should be an array");

    let paths: Vec<&str> = managed_paths
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    assert!(
        paths.contains(&"Dockerfile"),
        "managed_paths should contain Dockerfile"
    );
    assert!(
        paths.contains(&".dockerignore"),
        "managed_paths should contain .dockerignore"
    );
    assert!(
        paths.contains(&"compose.yaml"),
        "managed_paths should contain compose.yaml"
    );
    assert!(
        paths.contains(&"deploy/docker/"),
        "managed_paths should contain deploy/docker/"
    );
}

#[test]
fn test_manifest_update_policy_present() {
    let ctx = create_base_context();
    let json = render_manifest("backend-rust", &ctx);

    let update_policy = json.get("update_policy");
    assert!(
        update_policy.is_some(),
        "manifest should contain update_policy section"
    );

    let policy = update_policy.unwrap().as_object().unwrap();
    assert!(!policy.is_empty(), "update_policy should not be empty");

    // Verify some expected policies
    assert_eq!(
        policy.get("deploy/").and_then(|v| v.as_str()),
        Some("auto"),
        "deploy/ should have auto policy"
    );
    assert_eq!(
        policy.get("src/domain/").and_then(|v| v.as_str()),
        Some("protected"),
        "src/domain/ should have protected policy"
    );
}

#[test]
fn test_manifest_features_all_present() {
    let ctx = create_base_context();
    let json = render_manifest("backend-rust", &ctx);

    let features = json["service"]["features"]
        .as_object()
        .expect("features should be an object");

    let expected_keys = ["with_grpc", "with_rest", "with_db", "with_cache", "with_docker"];
    for key in &expected_keys {
        assert!(
            features.contains_key(*key),
            "features should contain {}",
            key
        );
        assert!(
            features[*key].is_boolean(),
            "{} should be a boolean",
            key
        );
    }
}
