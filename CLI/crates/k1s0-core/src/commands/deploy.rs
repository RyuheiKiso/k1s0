use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::CliConfig;
use crate::progress::ProgressEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStep {
    DockerBuild,
    DockerPush,
    CosignSign,
    HelmDeploy,
}

impl DeployStep {
    pub fn label(&self) -> &'static str {
        match self {
            DeployStep::DockerBuild => "Docker image build",
            DeployStep::DockerPush => "Docker image push",
            DeployStep::CosignSign => "Cosign image signing",
            DeployStep::HelmDeploy => "Helm deploy",
        }
    }

    pub fn step_number(&self) -> usize {
        match self {
            DeployStep::DockerBuild => 1,
            DeployStep::DockerPush => 2,
            DeployStep::CosignSign => 3,
            DeployStep::HelmDeploy => 4,
        }
    }
}

pub const TOTAL_DEPLOY_STEPS: usize = 4;

pub fn build_image_tag(
    registry: &str,
    tier: &str,
    service_name: &str,
    version: &str,
    sha: &str,
) -> String {
    format!("{registry}/k1s0-{tier}/{service_name}:{version}-{sha}")
}

pub fn build_helm_args(
    service_name: &str,
    helm_path: &str,
    tier: &str,
    env: &str,
    image_tag: &str,
) -> Vec<String> {
    vec![
        "upgrade".to_string(),
        "--install".to_string(),
        service_name.to_string(),
        format!("./infra/helm/services/{helm_path}"),
        "-n".to_string(),
        format!("k1s0-{tier}"),
        "-f".to_string(),
        format!("./infra/helm/services/{helm_path}/values-{env}.yaml"),
        "--set".to_string(),
        format!("image.tag={image_tag}"),
    ]
}

