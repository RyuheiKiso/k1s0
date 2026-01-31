use super::super::*;
use super::create_backend_rust_structure;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_manifest_with_domain_deps(dir: &std::path::Path, domains: HashMap<String, String>) {
    let k1s0_dir = dir.join(".k1s0");
    fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: "backend-rust".to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: "CLI/templates/backend-rust/feature".to_string(),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-service".to_string(),
            language: "rust".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        layer: crate::manifest::LayerType::Feature,
        domain: None,
        version: None,
        domain_version: None,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec!["deploy/".to_string()],
        protected_paths: vec!["src/domain/".to_string()],
        update_policy: HashMap::new(),
        checksums: HashMap::new(),
        dependencies: Some(crate::manifest::Dependencies {
            framework_crates: vec![],
            framework: vec![],
            domain: Some(domains),
        }),
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();
}

#[test]
fn test_rule_id_k028() {
    assert_eq!(RuleId::UnusedDomainDependency.as_str(), "K028");
}

#[test]
fn test_unused_domain_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let mut domains = HashMap::new();
    domains.insert("user-management".to_string(), "^1.0.0".to_string());

    create_manifest_with_domain_deps(path, domains);
    create_backend_rust_structure(path);

    // src/ にはdomain を使用するコードなし
    fs::write(path.join("src/domain/mod.rs"), "pub struct Foo;").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::UnusedDomainDependency),
        "Expected K028 violation for unused domain",
    );
}

#[test]
fn test_used_domain_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let mut domains = HashMap::new();
    domains.insert("user-management".to_string(), "^1.0.0".to_string());

    create_manifest_with_domain_deps(path, domains);
    create_backend_rust_structure(path);

    let code = r#"
use user_management::UserEntity;

fn get_user() -> UserEntity {
    todo!()
}
"#;
    fs::write(path.join("src/application/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::UnusedDomainDependency),
        "Used domain should not trigger K028",
    );
}

#[test]
fn test_no_domain_deps_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_manifest_with_domain_deps(path, HashMap::new());
    create_backend_rust_structure(path);

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::UnusedDomainDependency),
    );
}

#[test]
fn test_domain_used_via_path_separator() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let mut domains = HashMap::new();
    domains.insert("order-processing".to_string(), "^2.0.0".to_string());

    create_manifest_with_domain_deps(path, domains);
    create_backend_rust_structure(path);

    let code = r#"
use order_processing::OrderEntity;
"#;
    fs::write(path.join("src/domain/mod.rs"), code).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::UnusedDomainDependency),
    );
}
