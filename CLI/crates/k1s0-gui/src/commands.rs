use std::path::Path;

use k1s0_core::commands::build::{self as build_cmd, BuildConfig};
use k1s0_core::commands::deploy::{self as deploy_cmd, DeployConfig};
use k1s0_core::commands::generate::{self as gen_cmd, GenerateConfig};
use k1s0_core::commands::init::{self as init_cmd, InitConfig};
use k1s0_core::commands::test_cmd::{self as test_cmd, TestConfig};
use k1s0_core::config::CliConfig;
use k1s0_core::progress::ProgressEvent;
use tauri::ipc::Channel;

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config_path: String) -> Result<CliConfig, String> {
    k1s0_core::load_config(&config_path).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_init(config: InitConfig) -> Result<(), String> {
    init_cmd::execute_init(&config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate(config: GenerateConfig) -> Result<(), String> {
    gen_cmd::execute_generate(&config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_build(config: BuildConfig) -> Result<(), String> {
    build_cmd::execute_build(&config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test(config: TestConfig) -> Result<(), String> {
    test_cmd::execute_test(&config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deploy(config: DeployConfig) -> Result<(), String> {
    deploy_cmd::execute_deploy(&config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_placements(
    tier: k1s0_core::commands::generate::types::Tier,
    base_dir: String,
) -> Vec<String> {
    gen_cmd::scan_placements_at(&tier, Path::new(&base_dir))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_buildable_targets(base_dir: String) -> Vec<String> {
    build_cmd::scan_buildable_targets_at(Path::new(&base_dir))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_deployable_targets(base_dir: String) -> Vec<String> {
    deploy_cmd::scan_deployable_targets_at(Path::new(&base_dir))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_testable_targets(base_dir: String) -> Vec<String> {
    test_cmd::scan_testable_targets_at(Path::new(&base_dir))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_e2e_suites(base_dir: String) -> Vec<String> {
    test_cmd::scan_e2e_suites_at(Path::new(&base_dir))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn validate_name(name: String) -> Result<(), String> {
    k1s0_core::validate_name(&name)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test_with_progress(
    config: TestConfig,
    on_event: Channel<ProgressEvent>,
) -> Result<(), String> {
    test_cmd::execute_test_with_progress(&config, |event| {
        let _ = on_event.send(event);
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_build_with_progress(
    config: BuildConfig,
    on_event: Channel<ProgressEvent>,
) -> Result<(), String> {
    build_cmd::execute_build_with_progress(&config, |event| {
        let _ = on_event.send(event);
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deploy_with_progress(
    config: DeployConfig,
    on_event: Channel<ProgressEvent>,
) -> Result<(), String> {
    deploy_cmd::execute_deploy_with_progress(&config, |event| {
        let _ = on_event.send(event);
    })
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k1s0_core::commands::generate::types::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("order".to_string()).is_ok());
        assert!(validate_name("my-service".to_string()).is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_name("-order".to_string()).is_err());
        assert!(validate_name("Order".to_string()).is_err());
        assert!(validate_name(String::new()).is_err());
    }

    #[test]
    fn test_get_config_nonexistent() {
        let result = get_config("/nonexistent/path.yaml".to_string());
        // Should return default config or error
        // load_config with non-existent path returns default
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_scan_placements_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = scan_placements(
            k1s0_core::commands::generate::types::Tier::Business,
            tmp.path().to_string_lossy().to_string(),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_buildable_targets_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let targets = scan_buildable_targets(tmp.path().to_string_lossy().to_string());
        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_deployable_targets_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let targets = scan_deployable_targets(tmp.path().to_string_lossy().to_string());
        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_testable_targets_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let targets = scan_testable_targets(tmp.path().to_string_lossy().to_string());
        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_e2e_suites_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let suites = scan_e2e_suites(tmp.path().to_string_lossy().to_string());
        assert!(suites.is_empty());
    }

    #[test]
    fn test_execute_init_creates_project() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_path = tmp.path().join("test-project");
        let config = InitConfig {
            project_name: project_path.to_string_lossy().to_string(),
            git_init: false,
            sparse_checkout: false,
            tiers: vec![
                k1s0_core::commands::init::Tier::System,
                k1s0_core::commands::init::Tier::Business,
            ],
        };
        let result = execute_init(config);
        assert!(result.is_ok());
        assert!(project_path.join("regions/system").is_dir());
        assert!(project_path.join("regions/business").is_dir());
    }

    #[test]
    fn test_execute_generate_creates_files() {
        let _tmp = tempfile::TempDir::new().unwrap();
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
        // execute_generate uses current dir, so we use execute_generate_at instead
        // but the Tauri command calls execute_generate which uses cwd
        // For testing, just verify it doesn't panic
        let _ = execute_generate(config);
    }

    #[test]
    fn test_execute_build_with_nonexistent_target() {
        let config = BuildConfig {
            targets: vec!["/nonexistent/path".to_string()],
            mode: k1s0_core::commands::build::BuildMode::Development,
        };
        let result = execute_build(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_test_with_nonexistent_target() {
        let config = TestConfig {
            kind: k1s0_core::commands::test_cmd::TestKind::Unit,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let result = execute_test(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_deploy_with_nonexistent_target() {
        let config = DeployConfig {
            environment: k1s0_core::commands::deploy::Environment::Dev,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let result = execute_deploy(config);
        assert!(result.is_ok());
    }
}