pub fn build_helm_rollback_args(service_name: &str, tier: &str) -> Vec<String> {
    vec![
        "rollback".to_string(),
        service_name.to_string(),
        "-n".to_string(),
        format!("k1s0-{tier}"),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployError {
    pub step: DeployStep,
    pub message: String,
    pub manual_command: String,
}

pub fn extract_tier_from_target_path(target: &str) -> Option<String> {
    let normalized = target.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        if *part == "regions" && index + 1 < parts.len() {
            let tier = parts[index + 1];
            if matches!(tier, "system" | "business" | "service") {
                return Some(tier.to_string());
            }
        }
    }
    None
}

pub fn extract_service_name_from_target_path(target: &str) -> Option<String> {
    let normalized = target.replace('\\', "/");
    let parts: Vec<&str> = normalized.trim_end_matches('/').split('/').collect();
    let regions_idx = parts.iter().position(|part| *part == "regions")?;
    let tier = *parts.get(regions_idx + 1)?;

    match tier {
        "service" => parts.get(regions_idx + 2).map(|part| (*part).to_string()),
        "system" | "business" => parts.last().map(|part| (*part).to_string()),
        _ => None,
    }
}

pub fn format_deploy_success(env: &str, service_name: &str, image_tag: &str, tier: &str) -> String {
    format!(
        "Deploy completed\n  Environment: {env}\n  Service: {service_name}\n  Image: {image_tag}\n  Helm: helm status {service_name} -n k1s0-{tier}"
    )
}

pub fn format_deploy_failure(error: &DeployError) -> String {
    format!(
        "Deploy failed\n  Step: {}\n  Error: {}\n  Manual retry: {}",
        error.step.label(),
        error.message,
        error.manual_command
    )
}

pub fn format_step_start(step: &DeployStep) -> String {
    format!(
        "[{}/{}] {}...",
        step.step_number(),
        TOTAL_DEPLOY_STEPS,
        step.label()
    )
}

pub fn format_step_done(step: &DeployStep) -> String {
    format!(
        "[{}/{}] done: {}",
        step.step_number(),
        TOTAL_DEPLOY_STEPS,
        step.label()
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Dev,
    Staging,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Prod => "prod",
        }
    }

    pub fn is_prod(&self) -> bool {
        matches!(self, Environment::Prod)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployConfig {
    pub environment: Environment,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeployPlan {
    environment: Environment,
    workspace_root: PathBuf,
    target_path: PathBuf,
    module_path: String,
    tier: String,
    service_name: String,
    helm_path: String,
    version: String,
    sha: String,
    image_tag_suffix: String,
    full_image_tag: String,
}

/// Execute deployment for the selected targets.
///
/// # Errors
///
/// Returns an error when target validation fails or any deployment command fails.
pub fn execute_deploy(config: &DeployConfig) -> Result<()> {
    execute_deploy_internal(config, Option::<&fn(ProgressEvent)>::None)
}

/// Execute deployment for the selected targets while emitting progress events.
///
/// # Errors
///
/// Returns an error when target validation fails or any deployment command fails.
pub fn execute_deploy_with_progress(
    config: &DeployConfig,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    execute_deploy_internal(config, Some(&on_progress))
}

/// Roll back the most recent production Helm release for a target.
///
/// # Errors
///
/// Returns an error when the deploy plan cannot be resolved or the Helm command fails.
pub fn execute_deploy_rollback(target: &str) -> Result<String> {
    let plan = build_deploy_plan(Path::new(target), Environment::Prod)?;
    let args = build_helm_rollback_args(&plan.service_name, &plan.tier);
    run_checked_command("helm", &args, &plan.workspace_root)?;
    Ok(format!(
        "Rollback completed for {} in k1s0-{}",
        plan.service_name, plan.tier
    ))
}

fn execute_deploy_internal<F>(config: &DeployConfig, on_progress: Option<&F>) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    if config.targets.is_empty() {
        emit(
            on_progress,
            ProgressEvent::Finished {
                success: false,
                message: "No deploy targets selected".to_string(),
            },
        );
        bail!("no deploy targets selected");
    }

    let total = config.targets.len();

    for (index, target) in config.targets.iter().enumerate() {
        let step = index + 1;
        emit(
            on_progress,
            ProgressEvent::StepStarted {
                step,
                total,
                message: format!("Deploying {target} to {}", config.environment.as_str()),
            },
        );

        let plan = match build_deploy_plan(Path::new(target), config.environment) {
            Ok(plan) => plan,
            Err(error) => {
                let message = error.to_string();
                emit(
                    on_progress,
                    ProgressEvent::Error {
                        message: message.clone(),
                    },
                );
                emit(
                    on_progress,
                    ProgressEvent::Finished {
                        success: false,
                        message: "Deploy failed".to_string(),
                    },
                );
                return Err(error);
            }
        };

        if let Err(error) = execute_deploy_plan(&plan, on_progress) {
            let message = format_deploy_failure(&error);
            emit(
                on_progress,
                ProgressEvent::Error {
                    message: message.clone(),
                },
            );
            if plan.environment.is_prod() {
                let rollback = build_helm_rollback_args(&plan.service_name, &plan.tier);
                emit(
                    on_progress,
                    ProgressEvent::Warning {
                        message: format!("Rollback available: helm {}", rollback.join(" ")),
                    },
                );
            }
            emit(
                on_progress,
                ProgressEvent::Finished {
                    success: false,
                    message: "Deploy failed".to_string(),
                },
            );
            return Err(anyhow!(message));
        }

        emit(
            on_progress,
            ProgressEvent::StepCompleted {
                step,
                total,
                message: format!("Deployed {}", plan.module_path),
            },
        );
    }

    emit(
        on_progress,
        ProgressEvent::Finished {
            success: true,
            message: "Deploy completed".to_string(),
        },
    );
    Ok(())
}

fn execute_deploy_plan<F>(
    plan: &DeployPlan,
    on_progress: Option<&F>,
) -> std::result::Result<(), DeployError>
where
    F: Fn(ProgressEvent),
{
    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_start(&DeployStep::DockerBuild),
        },
    );
    run_checked_command(
        "docker",
        &[
            "build".to_string(),
            "-t".to_string(),
            plan.full_image_tag.clone(),
            ".".to_string(),
        ],
        &plan.target_path,
    )
    .map_err(|error| build_error(plan, DeployStep::DockerBuild, &error))?;
    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_done(&DeployStep::DockerBuild),
        },
    );

    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_start(&DeployStep::DockerPush),
        },
    );
    run_checked_command(
        "docker",
        &["push".to_string(), plan.full_image_tag.clone()],
        &plan.target_path,
    )
    .map_err(|error| build_error(plan, DeployStep::DockerPush, &error))?;
    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_done(&DeployStep::DockerPush),
        },
    );

    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_start(&DeployStep::CosignSign),
        },
    );
    run_checked_command(
        "cosign",
        &[
            "sign".to_string(),
            "--yes".to_string(),
            plan.full_image_tag.clone(),
        ],
        &plan.workspace_root,
    )
    .map_err(|error| build_error(plan, DeployStep::CosignSign, &error))?;
    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_done(&DeployStep::CosignSign),
        },
    );

    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_start(&DeployStep::HelmDeploy),
        },
    );
    let helm_args = build_helm_args(
        &plan.service_name,
        &plan.helm_path,
        &plan.tier,
        plan.environment.as_str(),
        &plan.image_tag_suffix,
    );
    run_checked_command("helm", &helm_args, &plan.workspace_root)
        .map_err(|error| build_error(plan, DeployStep::HelmDeploy, &error))?;
    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_step_done(&DeployStep::HelmDeploy),
        },
    );

    emit(
        on_progress,
        ProgressEvent::Log {
            message: format_deploy_success(
                plan.environment.as_str(),
                &plan.service_name,
                &plan.full_image_tag,
                &plan.tier,
            ),
        },
    );

    Ok(())
}

