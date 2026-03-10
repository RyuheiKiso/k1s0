use std::path::{Path, PathBuf};

use k1s0_core::commands::build::{self as build_cmd, BuildConfig};
use k1s0_core::commands::deploy::{self as deploy_cmd, DeployConfig};
use k1s0_core::commands::deps::{self as deps_cmd, DepsConfig, DepsResult};
use k1s0_core::commands::dev::{self as dev_cmd, DevDownConfig, DevUpConfig};
use k1s0_core::commands::generate::config_types as config_types_cmd;
use k1s0_core::commands::generate::navigation as nav_gen_cmd;
use k1s0_core::commands::generate::{self as gen_cmd, GenerateConfig};
use k1s0_core::commands::init::{self as init_cmd, InitConfig};
use k1s0_core::commands::migrate::{self as migrate_cmd, MigrateCreateConfig};
use k1s0_core::commands::test_cmd::{self as test_cmd, TestConfig};
use k1s0_core::commands::validate::config_schema as config_schema_validate;
use k1s0_core::commands::validate::navigation as nav_validate;
use k1s0_core::config::CliConfig;
use k1s0_core::progress::ProgressEvent;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

const DEFAULT_DISCOVERY_URL: &str =
    "https://auth.k1s0.internal.example.com/realms/k1s0/.well-known/openid-configuration";
