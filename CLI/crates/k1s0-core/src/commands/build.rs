use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

use super::command_runner::run_streaming_command;
use crate::progress::ProgressEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildMode {
    Development,
    Production,
}

impl BuildMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildMode::Development => "development",
            BuildMode::Production => "production",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    pub targets: Vec<String>,
    pub mode: BuildMode,
}

/// Execute builds for the selected targets.
///
/// # Errors
///
/// Returns an error when no targets are selected or any build command fails.
pub fn execute_build(config: &BuildConfig) -> Result<()> {
    execute_build_internal(config, Option::<&fn(ProgressEvent)>::None)
}

/// Execute builds for the selected targets while emitting progress events.
///
/// # Errors
///
/// Returns an error when no targets are selected or any build command fails.
pub fn execute_build_with_progress(
    config: &BuildConfig,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    execute_build_internal(config, Some(&on_progress))
}

fn execute_build_internal<F>(config: &BuildConfig, on_progress: Option<&F>) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    if config.targets.is_empty() {
        emit(
            on_progress,
            ProgressEvent::Finished {
                success: false,
                message: "No build targets selected".to_string(),
            },
        );
        bail!("no build targets selected");
    }

    let total = config.targets.len();
    let mut errors = Vec::new();

    for (index, target) in config.targets.iter().enumerate() {
        let step = index + 1;
        let target_path = Path::new(target);

        emit(
            on_progress,
            ProgressEvent::StepStarted {
                step,
                total,
                message: format!("Building {target} ({})", config.mode.as_str()),
            },
        );

        match build_target(target_path, config.mode, on_progress) {
            Ok(()) => emit(
                on_progress,
                ProgressEvent::StepCompleted {
                    step,
                    total,
                    message: format!("Built {target}"),
                },
            ),
            Err(error) => {
                let message = format!("{target}: {error}");
                errors.push(message.clone());
                emit(on_progress, ProgressEvent::Error { message });
            }
        }
    }

    let success = errors.is_empty();
    emit(
        on_progress,
        ProgressEvent::Finished {
            success,
            message: if success {
                "Build completed".to_string()
            } else {
                "Build failed".to_string()
            },
        },
    );

    if success {
        Ok(())
    } else {
        bail!(errors.join("; "))
    }
}

fn build_target<F>(target_path: &Path, mode: BuildMode, on_progress: Option<&F>) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    if !target_path.is_dir() {
        bail!("target directory does not exist");
    }

    if target_path.join("go.mod").exists() {
        let args = vec!["build".to_string(), "./...".to_string()];
        run_command("go", &args, target_path, on_progress)
    } else if target_path.join("Cargo.toml").exists() {
        let args = if mode == BuildMode::Production {
            vec!["build".to_string(), "--release".to_string()]
        } else {
            vec!["build".to_string()]
        };
        run_command("cargo", &args, target_path, on_progress)
    } else if target_path.join("package.json").exists() {
        let args = resolve_node_build_args(target_path, mode)?;
        run_command("npm", &args, target_path, on_progress)
    } else if target_path.join("pubspec.yaml").exists() {
        let args = vec!["build".to_string()];
        run_command("flutter", &args, target_path, on_progress)
    } else {
        bail!("target is not buildable");
    }
}

fn resolve_node_build_args(target_path: &Path, mode: BuildMode) -> Result<Vec<String>> {
    let package_json = fs::read_to_string(target_path.join("package.json"))
        .map_err(|error| anyhow!("failed to read package.json: {error}"))?;
    let parsed: Value = serde_json::from_str(&package_json)
        .map_err(|error| anyhow!("failed to parse package.json: {error}"))?;
    let scripts = parsed
        .get("scripts")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("package.json does not define scripts"))?;

    let preferred = match mode {
        BuildMode::Development => ["build:dev", "build:development", "build"],
        BuildMode::Production => ["build:prod", "build:production", "build"],
    };

    for script in preferred {
        if scripts.contains_key(script) {
            return Ok(vec!["run".to_string(), script.to_string()]);
        }
    }

    bail!("package.json does not define a compatible build script")
}