fn build_deploy_plan(target_path: &Path, environment: Environment) -> Result<DeployPlan> {
    if !target_path.is_dir() {
        bail!("target directory does not exist");
    }
    if !target_path.join("Dockerfile").exists() {
        bail!("target does not contain a Dockerfile");
    }

    let workspace_root = find_workspace_root(target_path).ok_or_else(|| {
        anyhow!(
            "failed to locate workspace root for {}",
            target_path.display()
        )
    })?;
    let module_path = to_relative_path(&workspace_root, target_path)?;
    let tier = extract_tier_from_target_path(&module_path)
        .ok_or_else(|| anyhow!("failed to determine tier from {module_path}"))?;
    let service_name = extract_service_name_from_target_path(&module_path)
        .ok_or_else(|| anyhow!("failed to determine service name from {module_path}"))?;
    let helm_path = resolve_helm_path(&workspace_root, &module_path, &service_name, &tier)?;
    let version = detect_version(target_path).unwrap_or_else(|_| "0.1.0".to_string());
    let sha = detect_revision(&workspace_root);
    let image_tag_suffix = format!("{version}-{sha}");
    let full_image_tag = build_image_tag(
        &resolve_registry(&workspace_root),
        &tier,
        &service_name,
        &version,
        &sha,
    );

    Ok(DeployPlan {
        environment,
        workspace_root,
        target_path: target_path.to_path_buf(),
        module_path,
        tier,
        service_name,
        helm_path,
        version,
        sha,
        image_tag_suffix,
        full_image_tag,
    })
}

fn resolve_registry(workspace_root: &Path) -> String {
    let candidate_paths = [
        workspace_root.join("k1s0-cli.yaml"),
        workspace_root.join("config.yaml"),
        workspace_root.join(".k1s0").join("cli-config.yaml"),
    ];

    for candidate in candidate_paths {
        if let Some(registry) = load_registry_from_yaml(&candidate) {
            return registry;
        }
    }

    CliConfig::default().docker_registry
}