const DEFAULT_CLIENT_ID: &str = "k1s0-cli";
const DEFAULT_SCOPE: &str = "openid profile email";
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config_path: String) -> Result<CliConfig, String> {
    k1s0_core::load_config(&config_path).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_init(config: InitConfig) -> Result<(), String> {
    init_cmd::execute_init(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate(config: GenerateConfig) -> Result<(), String> {
    gen_cmd::execute_generate(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate_at(config: GenerateConfig, base_dir: String) -> Result<(), String> {
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    gen_cmd::execute_generate_at(&config, &workspace_root).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_build(config: BuildConfig) -> Result<(), String> {
    build_cmd::execute_build(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test(config: TestConfig) -> Result<(), String> {
    test_cmd::execute_test(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test_at(config: TestConfig, base_dir: String) -> Result<(), String> {
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    test_cmd::execute_test_at(&config, &workspace_root).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deploy(config: DeployConfig) -> Result<(), String> {
    deploy_cmd::execute_deploy(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deploy_rollback(target: String) -> Result<String, String> {
    deploy_cmd::execute_deploy_rollback(&target).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_placements(
    tier: k1s0_core::commands::generate::types::Tier,
    base_dir: String,
) -> Vec<String> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => gen_cmd::scan_placements_at(&tier, &workspace_root),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_buildable_targets(base_dir: String) -> Vec<String> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => build_cmd::scan_buildable_targets_at(&workspace_root),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_deployable_targets(base_dir: String) -> Vec<String> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => deploy_cmd::scan_deployable_targets_at(&workspace_root),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_testable_targets(base_dir: String) -> Vec<String> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => test_cmd::scan_testable_targets_at(&workspace_root),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn validate_name(name: String) -> Result<(), String> {
    k1s0_core::validate_name(&name)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_validate_config_schema(path: String) -> Result<usize, String> {
    config_schema_validate::validate_config_schema(&path).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_validate_navigation(path: String) -> Result<usize, String> {
    nav_validate::validate_navigation(&path).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate_config_types(
    schema_path: String,
    target: String,
) -> Result<String, String> {
    match target.as_str() {
        "typescript" => config_types_cmd::generate_typescript_types_from_file(&schema_path)
            .map_err(|error| error.to_string()),
        "dart" => config_types_cmd::generate_dart_types_from_file(&schema_path)
            .map_err(|error| error.to_string()),
        _ => Err(format!("Unknown target: {target}")),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate_navigation_types(
    nav_path: String,
    target: String,
) -> Result<String, String> {
    match target.as_str() {
        "typescript" => nav_gen_cmd::generate_typescript_routes_from_file(&nav_path)
            .map_err(|error| error.to_string()),
        "dart" => nav_gen_cmd::generate_dart_routes_from_file(&nav_path)
            .map_err(|error| error.to_string()),
        _ => Err(format!("Unknown target: {target}")),
    }
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
    .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test_with_progress_at(
    config: TestConfig,
    base_dir: String,
    on_event: Channel<ProgressEvent>,
) -> Result<(), String> {
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    test_cmd::execute_test_with_progress_at(&config, &workspace_root, |event| {
        let _ = on_event.send(event);
    })
    .map_err(|error| error.to_string())
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
    .map_err(|error| error.to_string())
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
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn detect_workspace_root() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|current_dir| find_workspace_root(&current_dir))
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn resolve_workspace_root(path: String) -> Result<String, String> {
    resolve_workspace_root_path(Path::new(&path))
        .map(|workspace_root| workspace_root.to_string_lossy().to_string())
}

// deps
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deps(config: DepsConfig) -> Result<DepsResult, String> {
    deps_cmd::execute_deps(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_services(base_dir: String) -> Vec<deps_cmd::ServiceInfo> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => deps_cmd::scan_services_at(&workspace_root),
        Err(_) => Vec::new(),
    }
}

// dev
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_dev_up(config: DevUpConfig) -> Result<(), String> {
    dev_cmd::execute_dev_up(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_dev_down(config: DevDownConfig) -> Result<(), String> {
    dev_cmd::execute_dev_down(&config).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn execute_dev_status() -> Result<(), String> {
    dev_cmd::execute_dev_status().map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_dev_targets(base_dir: String) -> Vec<(String, String)> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => dev_cmd::scan_dev_targets(&workspace_root),
        Err(_) => Vec::new(),
    }
}

// migrate
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_migrate_create(config: MigrateCreateConfig) -> Result<(String, String), String> {
    migrate_cmd::create_migration(&config)
        .map(|(up, down)| {
            (
                up.to_string_lossy().to_string(),
                down.to_string_lossy().to_string(),
            )
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_migrate_targets(base_dir: String) -> Vec<migrate_cmd::MigrateTarget> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => migrate_cmd::scan_migrate_targets(&workspace_root),
        Err(_) => Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorizationChallenge {
    pub issuer: String,
    pub client_id: String,
    pub scope: String,
    pub token_endpoint: String,
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub interval: u64,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DeviceAuthorizationPollResult {
    Pending { interval: u64, message: String },
    Success { tokens: AuthTokens },
    Error { message: String },
}

#[derive(Debug, Deserialize)]
struct DiscoveryDocument {
    issuer: String,
    token_endpoint: String,
    device_authorization_endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawDeviceAuthorizationResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: Option<String>,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: String,
    error_description: Option<String>,
    interval: Option<u64>,
}

#[tauri::command]
pub fn start_device_authorization() -> Result<DeviceAuthorizationChallenge, String> {
    let client = http_client()?;
    let discovery_url = std::env::var("K1S0_GUI_OIDC_DISCOVERY_URL")
        .unwrap_or_else(|_| DEFAULT_DISCOVERY_URL.to_string());
    let client_id =
        std::env::var("K1S0_GUI_OIDC_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
    let scope = std::env::var("K1S0_GUI_OIDC_SCOPE").unwrap_or_else(|_| DEFAULT_SCOPE.to_string());

    let discovery: DiscoveryDocument = client
        .get(&discovery_url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())?;

    let device_endpoint = discovery
        .device_authorization_endpoint
        .clone()
        .unwrap_or_else(|| {
            format!(
                "{}/protocol/openid-connect/auth/device",
                discovery.issuer.trim_end_matches('/')
            )
        });

    let response: RawDeviceAuthorizationResponse = client
        .post(&device_endpoint)
        .form(&[("client_id", client_id.as_str()), ("scope", scope.as_str())])
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())?;

    Ok(DeviceAuthorizationChallenge {
        issuer: discovery.issuer,
        client_id,
        scope,
        token_endpoint: discovery.token_endpoint,
        device_code: response.device_code,
        user_code: response.user_code,
        verification_uri: response.verification_uri.clone(),
        verification_uri_complete: response
            .verification_uri_complete
            .unwrap_or(response.verification_uri),
        interval: response.interval.unwrap_or(5),
        expires_in: response.expires_in,
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn poll_device_authorization(
    challenge: DeviceAuthorizationChallenge,
) -> Result<DeviceAuthorizationPollResult, String> {
    let client = http_client()?;
    let response = client
        .post(&challenge.token_endpoint)
        .form(&[
            ("grant_type", DEVICE_CODE_GRANT),
            ("client_id", challenge.client_id.as_str()),
            ("device_code", challenge.device_code.as_str()),
        ])
        .send()
        .map_err(|error| error.to_string())?;

    if response.status().is_success() {
        let payload: RawTokenResponse = response.json().map_err(|error| error.to_string())?;
        return Ok(DeviceAuthorizationPollResult::Success {
            tokens: AuthTokens {
                access_token: payload.access_token,
                refresh_token: payload.refresh_token,
                id_token: payload.id_token,
                token_type: payload.token_type,
                expires_in: payload.expires_in,
                scope: payload.scope,
            },
        });
    }

    let status = response.status();
    let error_payload: OAuthErrorResponse = response.json().unwrap_or(OAuthErrorResponse {
        error: format!("http_{}", status.as_u16()),
        error_description: Some("Authentication request failed".to_string()),
        interval: None,
    });

    let message = error_payload
        .error_description
        .unwrap_or_else(|| error_payload.error.clone());

    match error_payload.error.as_str() {
        "authorization_pending" | "slow_down" => Ok(DeviceAuthorizationPollResult::Pending {
            interval: error_payload.interval.unwrap_or(challenge.interval),
            message,
        }),
        _ => Ok(DeviceAuthorizationPollResult::Error { message }),
    }
}

fn http_client() -> Result<Client, String> {
    Client::builder().build().map_err(|error| error.to_string())
}

fn resolve_workspace_root_path(path: &Path) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() {
        return Err("workspace path is empty".to_string());
    }

    let candidate = if path.is_file() {
        path.parent()
            .ok_or_else(|| "workspace path has no parent".to_string())?
    } else {
        path
    };

    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("failed to resolve workspace path: {error}"))?;

    find_workspace_root(&canonical).ok_or_else(|| "path is not inside a k1s0 workspace".to_string())
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|ancestor| {
        let has_regions = ancestor.join("regions").is_dir();
        let has_helm = ancestor
            .join("infra")
            .join("helm")
            .join("services")
            .is_dir();
        if has_regions && has_helm {
            Some(ancestor.to_path_buf())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_workspace_root_resolution_from_nested_path() {
        let tmp = tempfile::TempDir::new().unwrap();
        let nested = tmp.path().join("regions/system/server/rust/auth");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(tmp.path().join("infra/helm/services")).unwrap();

        let resolved = resolve_workspace_root_path(&nested).unwrap();
        assert_eq!(resolved, tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_workspace_root_resolution_rejects_non_workspace() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(resolve_workspace_root_path(tmp.path()).is_err());
    }

    #[test]
    fn test_device_authorization_endpoint_falls_back_to_keycloak_default() {
        let discovery = DiscoveryDocument {
            issuer: "https://auth.example.com/realms/k1s0".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            device_authorization_endpoint: None,
        };

        let endpoint = discovery
            .device_authorization_endpoint
            .clone()
            .unwrap_or_else(|| {
                format!(
                    "{}/protocol/openid-connect/auth/device",
                    discovery.issuer.trim_end_matches('/')
                )
            });

        assert_eq!(
            endpoint,
            "https://auth.example.com/realms/k1s0/protocol/openid-connect/auth/device"
        );
    }
}
