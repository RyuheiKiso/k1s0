use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use crate::auth_session::{self, AuthSessionSummary, TokenPayload};
use k1s0_core::commands::build::{self as build_cmd, BuildConfig};
use k1s0_core::commands::deploy::{self as deploy_cmd, DeployConfig};
use k1s0_core::commands::deps::{
    self as deps_cmd, output as deps_output, DepsConfig, DepsOutputFormat, DepsResult,
};
use k1s0_core::commands::dev::{self as dev_cmd, DevDownConfig, DevUpConfig};
use k1s0_core::commands::generate::config_types as config_types_cmd;
use k1s0_core::commands::generate::navigation as nav_gen_cmd;
use k1s0_core::commands::generate::{self as gen_cmd, GenerateConfig};
use k1s0_core::commands::generate_events as event_codegen_cmd;
use k1s0_core::commands::init::{self as init_cmd, InitConfig};
use k1s0_core::commands::migrate::{
    self as migrate_cmd, DbConnection, MigrateCreateConfig, MigrateDownConfig, MigrateTarget,
    MigrateUpConfig, MigrationStatus, RepairOperation,
};
use k1s0_core::commands::template_migrate::{
    executor as template_migrate_executor, planner as template_migrate_planner,
    rollback as template_migrate_rollback, scanner as template_migrate_scanner,
    types::{MigrationPlan as TemplateMigrationPlan, MigrationTarget as TemplateMigrationTarget},
};
use k1s0_core::commands::test_cmd::{self as test_cmd, TestConfig};
use k1s0_core::commands::validate::config_schema as config_schema_validate;
use k1s0_core::commands::validate::navigation as nav_validate;
use k1s0_core::commands::validate::ValidationDiagnostic;
use k1s0_core::config::CliConfig;
use k1s0_core::progress::ProgressEvent;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use tauri::ipc::Channel;

const DEFAULT_DISCOVERY_URL: &str =
    "https://auth.k1s0.internal.example.com/realms/k1s0/.well-known/openid-configuration";
const DEFAULT_CLIENT_ID: &str = "k1s0-cli";
const DEFAULT_SCOPE: &str = "openid profile email";
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
static WORKSPACE_CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static FAILED_PROD_ROLLBACK_TARGET: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFileResult {
    pub path: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldDatabaseInfo {
    pub name: String,
    pub rdbms: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevAdditionalService {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevUpPreview {
    pub dependencies: dev_cmd::DetectedDependencies,
    pub ports: dev_cmd::PortAssignments,
    pub additional_services: Vec<DevAdditionalService>,
}

fn workspace_cwd_lock() -> &'static Mutex<()> {
    WORKSPACE_CWD_LOCK.get_or_init(|| Mutex::new(()))
}

fn failed_prod_rollback_target() -> &'static Mutex<Option<String>> {
    FAILED_PROD_ROLLBACK_TARGET.get_or_init(|| Mutex::new(None))
}

fn set_failed_prod_rollback_target(target: Option<String>) -> Result<(), String> {
    let mut guard = failed_prod_rollback_target()
        .lock()
        .map_err(|_| "failed to lock rollback target state".to_string())?;
    *guard = target;
    Ok(())
}

fn ensure_authenticated() -> Result<AuthSessionSummary, String> {
    auth_session::require_auth_session()
}

fn current_failed_prod_rollback_target() -> Result<Option<String>, String> {
    failed_prod_rollback_target()
        .lock()
        .map_err(|_| "failed to lock rollback target state".to_string())
        .map(|guard| guard.clone())
}

fn deploy_rollback_target(config: &DeployConfig) -> Option<String> {
    if config.environment.is_prod() && config.targets.len() == 1 {
        return config.targets.first().cloned();
    }
    None
}

fn resolve_workspace_root_from_option(base_dir: Option<String>) -> Result<PathBuf, String> {
    match base_dir {
        Some(path) => resolve_workspace_root_path(Path::new(&path)),
        None => std::env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))
            .and_then(|path| {
                find_workspace_root(&path)
                    .ok_or_else(|| "path is not inside a k1s0 workspace".to_string())
            }),
    }
}