fn load_registry_from_yaml(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let parsed = serde_yaml::from_str::<Value>(&content).ok()?;
    parsed
        .get("docker_registry")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn detect_version(target_path: &Path) -> Result<String> {
    if target_path.join("Cargo.toml").exists() {
        let cargo_toml = fs::read_to_string(target_path.join("Cargo.toml"))
            .map_err(|error| anyhow!("failed to read Cargo.toml: {error}"))?;
        let regex = Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).unwrap();
        let version = regex
            .captures(&cargo_toml)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .ok_or_else(|| anyhow!("failed to detect Cargo version"))?;
        return Ok(version);
    }

    if target_path.join("package.json").exists() {
        let package_json = fs::read_to_string(target_path.join("package.json"))
            .map_err(|error| anyhow!("failed to read package.json: {error}"))?;
        let parsed: Value = serde_json::from_str(&package_json)
            .map_err(|error| anyhow!("invalid package.json: {error}"))?;
        let version = parsed
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("package.json does not contain version"))?;
        return Ok(version.to_string());
    }

    if target_path.join("pubspec.yaml").exists() {
        let pubspec = fs::read_to_string(target_path.join("pubspec.yaml"))
            .map_err(|error| anyhow!("failed to read pubspec.yaml: {error}"))?;
        let regex = Regex::new(r"(?m)^version:\s*([^\s]+)").unwrap();
        let version = regex
            .captures(&pubspec)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .ok_or_else(|| anyhow!("failed to detect pubspec version"))?;
        return Ok(version);
    }

    bail!("failed to detect target version")
}

