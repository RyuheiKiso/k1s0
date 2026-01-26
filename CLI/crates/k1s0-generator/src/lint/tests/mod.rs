use std::fs;
use std::path::Path;

fn create_test_manifest(dir: &Path, template_name: &str) {
    let k1s0_dir = dir.join(".k1s0");
    fs::create_dir_all(&k1s0_dir).unwrap();

    let manifest = crate::manifest::Manifest {
        schema_version: "1.0.0".to_string(),
        k1s0_version: "0.1.0".to_string(),
        template: crate::manifest::TemplateInfo {
            name: template_name.to_string(),
            version: "0.1.0".to_string(),
            source: "local".to_string(),
            path: format!("CLI/templates/{}/feature", template_name),
            revision: None,
            fingerprint: "abcd1234".to_string(),
        },
        service: crate::manifest::ServiceInfo {
            service_name: "test-service".to_string(),
            language: "rust".to_string(),
            service_type: "backend".to_string(),
            framework: None,
        },
        generated_at: "2026-01-26T10:00:00Z".to_string(),
        managed_paths: vec!["deploy/".to_string()],
        protected_paths: vec!["src/domain/".to_string()],
        update_policy: std::collections::HashMap::new(),
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    };

    manifest.save(k1s0_dir.join("manifest.json")).unwrap();
}

fn create_backend_rust_structure(dir: &Path) {
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
    ] {
        fs::create_dir_all(dir.join(d)).unwrap();
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
    ] {
        fs::write(dir.join(f), "").unwrap();
    }
}

mod dependency;
mod env_vars;
mod manifest;
mod required_files;
mod retry;
mod secret_config;
mod utils;
