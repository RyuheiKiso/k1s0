// 競合検出のテスト。
// 既存アセットとの衝突が正しく検出・拒否されることを検証する。

use std::fs;
use tempfile::TempDir;

use crate::commands::generate::conflict::find_generate_conflicts_at;
use crate::commands::generate::execute::execute_generate_at;
use crate::commands::generate::types::{
    ApiStyle, DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Tier,
};

#[test]
fn test_find_generate_conflicts_at_detects_existing_output_directory() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("regions/system/library/rust/utils");
    fs::create_dir_all(&output_path).unwrap();

    let config = GenerateConfig {
        kind: Kind::Library,
        tier: Tier::System,
        placement: None,
        lang_fw: LangFw::Language(Language::Rust),
        detail: DetailConfig {
            name: Some("utils".to_string()),
            ..DetailConfig::default()
        },
    };

    let conflicts = find_generate_conflicts_at(&config, tmp.path());
    assert_eq!(
        conflicts,
        vec![output_path.to_string_lossy().replace('\\', "/")]
    );
}

#[test]
fn test_find_generate_conflicts_at_detects_server_helm_collision() {
    let tmp = TempDir::new().unwrap();
    let helm_path = tmp.path().join("infra/helm/services/system/auth");
    fs::create_dir_all(&helm_path).unwrap();

    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::System,
        placement: None,
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("auth".to_string()),
            api_styles: vec![ApiStyle::Rest],
            db: None,
            kafka: false,
            redis: false,
            bff_language: None,
        },
    };

    let conflicts = find_generate_conflicts_at(&config, tmp.path());
    assert_eq!(
        conflicts,
        vec![helm_path.to_string_lossy().replace('\\', "/")]
    );
}

#[test]
fn test_execute_generate_at_rejects_existing_conflicting_assets() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("regions/system/library/rust/utils");
    fs::create_dir_all(&output_path).unwrap();

    let config = GenerateConfig {
        kind: Kind::Library,
        tier: Tier::System,
        placement: None,
        lang_fw: LangFw::Language(Language::Rust),
        detail: DetailConfig {
            name: Some("utils".to_string()),
            ..DetailConfig::default()
        },
    };

    let result = execute_generate_at(&config, tmp.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("generated assets already exist"));
}

#[test]
fn test_execute_generate_at_allows_service_server_and_client_to_coexist() {
    let tmp = TempDir::new().unwrap();

    let server_config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Service,
        placement: Some("task".to_string()),
        lang_fw: LangFw::Language(Language::Rust),
        detail: DetailConfig {
            name: Some("task".to_string()),
            api_styles: vec![ApiStyle::Rest],
            db: None,
            kafka: false,
            redis: false,
            bff_language: None,
        },
    };
    execute_generate_at(&server_config, tmp.path()).unwrap();

    let client_config = GenerateConfig {
        kind: Kind::Client,
        tier: Tier::Service,
        placement: Some("task".to_string()),
        lang_fw: LangFw::Framework(Framework::React),
        detail: DetailConfig {
            name: Some("task".to_string()),
            ..DetailConfig::default()
        },
    };

    let result = execute_generate_at(&client_config, tmp.path());
    assert!(result.is_ok());
    assert!(tmp.path().join("regions/service/task/server/rust").is_dir());
    assert!(tmp
        .path()
        .join("regions/service/task/client/react")
        .is_dir());
    assert!(tmp
        .path()
        .join(".github/workflows/service-task-server-rust-ci.yaml")
        .is_file());
    assert!(tmp
        .path()
        .join(".github/workflows/service-task-client-react-ci.yaml")
        .is_file());
}