fn detect_revision(workspace_root: &Path) -> String {
    if let Ok(sha) = std::env::var("K1S0_IMAGE_SHA") {
        let trimmed = sha.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(workspace_root)
        .output();

    match output {
        Ok(result) if result.status.success() => {
            String::from_utf8_lossy(&result.stdout).trim().to_string()
        }
        _ => "local".to_string(),
    }
}

fn resolve_helm_path(
    workspace_root: &Path,
    module_path: &str,
    service_name: &str,
    tier: &str,
) -> Result<String> {
    let normalized = module_path.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();

    let mut candidates = Vec::new();
    match tier {
        "service" if parts.len() >= 3 => candidates.push(format!("service/{}", parts[2])),
        "business" if parts.len() >= 6 => {
            candidates.push(format!("business/{}/{}", parts[2], service_name));
        }
        "system" => candidates.push(format!("system/{service_name}")),
        _ => {}
    }
    candidates.push(format!("{tier}/{service_name}"));

    for candidate in candidates {
        if workspace_root
            .join("infra")
            .join("helm")
            .join("services")
            .join(&candidate)
            .is_dir()
        {
            return Ok(candidate);
        }
    }

    let services_root = workspace_root.join("infra").join("helm").join("services");
    for entry in walkdir::WalkDir::new(&services_root).into_iter().flatten() {
        if entry.file_type().is_dir() && entry.file_name().to_string_lossy() == service_name {
            let relative = entry
                .path()
                .strip_prefix(&services_root)
                .map_err(|error| anyhow!("failed to resolve helm path: {error}"))?;
            return Ok(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    bail!("failed to resolve helm path for {module_path}")
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

fn to_relative_path(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| anyhow!("failed to relativize path: {error}"))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn run_checked_command(cmd: &str, args: &[String], cwd: &Path) -> Result<()> {
    let status = Command::new(cmd)
        .args(args.iter().map(String::as_str))
        .current_dir(cwd)
        .status()
        .map_err(|error| anyhow!("failed to start {cmd}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        bail!("{cmd} exited with {}", status.code().unwrap_or(-1))
    }
}

fn build_error(plan: &DeployPlan, step: DeployStep, error: &anyhow::Error) -> DeployError {
    let manual_command = match step {
        DeployStep::DockerBuild => format!(
            "cd {} && docker build -t {} .",
            plan.module_path, plan.full_image_tag
        ),
        DeployStep::DockerPush => format!("docker push {}", plan.full_image_tag),
        DeployStep::CosignSign => format!("cosign sign --yes {}", plan.full_image_tag),
        DeployStep::HelmDeploy => format!(
            "cd {} && helm {}",
            plan.workspace_root.display(),
            build_helm_args(
                &plan.service_name,
                &plan.helm_path,
                &plan.tier,
                plan.environment.as_str(),
                &plan.image_tag_suffix,
            )
            .join(" ")
        ),
    };

    DeployError {
        step,
        message: error.to_string(),
        manual_command,
    }
}

fn emit<F>(on_progress: Option<&F>, event: ProgressEvent)
where
    F: Fn(ProgressEvent),
{
    if let Some(callback) = on_progress {
        callback(event);
    } else {
        crate::progress::print_progress(&event);
    }
}

pub fn scan_deployable_targets() -> Vec<String> {
    scan_deployable_targets_at(Path::new("."))
}

pub fn scan_deployable_targets_at(base_dir: &Path) -> Vec<String> {
    let mut targets = Vec::new();
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return targets;
    }
    scan_targets_recursive(&regions, &mut targets);
    targets.sort();
    targets
}

fn scan_targets_recursive(path: &Path, targets: &mut Vec<String>) {
    if !path.is_dir() {
        return;
    }

    let is_deployable = path.join("Dockerfile").exists();
    if is_deployable {
        let path_str = path.to_string_lossy().replace('\\', "/");
        if !path_str.contains("/library/") {
            targets.push(path.to_string_lossy().to_string());
        }
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_targets_recursive(&entry.path(), targets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Dev.as_str(), "dev");
        assert_eq!(Environment::Staging.as_str(), "staging");
        assert_eq!(Environment::Prod.as_str(), "prod");
    }

    #[test]
    fn test_environment_is_prod() {
        assert!(!Environment::Dev.is_prod());
        assert!(!Environment::Staging.is_prod());
        assert!(Environment::Prod.is_prod());
    }

    #[test]
    fn test_deploy_config_creation() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec!["regions/system/server/rust/auth".to_string()],
        };
        assert_eq!(config.environment, Environment::Dev);
        assert_eq!(config.targets.len(), 1);
    }

    #[test]
    fn test_scan_deployable_targets_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(scan_deployable_targets_at(tmp.path()).is_empty());
    }

    #[test]
    fn test_scan_deployable_targets_excludes_library() {
        let tmp = TempDir::new().unwrap();

        let server_path = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&server_path).unwrap();
        fs::write(server_path.join("Dockerfile"), "FROM scratch\n").unwrap();

        let lib_path = tmp.path().join("regions/system/library/rust/authlib");
        fs::create_dir_all(&lib_path).unwrap();
        fs::write(lib_path.join("Dockerfile"), "FROM scratch\n").unwrap();

        let targets = scan_deployable_targets_at(tmp.path());
        assert_eq!(targets.len(), 1);
        assert!(targets[0].contains("server"));
    }

    #[test]
    fn test_build_image_tag() {
        let tag = build_image_tag(
            "harbor.internal.example.com",
            "service",
            "order",
            "1.2.3",
            "abc1234",
        );
        assert_eq!(
            tag,
            "harbor.internal.example.com/k1s0-service/order:1.2.3-abc1234"
        );
    }

    #[test]
    fn test_build_helm_args() {
        let args = build_helm_args("order", "service/order", "service", "dev", "1.2.3-abc1234");
        assert_eq!(
            args,
            vec![
                "upgrade",
                "--install",
                "order",
                "./infra/helm/services/service/order",
                "-n",
                "k1s0-service",
                "-f",
                "./infra/helm/services/service/order/values-dev.yaml",
                "--set",
                "image.tag=1.2.3-abc1234",
            ]
        );
    }

    #[test]
    fn test_build_helm_rollback_args() {
        let args = build_helm_rollback_args("order", "service");
        assert_eq!(args, vec!["rollback", "order", "-n", "k1s0-service"]);
    }

    #[test]
    fn test_extract_tier_from_target_path() {
        assert_eq!(
            extract_tier_from_target_path("regions/service/order/server/rust"),
            Some("service".to_string())
        );
        assert_eq!(
            extract_tier_from_target_path("regions/system/server/rust/auth"),
            Some("system".to_string())
        );
        assert_eq!(extract_tier_from_target_path("invalid/path"), None);
    }

    #[test]
    fn test_extract_service_name_from_target_path() {
        assert_eq!(
            extract_service_name_from_target_path("regions/service/order/server/rust"),
            Some("order".to_string())
        );
        assert_eq!(
            extract_service_name_from_target_path("regions/system/server/rust/auth"),
            Some("auth".to_string())
        );
        assert_eq!(extract_service_name_from_target_path("invalid"), None);
    }

    #[test]
    fn test_format_step_messages() {
        assert_eq!(
            format_step_start(&DeployStep::DockerBuild),
            "[1/4] Docker image build..."
        );
        assert_eq!(
            format_step_done(&DeployStep::HelmDeploy),
            "[4/4] done: Helm deploy"
        );
    }

    #[test]
    fn test_format_deploy_success() {
        let message = format_deploy_success(
            "dev",
            "order",
            "harbor.internal.example.com/k1s0-service/order:1.0.0-abc1234",
            "service",
        );
        assert!(message.contains("Deploy completed"));
        assert!(message.contains("helm status order -n k1s0-service"));
    }

    #[test]
    fn test_format_deploy_failure() {
        let error = DeployError {
            step: DeployStep::DockerBuild,
            message: "exit code 1".to_string(),
            manual_command: "docker build".to_string(),
        };
        let message = format_deploy_failure(&error);
        assert!(message.contains("Deploy failed"));
        assert!(message.contains("Docker image build"));
    }

    #[test]
    fn test_total_deploy_steps() {
        assert_eq!(TOTAL_DEPLOY_STEPS, 4);
    }

    #[test]
    fn test_build_deploy_plan_detects_workspace_and_chart() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("regions/system/server/rust/auth");
        let helm = tmp.path().join("infra/helm/services/system/auth");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&helm).unwrap();
        fs::write(target.join("Dockerfile"), "FROM scratch\n").unwrap();
        fs::write(
            target.join("Cargo.toml"),
            "[package]\nname = \"auth\"\nversion = \"0.4.0\"\n",
        )
        .unwrap();
        fs::write(helm.join("values-dev.yaml"), "image: {}\n").unwrap();

        let plan = build_deploy_plan(&target, Environment::Dev).unwrap();
        assert_eq!(plan.tier, "system");
        assert_eq!(plan.service_name, "auth");
        assert_eq!(plan.helm_path, "system/auth");
        assert_eq!(plan.version, "0.4.0");
    }

    #[test]
    fn test_execute_deploy_nonexistent_target_is_error() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec!["/nonexistent/path".to_string()],
        };
        assert!(execute_deploy(&config).is_err());
    }

    #[test]
    fn test_execute_deploy_with_progress_nonexistent_target_marks_failure() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_deploy_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });

        assert!(result.is_err());

        let collected = events.lock().unwrap();
        assert!(matches!(
            &collected[0],
            ProgressEvent::StepStarted {
                step: 1,
                total: 1,
                ..
            }
        ));
        assert!(collected
            .iter()
            .any(|event| matches!(event, ProgressEvent::Error { .. })));
        assert!(matches!(
            collected.last().unwrap(),
            ProgressEvent::Finished { success: false, .. }
        ));
    }

    #[test]
    fn test_execute_deploy_with_progress_empty_targets_marks_failure() {
        let config = DeployConfig {
            environment: Environment::Dev,
            targets: vec![],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_deploy_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });

        assert!(result.is_err());

        let collected = events.lock().unwrap();
        assert_eq!(collected.len(), 1);
        assert!(matches!(
            &collected[0],
            ProgressEvent::Finished { success: false, .. }
        ));
    }
}
