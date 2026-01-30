//! Docker-specific lint tests (K010/K011)
//!
//! Tests that verify Dockerfile is required for feature layer but not for domain layer.

use super::super::*;
use super::create_test_manifest;
use crate::manifest::LayerType;
use tempfile::TempDir;

/// feature層の構造を作成（Dockerfileあり）
fn create_feature_backend_rust_with_docker(dir: &std::path::Path) {
    // manifest作成
    create_test_manifest(dir, "backend-rust");

    // ディレクトリ作成
    for d in &[
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "config",
        "deploy/base",
        "deploy/overlays/dev",
        "deploy/overlays/stg",
        "deploy/overlays/prod",
        "deploy/docker",
    ] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileを含む）
    for f in &[
        "Cargo.toml",
        "README.md",
        "src/main.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
        "src/presentation/mod.rs",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
        "Dockerfile",
    ] {
        std::fs::write(dir.join(f), "").unwrap();
    }
}

/// feature層の構造を作成（Dockerfileなし）
fn create_feature_backend_rust_without_docker(dir: &std::path::Path) {
    // manifest作成
    create_test_manifest(dir, "backend-rust");

    // ディレクトリ作成
    for d in &[
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "config",
        "deploy/base",
        "deploy/overlays/dev",
        "deploy/overlays/stg",
        "deploy/overlays/prod",
        "deploy/docker",
    ] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileを除く）
    for f in &[
        "Cargo.toml",
        "README.md",
        "src/main.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
        "src/presentation/mod.rs",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
        // Dockerfile は意図的に除外
    ] {
        std::fs::write(dir.join(f), "").unwrap();
    }
}

/// domain層の構造を作成（Dockerfileなし）
fn create_domain_backend_rust_without_docker(dir: &std::path::Path) {
    let k1s0_dir = dir.join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: "backend-rust".to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: "CLI/templates/backend-rust/domain".to_string(),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-domain".to_string(),
            language: "rust".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        layer: LayerType::Domain,
        domain: Some("test-domain".to_string()),
        version: Some("1.0.0".to_string()),
        domain_version: None,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec![],
        protected_paths: vec![],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();

    // ディレクトリ作成
    for d in &["src/domain", "src/application", "src/infrastructure"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileなし）
    for f in &[
        "Cargo.toml",
        "README.md",
        "src/lib.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
    ] {
        std::fs::write(dir.join(f), "").unwrap();
    }
}

#[test]
fn test_feature_layer_dockerfile_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // feature層の構造を作成（Dockerfileなし）
    create_feature_backend_rust_without_docker(path);

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K011エラーが発生するはず
    assert!(!result.is_success(), "Should have violations");
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        !dockerfile_violations.is_empty(),
        "Should have K011 violation for missing Dockerfile"
    );
}

#[test]
fn test_feature_layer_dockerfile_present() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // feature層の構造を作成（Dockerfileあり）
    create_feature_backend_rust_with_docker(path);

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // Dockerfileに関するエラーはないはず
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        dockerfile_violations.is_empty(),
        "Should not have K011 violation for Dockerfile when present"
    );
}

#[test]
fn test_domain_layer_dockerfile_not_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // domain層の構造を作成（Dockerfileなし）
    create_domain_backend_rust_without_docker(path);

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // Dockerfileに関するエラーはないはず（domain層には不要）
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        dockerfile_violations.is_empty(),
        "Domain layer should not require Dockerfile"
    );
}

#[test]
fn test_feature_layer_deploy_docker_dir_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // manifest作成
    create_test_manifest(path, "backend-rust");

    // ディレクトリ作成（deploy/dockerを除く）
    for d in &[
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "config",
        "deploy/base",
        "deploy/overlays/dev",
        "deploy/overlays/stg",
        "deploy/overlays/prod",
        // deploy/docker を意図的に除外
    ] {
        std::fs::create_dir_all(path.join(d)).unwrap();
    }

    // ファイル作成
    for f in &[
        "Cargo.toml",
        "README.md",
        "src/main.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
        "src/presentation/mod.rs",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
        "Dockerfile",
    ] {
        std::fs::write(path.join(f), "").unwrap();
    }

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K010エラーが発生するはず
    assert!(!result.is_success(), "Should have violations");
    let docker_dir_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredDirMissing && v.message.contains("deploy/docker"))
        .collect();

    assert!(
        !docker_dir_violations.is_empty(),
        "Should have K010 violation for missing deploy/docker directory"
    );
}