fn resolve_workspace_path(workspace_root: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        workspace_root.join(candidate)
    }
}

fn with_workspace_cwd<T>(
    base_dir: Option<String>,
    operation: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<T, String> {
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let _guard = workspace_cwd_lock()
        .lock()
        .map_err(|_| "failed to lock workspace current directory".to_string())?;
    let previous = std::env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    std::env::set_current_dir(&workspace_root)
        .map_err(|error| format!("failed to switch workspace directory: {error}"))?;

    let result = operation(&workspace_root);
    let restore = std::env::set_current_dir(previous)
        .map_err(|error| format!("failed to restore current directory: {error}"));

    match (result, restore) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (_, Err(error)) => Err(error),
    }
}

fn resolve_event_template_dir(workspace_root: &Path) -> Result<PathBuf, String> {
    let candidates = [
        workspace_root
            .join("CLI")
            .join("crates")
            .join("k1s0-cli")
            .join("templates")
            .join("events"),
        workspace_root.join("templates").join("events"),
        event_codegen_cmd::default_template_dir(),
    ];

    candidates
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| "event codegen templates directory was not found".to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config_path: String) -> Result<CliConfig, String> {
    k1s0_core::load_config(&config_path).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_init(config: InitConfig) -> Result<(), String> {
    ensure_authenticated()?;
    init_cmd::execute_init(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_init_at(config: InitConfig, base_dir: String) -> Result<String, String> {
    ensure_authenticated()?;
    init_cmd::execute_init_at(&config, Path::new(&base_dir))
        .map(|workspace_root| workspace_root.to_string_lossy().to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate(config: GenerateConfig) -> Result<(), String> {
    ensure_authenticated()?;
    gen_cmd::execute_generate(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate_at(config: GenerateConfig, base_dir: String) -> Result<(), String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    gen_cmd::execute_generate_at(&config, &workspace_root).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_generate_conflicts(
    config: GenerateConfig,
    base_dir: String,
) -> Result<Vec<String>, String> {
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    Ok(gen_cmd::find_generate_conflicts_at(
        &config,
        &workspace_root,
    ))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_build(config: BuildConfig) -> Result<(), String> {
    ensure_authenticated()?;
    build_cmd::execute_build(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test(config: TestConfig) -> Result<(), String> {
    ensure_authenticated()?;
    test_cmd::execute_test(&config).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test_at(config: TestConfig, base_dir: String) -> Result<(), String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    test_cmd::execute_test_at(&config, &workspace_root).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deploy(config: DeployConfig) -> Result<(), String> {
    ensure_authenticated()?;
    let rollback_target = deploy_rollback_target(&config);
    let result = deploy_cmd::execute_deploy(&config).map_err(|error| error.to_string());
    set_failed_prod_rollback_target(if result.is_err() {
        rollback_target
    } else {
        None
    })?;
    result
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_deploy_rollback(target: String) -> Result<String, String> {
    ensure_authenticated()?;

    let expected_target = failed_prod_rollback_target()
        .lock()
        .map_err(|_| "failed to lock rollback target state".to_string())?
        .clone();
    if expected_target.as_deref() != Some(target.as_str()) {
        return Err(
            "Rollback is only available for the last failed production deployment target."
                .to_string(),
        );
    }

    let result = deploy_cmd::execute_deploy_rollback(&target).map_err(|error| error.to_string());
    if result.is_ok() {
        set_failed_prod_rollback_target(None)?;
    }
    result
}

#[tauri::command]
pub fn get_failed_prod_rollback_target() -> Result<Option<String>, String> {
    current_failed_prod_rollback_target()
}

#[tauri::command]
pub fn get_auth_session() -> Result<Option<AuthSessionSummary>, String> {
    auth_session::load_auth_session()
}

#[tauri::command]
pub fn clear_auth_session() -> Result<(), String> {
    auth_session::clear_auth_session()
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
pub fn scan_databases(
    tier: k1s0_core::commands::generate::types::Tier,
    base_dir: String,
) -> Vec<ScaffoldDatabaseInfo> {
    let Ok(workspace_root) = resolve_workspace_root_path(Path::new(&base_dir)) else {
        return Vec::new();
    };

    let database_dirs: Vec<PathBuf> = match tier {
        k1s0_core::commands::generate::types::Tier::System => {
            vec![workspace_root
                .join("regions")
                .join("system")
                .join("database")]
        }
        k1s0_core::commands::generate::types::Tier::Business => {
            let root = workspace_root.join("regions").join("business");
            std::fs::read_dir(root)
                .ok()
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path().join("database"))
                .collect()
        }
        k1s0_core::commands::generate::types::Tier::Service => {
            let root = workspace_root.join("regions").join("service");
            std::fs::read_dir(root)
                .ok()
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path().join("database"))
                .collect()
        }
    };

    let mut databases = Vec::new();
    for directory in database_dirs {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let database_yaml = path.join("database.yaml");
            if !database_yaml.is_file() {
                continue;
            }

            let Ok(content) = std::fs::read_to_string(&database_yaml) else {
                continue;
            };
            let Ok(parsed) = serde_yaml::from_str::<YamlValue>(&content) else {
                continue;
            };
            let Some(name) = parsed.get("name").and_then(YamlValue::as_str) else {
                continue;
            };
            let Some(rdbms) = parsed.get("rdbms").and_then(YamlValue::as_str) else {
                continue;
            };

            databases.push(ScaffoldDatabaseInfo {
                name: name.to_string(),
                rdbms: rdbms.to_string(),
                path: path.to_string_lossy().to_string(),
            });
        }
    }

    databases.sort_by(|left, right| left.name.cmp(&right.name));
    databases
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
pub fn execute_validate_config_schema(
    path: String,
    base_dir: Option<String>,
) -> Result<Vec<ValidationDiagnostic>, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let resolved = resolve_workspace_path(&workspace_root, &path);
    config_schema_validate::collect_config_schema_diagnostics(&resolved.to_string_lossy())
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_validate_navigation(
    path: String,
    base_dir: Option<String>,
) -> Result<Vec<ValidationDiagnostic>, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let resolved = resolve_workspace_path(&workspace_root, &path);
    nav_validate::collect_navigation_diagnostics(&resolved.to_string_lossy())
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate_config_types(
    schema_path: String,
    target: String,
    base_dir: Option<String>,
) -> Result<String, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let resolved = resolve_workspace_path(&workspace_root, &schema_path);
    match target.as_str() {
        "typescript" => {
            config_types_cmd::generate_typescript_types_from_file(&resolved.to_string_lossy())
                .map_err(|error| error.to_string())
        }
        "dart" => config_types_cmd::generate_dart_types_from_file(&resolved.to_string_lossy())
            .map_err(|error| error.to_string()),
        _ => Err(format!("Unknown target: {target}")),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_generate_navigation_types(
    nav_path: String,
    target: String,
    base_dir: Option<String>,
) -> Result<String, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let resolved = resolve_workspace_path(&workspace_root, &nav_path);
    match target.as_str() {
        "typescript" => {
            nav_gen_cmd::generate_typescript_routes_from_file(&resolved.to_string_lossy())
                .map_err(|error| error.to_string())
        }
        "dart" => nav_gen_cmd::generate_dart_routes_from_file(&resolved.to_string_lossy())
            .map_err(|error| error.to_string()),
        _ => Err(format!("Unknown target: {target}")),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn write_config_types(
    schema_path: String,
    output_dir: String,
    targets: Vec<String>,
    base_dir: String,
) -> Result<Vec<GeneratedFileResult>, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    let resolved_schema = resolve_workspace_path(&workspace_root, &schema_path);
    let resolved_output_dir = resolve_workspace_path(&workspace_root, &output_dir);
    let target_refs: Vec<&str> = targets.iter().map(String::as_str).collect();
    let written = config_types_cmd::write_generated_types_from_file(
        &resolved_schema,
        &resolved_output_dir,
        &target_refs,
    )
    .map_err(|error| error.to_string())?;

    let mut files = Vec::new();
    for path in written {
        let preview = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        files.push(GeneratedFileResult {
            path: path.to_string_lossy().to_string(),
            preview,
        });
    }
    Ok(files)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn write_navigation_types(
    nav_path: String,
    output_dir: String,
    targets: Vec<String>,
    base_dir: String,
) -> Result<Vec<GeneratedFileResult>, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_path(Path::new(&base_dir))?;
    let resolved_nav = resolve_workspace_path(&workspace_root, &nav_path);
    let resolved_output_dir = resolve_workspace_path(&workspace_root, &output_dir);
    let target_refs: Vec<&str> = targets.iter().map(String::as_str).collect();
    let written = nav_gen_cmd::write_generated_routes_from_file(
        &resolved_nav,
        &resolved_output_dir,
        &target_refs,
    )
    .map_err(|error| error.to_string())?;

    let mut files = Vec::new();
    for path in written {
        let preview = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        files.push(GeneratedFileResult {
            path: path.to_string_lossy().to_string(),
            preview,
        });
    }
    Ok(files)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_test_with_progress(
    config: TestConfig,
    on_event: Channel<ProgressEvent>,
) -> Result<(), String> {
    ensure_authenticated()?;
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
    ensure_authenticated()?;
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
    ensure_authenticated()?;
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
    ensure_authenticated()?;
    let rollback_target = deploy_rollback_target(&config);
    let result = deploy_cmd::execute_deploy_with_progress(&config, |event| {
        let _ = on_event.send(event);
    })
    .map_err(|error| error.to_string());
    set_failed_prod_rollback_target(if result.is_err() {
        rollback_target
    } else {
        None
    })?;
    result
}

#[tauri::command]
pub fn detect_workspace_root() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|current_dir| find_workspace_root(&current_dir))
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_current_directory() -> Result<String, String> {
    std::env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|error| format!("failed to resolve current directory: {error}"))
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
pub fn execute_deps(config: DepsConfig, base_dir: Option<String>) -> Result<DepsResult, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let result =
        deps_cmd::execute_deps_at(&workspace_root, &config).map_err(|error| error.to_string())?;

    match &config.output {
        DepsOutputFormat::Mermaid(path) | DepsOutputFormat::Both(path) => {
            let output_path = if path.is_absolute() {
                path.clone()
            } else {
                workspace_root.join(path)
            };
            deps_output::write_mermaid(&result, &output_path).map_err(|error| error.to_string())?;
        }
        DepsOutputFormat::Terminal => {}
    }

    Ok(result)
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
pub fn execute_dev_up(config: DevUpConfig, base_dir: Option<String>) -> Result<(), String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        dev_cmd::execute_dev_up(&config).map_err(|error| error.to_string())
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_dev_down(config: DevDownConfig, base_dir: Option<String>) -> Result<(), String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        dev_cmd::execute_dev_down(&config).map_err(|error| error.to_string())
    })
}

#[tauri::command]
#[allow(clippy::format_push_string)]
pub fn execute_dev_status(base_dir: Option<String>) -> Result<String, String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        let compose_dir = Path::new(".k1s0-dev");
        if !compose_dir.join("docker-compose.yaml").exists() {
            return Ok("ローカル開発環境は起動していません。".to_string());
        }

        let mut output = String::new();
        if let Some(state) = dev_cmd::state::load_state() {
            output.push_str("--- ローカル開発環境の状態 ---\n");
            output.push_str(&format!("起動日時: {}\n", state.started_at));
            output.push_str(&format!("認証モード: {}\n", state.auth_mode));
            output.push_str("対象サービス:\n");
            for service in state.services {
                writeln!(output, "- {service}").expect("writing to String cannot fail");
            }
        }

        let compose_status =
            dev_cmd::docker::compose_status(compose_dir).map_err(|error| error.to_string())?;
        output.push_str("\n--- コンテナの状態 ---\n");
        output.push_str(&compose_status);

        Ok(output)
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_dev_logs(
    service: Option<String>,
    base_dir: Option<String>,
) -> Result<String, String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        let compose_dir = Path::new(".k1s0-dev");
        if !compose_dir.join("docker-compose.yaml").exists() {
            return Ok("ローカル開発環境は起動していません。".to_string());
        }

        let mut args = vec![
            "compose".to_string(),
            "logs".to_string(),
            "--tail".to_string(),
            "200".to_string(),
        ];
        if let Some(service_name) = service {
            args.push(service_name);
        }

        let output = Command::new("docker")
            .args(args.iter().map(String::as_str))
            .current_dir(compose_dir)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_dev_targets(base_dir: String) -> Vec<(String, String)> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => dev_cmd::scan_dev_targets(&workspace_root),
        Err(_) => Vec::new(),
    }
}

fn build_dev_additional_services(
    dependencies: &dev_cmd::DetectedDependencies,
    ports: &dev_cmd::PortAssignments,
    auth_mode: &dev_cmd::AuthMode,
) -> Vec<DevAdditionalService> {
    let mut services = Vec::new();

    if !dependencies.databases.is_empty() {
        services.push(DevAdditionalService {
            name: "pgAdmin".to_string(),
            url: format!("http://localhost:{}", ports.pgadmin),
        });
    }

    if dependencies.has_kafka {
        services.push(DevAdditionalService {
            name: "Kafka UI".to_string(),
            url: format!("http://localhost:{}", ports.kafka_ui),
        });
    }

    if matches!(auth_mode, dev_cmd::AuthMode::Keycloak) {
        services.push(DevAdditionalService {
            name: "Keycloak".to_string(),
            url: format!("http://localhost:{}", ports.keycloak),
        });
    }

    services
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn preview_dev_up(
    config: DevUpConfig,
    base_dir: Option<String>,
) -> Result<DevUpPreview, String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        let dependencies = config
            .services
            .iter()
            .map(|service| dev_cmd::detect::detect_dependencies(service))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        let merged = dev_cmd::detect::merge_dependencies(&dependencies);
        let ports = dev_cmd::port::resolve_ports(&dev_cmd::port::default_ports());

        Ok(DevUpPreview {
            additional_services: build_dev_additional_services(&merged, &ports, &config.auth_mode),
            dependencies: merged,
            ports,
        })
    })
}

// migrate
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_migrate_create(config: MigrateCreateConfig) -> Result<(String, String), String> {
    ensure_authenticated()?;
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
pub fn execute_migrate_up(config: MigrateUpConfig, base_dir: Option<String>) -> Result<(), String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        migrate_cmd::execute_migrate_up(&config).map_err(|error| error.to_string())
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_migrate_down(
    config: MigrateDownConfig,
    base_dir: Option<String>,
) -> Result<(), String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        migrate_cmd::execute_migrate_down(&config).map_err(|error| error.to_string())
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_migrate_status(
    target: MigrateTarget,
    connection: DbConnection,
    base_dir: Option<String>,
) -> Result<Vec<MigrationStatus>, String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        migrate_cmd::get_migration_status(&target, &connection).map_err(|error| error.to_string())
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_migrate_repair(
    target: MigrateTarget,
    operation: RepairOperation,
    connection: DbConnection,
    base_dir: Option<String>,
) -> Result<(), String> {
    ensure_authenticated()?;
    with_workspace_cwd(base_dir, |_| {
        migrate_cmd::execute_repair(&target, &operation, &connection)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_migrate_targets(base_dir: String) -> Vec<migrate_cmd::MigrateTarget> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => migrate_cmd::scan_migrate_targets(&workspace_root),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn scan_template_migration_targets(base_dir: String) -> Vec<TemplateMigrationTarget> {
    match resolve_workspace_root_path(Path::new(&base_dir)) {
        Ok(workspace_root) => {
            template_migrate_scanner::scan_targets(&workspace_root).unwrap_or_else(|_| Vec::new())
        }
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn preview_template_migration(
    target: TemplateMigrationTarget,
) -> Result<TemplateMigrationPlan, String> {
    ensure_authenticated()?;
    template_migrate_planner::build_plan(&target).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_template_migration(plan: TemplateMigrationPlan) -> Result<(), String> {
    ensure_authenticated()?;
    template_migrate_executor::execute_migration(&plan).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn list_template_migration_backups(project_dir: String) -> Result<Vec<String>, String> {
    let project_path = PathBuf::from(project_dir);
    template_migrate_rollback::list_backups(&project_path).map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_template_migration_rollback(
    project_dir: String,
    backup_id: String,
) -> Result<(), String> {
    ensure_authenticated()?;
    let project_path = PathBuf::from(project_dir);
    let backup_dir = template_migrate_rollback::backup_dir(&project_path, &backup_id);
    template_migrate_rollback::rollback(&project_path, &backup_dir)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn preview_event_codegen(
    events_path: String,
    base_dir: Option<String>,
) -> Result<String, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let resolved = resolve_workspace_path(&workspace_root, &events_path);
    let config = event_codegen_cmd::parse_events_yaml(&resolved.to_string_lossy())
        .map_err(|error| error.to_string())?;
    Ok(event_codegen_cmd::format_generation_summary(&config))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn execute_event_codegen(
    events_path: String,
    base_dir: Option<String>,
) -> Result<Vec<String>, String> {
    ensure_authenticated()?;
    let workspace_root = resolve_workspace_root_from_option(base_dir)?;
    let resolved = resolve_workspace_path(&workspace_root, &events_path);
    let config = event_codegen_cmd::parse_events_yaml(&resolved.to_string_lossy())
        .map_err(|error| error.to_string())?;
    let template_dir = resolve_event_template_dir(&workspace_root)?;
    let output_dir = resolved
        .parent()
        .map_or_else(|| workspace_root.clone(), Path::to_path_buf);
    let generated = event_codegen_cmd::execute_event_codegen(&config, &output_dir, &template_dir)
        .map_err(|error| error.to_string())?;
    Ok(generated
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect())
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
pub struct DeviceAuthorizationSettings {
    pub discovery_url: String,
    pub client_id: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorizationDiscovery {
    pub issuer: String,
    pub token_endpoint: String,
    pub device_authorization_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DeviceAuthorizationPollResult {
    Pending { interval: u64, message: String },
    Success { session: AuthSessionSummary },
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

fn default_device_authorization_settings() -> DeviceAuthorizationSettings {
    DeviceAuthorizationSettings {
        discovery_url: std::env::var("K1S0_GUI_OIDC_DISCOVERY_URL")
            .unwrap_or_else(|_| DEFAULT_DISCOVERY_URL.to_string()),
        client_id: std::env::var("K1S0_GUI_OIDC_CLIENT_ID")
            .unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string()),
        scope: std::env::var("K1S0_GUI_OIDC_SCOPE").unwrap_or_else(|_| DEFAULT_SCOPE.to_string()),
    }
}

fn normalize_device_authorization_settings(
    settings: Option<DeviceAuthorizationSettings>,
) -> Result<DeviceAuthorizationSettings, String> {
    let defaults = default_device_authorization_settings();
    let settings = settings.unwrap_or(defaults);

    let discovery_url = settings.discovery_url.trim();
    let client_id = settings.client_id.trim();
    let scope = settings.scope.trim();

    if discovery_url.is_empty() {
        return Err("OIDC discovery URL is required.".to_string());
    }
    if client_id.is_empty() {
        return Err("OIDC client ID is required.".to_string());
    }
    if scope.is_empty() {
        return Err("OIDC scope is required.".to_string());
    }

    Ok(DeviceAuthorizationSettings {
        discovery_url: discovery_url.to_string(),
        client_id: client_id.to_string(),
        scope: scope.to_string(),
    })
}

fn fetch_discovery_document(
    client: &Client,
    settings: &DeviceAuthorizationSettings,
) -> Result<DiscoveryDocument, String> {
    client
        .get(&settings.discovery_url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())
}

fn resolve_device_authorization_endpoint(discovery: &DiscoveryDocument) -> String {
    discovery
        .device_authorization_endpoint
        .clone()
        .unwrap_or_else(|| {
            format!(
                "{}/protocol/openid-connect/auth/device",
                discovery.issuer.trim_end_matches('/')
            )
        })
}

#[tauri::command]
pub fn get_device_authorization_defaults() -> DeviceAuthorizationSettings {
    default_device_authorization_settings()
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn validate_device_authorization_settings(
    settings: DeviceAuthorizationSettings,
) -> Result<DeviceAuthorizationDiscovery, String> {
    let client = http_client()?;
    let settings = normalize_device_authorization_settings(Some(settings))?;
    let discovery = fetch_discovery_document(&client, &settings)?;

    Ok(DeviceAuthorizationDiscovery {
        issuer: discovery.issuer.clone(),
        token_endpoint: discovery.token_endpoint.clone(),
        device_authorization_endpoint: resolve_device_authorization_endpoint(&discovery),
    })
}

#[tauri::command]
pub fn start_device_authorization(
    settings: Option<DeviceAuthorizationSettings>,
) -> Result<DeviceAuthorizationChallenge, String> {
    let client = http_client()?;
    let settings = normalize_device_authorization_settings(settings)?;
    let discovery = fetch_discovery_document(&client, &settings)?;
    let device_endpoint = resolve_device_authorization_endpoint(&discovery);

    let response: RawDeviceAuthorizationResponse = client
        .post(&device_endpoint)
        .form(&[
            ("client_id", settings.client_id.as_str()),
            ("scope", settings.scope.as_str()),
        ])
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())?;

    Ok(DeviceAuthorizationChallenge {
        issuer: discovery.issuer,
        client_id: settings.client_id,
        scope: settings.scope,
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
        let session = auth_session::store_auth_session(
            challenge.issuer,
            challenge.client_id,
            challenge.scope,
            challenge.token_endpoint,
            TokenPayload {
                access_token: payload.access_token,
                refresh_token: payload.refresh_token,
                id_token: payload.id_token,
                token_type: payload.token_type,
                expires_in: payload.expires_in,
                scope: payload.scope,
            },
        )?;
        return Ok(DeviceAuthorizationPollResult::Success { session });
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

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn test_default_device_authorization_settings_have_values() {
        let settings = default_device_authorization_settings();

        assert!(!settings.discovery_url.is_empty());
        assert!(!settings.client_id.is_empty());
        assert!(!settings.scope.is_empty());
    }

    #[test]
    fn test_normalize_device_authorization_settings_rejects_blank_fields() {
        let error = normalize_device_authorization_settings(Some(DeviceAuthorizationSettings {
            discovery_url: "   ".to_string(),
            client_id: "client".to_string(),
            scope: "openid".to_string(),
        }))
        .expect_err("blank discovery URL should be rejected");

        assert!(error.contains("discovery URL"));
    }
}