fn run_command<F>(cmd: &str, args: &[String], cwd: &Path, on_progress: Option<&F>) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    emit(
        on_progress,
        ProgressEvent::Log {
            message: format!("Running {cmd} {}", args.join(" ")),
        },
    );

    run_streaming_command(cmd, args, cwd, |message| {
        emit(on_progress, ProgressEvent::Log { message });
    })
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

pub fn scan_buildable_targets() -> Vec<String> {
    scan_buildable_targets_at(Path::new("."))
}

pub fn scan_buildable_targets_at(base_dir: &Path) -> Vec<String> {
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

    let is_buildable = path.join("go.mod").exists()
        || path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join("pubspec.yaml").exists();

    if is_buildable {
        if let Some(path_str) = path.to_str() {
            targets.push(path_str.to_string());
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

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_mode_as_str() {
        assert_eq!(BuildMode::Development.as_str(), "development");
        assert_eq!(BuildMode::Production.as_str(), "production");
    }

    #[test]
    fn test_build_config_creation() {
        let config = BuildConfig {
            targets: vec!["regions/system/server/rust/auth".to_string()],
            mode: BuildMode::Development,
        };
        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.mode, BuildMode::Development);
    }

    #[test]
    fn test_scan_buildable_targets_empty() {
        let tmp = TempDir::new().unwrap();
        let targets = scan_buildable_targets_at(tmp.path());
        assert!(targets.is_empty());
    }

    #[test]
    fn test_scan_buildable_targets_with_projects() {
        let tmp = TempDir::new().unwrap();

        let rust_system_path = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&rust_system_path).unwrap();
        fs::write(rust_system_path.join("Cargo.toml"), "[package]\n").unwrap();

        let rust_path = tmp
            .path()
            .join("regions/business/accounting/server/rust/ledger");
        fs::create_dir_all(&rust_path).unwrap();
        fs::write(rust_path.join("Cargo.toml"), "[package]\n").unwrap();

        let react_path = tmp.path().join("regions/service/order/client/react");
        fs::create_dir_all(&react_path).unwrap();
        fs::write(
            react_path.join("package.json"),
            r#"{"scripts":{"build":"vite build"}}"#,
        )
        .unwrap();

        let targets = scan_buildable_targets_at(tmp.path());
        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn test_execute_build_nonexistent_target_is_error() {
        let config = BuildConfig {
            targets: vec!["/nonexistent/path".to_string()],
            mode: BuildMode::Development,
        };
        assert!(execute_build(&config).is_err());
    }

    #[test]
    fn test_execute_build_with_progress_nonexistent_target_marks_failure() {
        let config = BuildConfig {
            targets: vec!["/nonexistent/path".to_string()],
            mode: BuildMode::Development,
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_build_with_progress(&config, move |event| {
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
    fn test_execute_build_with_progress_empty_targets_marks_failure() {
        let config = BuildConfig {
            targets: vec![],
            mode: BuildMode::Development,
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_build_with_progress(&config, move |event| {
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

    #[test]
    fn test_execute_build_with_progress_unknown_project_type_marks_failure() {
        let tmp = TempDir::new().unwrap();
        let config = BuildConfig {
            targets: vec![tmp.path().to_string_lossy().to_string()],
            mode: BuildMode::Development,
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_build_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });

        assert!(result.is_err());

        let collected = events.lock().unwrap();
        assert!(collected
            .iter()
            .any(|event| matches!(event, ProgressEvent::Error { .. })));
    }

    #[test]
    fn test_resolve_node_build_args_prefers_mode_specific_script() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"scripts":{"build":"vite build","build:dev":"vite build --mode development"}}"#,
        )
        .unwrap();

        let args = resolve_node_build_args(tmp.path(), BuildMode::Development).unwrap();
        assert_eq!(args, vec!["run", "build:dev"]);
    }
}