#[test]
fn test_frontend_react_dockerfile_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let k1s0_dir = path.join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: "frontend-react".to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: "CLI/templates/frontend-react/feature".to_string(),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-frontend".to_string(),
            language: "typescript".to_string(),
            service_type: "frontend".to_string(),
            framework: None,
        },
        layer: LayerType::Feature,
        domain: None,
        version: None,
        domain_version: None,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec![],
        protected_paths: vec![],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();

    // ディレクトリ作成
    for d in &[
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "src/pages",
        "src/components/layout",
        "config",
        "deploy/docker",
    ] {
        std::fs::create_dir_all(path.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileを除く）
    for f in &[
        "package.json",
        "README.md",
        "src/main.tsx",
        "src/App.tsx",
        "config/default.yaml",
        // Dockerfile を意図的に除外
    ] {
        std::fs::write(path.join(f), "").unwrap();
    }

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K011エラーが発生するはず
    assert!(!result.is_success(), "Should have violations");
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        !dockerfile_violations.is_empty(),
        "Frontend feature layer should require Dockerfile"
    );
}

#[test]
fn test_backend_go_dockerfile_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let k1s0_dir = path.join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: "backend-go".to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: "CLI/templates/backend-go/feature".to_string(),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-go-service".to_string(),
            language: "go".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        layer: LayerType::Feature,
        domain: None,
        version: None,
        domain_version: None,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec![],
        protected_paths: vec![],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();

    // ディレクトリ作成
    for d in &[
        "cmd",
        "internal/domain",
        "internal/application",
        "internal/infrastructure",
        "internal/presentation",
        "config",
        "deploy/docker",
    ] {
        std::fs::create_dir_all(path.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileを除く）
    for f in &[
        "go.mod",
        "README.md",
        "cmd/main.go",
        "config/default.yaml",
        // Dockerfile を意図的に除外
    ] {
        std::fs::write(path.join(f), "").unwrap();
    }

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K011エラーが発生するはず
    assert!(!result.is_success(), "Should have violations");
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        !dockerfile_violations.is_empty(),
        "Go backend feature layer should require Dockerfile"
    );
}

#[test]
fn test_backend_python_dockerfile_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let k1s0_dir = path.join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: "backend-python".to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: "CLI/templates/backend-python/feature".to_string(),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-python-service".to_string(),
            language: "python".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        layer: LayerType::Feature,
        domain: None,
        version: None,
        domain_version: None,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec![],
        protected_paths: vec![],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();

    // ディレクトリ作成
    for d in &["src", "config", "deploy/base", "deploy/docker"] {
        std::fs::create_dir_all(path.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileを除く）
    for f in &[
        "pyproject.toml",
        "README.md",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        // Dockerfile を意図的に除外
    ] {
        std::fs::write(path.join(f), "").unwrap();
    }

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K011エラーが発生するはず
    assert!(!result.is_success(), "Should have violations");
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        !dockerfile_violations.is_empty(),
        "Python backend feature layer should require Dockerfile"
    );
}

#[test]
fn test_backend_csharp_dockerfile_required() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let k1s0_dir = path.join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: "backend-csharp".to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: "CLI/templates/backend-csharp/feature".to_string(),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-csharp-service".to_string(),
            language: "csharp".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        layer: LayerType::Feature,
        domain: None,
        version: None,
        domain_version: None,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec![],
        protected_paths: vec![],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();

    // ディレクトリ作成
    for d in &["src", "config", "deploy/base", "deploy/docker"] {
        std::fs::create_dir_all(path.join(d)).unwrap();
    }

    // ファイル作成（Dockerfileを除く）
    for f in &[
        "README.md",
        "config/default.yaml",
        "config/dev.yaml",
        "config/stg.yaml",
        "config/prod.yaml",
        "buf.yaml",
        // Dockerfile を意図的に除外
    ] {
        std::fs::write(path.join(f), "").unwrap();
    }

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    // K011エラーが発生するはず
    assert!(!result.is_success(), "Should have violations");
    let dockerfile_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::RequiredFileMissing && v.message.contains("Dockerfile"))
        .collect();

    assert!(
        !dockerfile_violations.is_empty(),
        "C# backend feature layer should require Dockerfile"
    );
}
