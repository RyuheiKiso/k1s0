use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::progress::ProgressEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestKind {
    Unit,
    Integration,
    All,
}

impl TestKind {
    pub fn label(&self) -> &'static str {
        match self {
            TestKind::Unit => "unit",
            TestKind::Integration => "integration",
            TestKind::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestConfig {
    pub kind: TestKind,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectLang {
    Go,
    Rust,
    Node,
    Flutter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCommand {
    pub cmd: String,
    pub args: Vec<String>,
}

pub fn detect_project_lang(target_path: &Path) -> Option<ProjectLang> {
    if target_path.join("go.mod").exists() {
        Some(ProjectLang::Go)
    } else if target_path.join("Cargo.toml").exists() {
        Some(ProjectLang::Rust)
    } else if target_path.join("package.json").exists() {
        Some(ProjectLang::Node)
    } else if target_path.join("pubspec.yaml").exists() {
        Some(ProjectLang::Flutter)
    } else {
        None
    }
}

pub fn resolve_test_command(kind: TestKind, lang: Option<ProjectLang>) -> Option<TestCommand> {
    let lang = lang?;
    let (cmd, args) = match (lang, kind) {
        (ProjectLang::Go, TestKind::Unit | TestKind::All) => ("go", vec!["test", "./..."]),
        (ProjectLang::Go, TestKind::Integration) => {
            ("go", vec!["test", "-tags=integration", "./..."])
        }
        (ProjectLang::Rust, TestKind::Unit) => ("cargo", vec!["test"]),
        (ProjectLang::Rust, TestKind::Integration) => ("cargo", vec!["test", "--tests"]),
        (ProjectLang::Rust, TestKind::All) => ("cargo", vec!["test", "--all"]),
        (ProjectLang::Node, _) => ("npm", vec!["test"]),
        (ProjectLang::Flutter, _) => ("flutter", vec!["test"]),
    };

    Some(TestCommand {
        cmd: cmd.to_string(),
        args: args.into_iter().map(str::to_string).collect(),
    })
}

pub fn execute_test(config: &TestConfig) -> Result<()> {
    execute_test_at(config, Path::new("."))
}

pub fn execute_test_at(config: &TestConfig, base_dir: &Path) -> Result<()> {
    execute_test_internal(config, base_dir, Option::<&fn(ProgressEvent)>::None)
}

pub fn execute_test_with_progress(
    config: &TestConfig,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    execute_test_with_progress_at(config, Path::new("."), on_progress)
}

pub fn execute_test_with_progress_at(
    config: &TestConfig,
    base_dir: &Path,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    execute_test_internal(config, base_dir, Some(&on_progress))
}

fn execute_test_internal<F>(
    config: &TestConfig,
    base_dir: &Path,
    on_progress: Option<&F>,
) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    let targets = match resolve_targets(config, base_dir) {
        Ok(targets) => targets,
        Err(error) => {
            emit(
                on_progress,
                ProgressEvent::Finished {
                    success: false,
                    message: "Tests failed".to_string(),
                },
            );
            return Err(error);
        }
    };
    let total = targets.len();
    let mut errors = Vec::new();

    for (index, target) in targets.iter().enumerate() {
        let step = index + 1;
        let target_path = Path::new(target);

        emit(
            on_progress,
            ProgressEvent::StepStarted {
                step,
                total,
                message: format!("Running {} tests for {target}", config.kind.label()),
            },
        );

        match test_target(target_path, config.kind, on_progress) {
            Ok(()) => emit(
                on_progress,
                ProgressEvent::StepCompleted {
                    step,
                    total,
                    message: format!("Tested {target}"),
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
                "Tests completed".to_string()
            } else {
                "Tests failed".to_string()
            },
        },
    );

    if success {
        Ok(())
    } else {
        bail!(errors.join("; "))
    }
}

fn resolve_targets(config: &TestConfig, base_dir: &Path) -> Result<Vec<String>> {
    let targets = if config.kind == TestKind::All && config.targets.is_empty() {
        scan_testable_targets_at(base_dir)
    } else {
        config.targets.clone()
    };

    if targets.is_empty() {
        bail!("no test targets selected");
    }

    Ok(targets)
}

fn test_target<F>(target_path: &Path, kind: TestKind, on_progress: Option<&F>) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    if !target_path.is_dir() {
        bail!("target directory does not exist");
    }

    let lang = detect_project_lang(target_path);
    let test_cmd =
        resolve_test_command(kind, lang).ok_or_else(|| anyhow!("target is not testable"))?;

    run_command(&test_cmd.cmd, &test_cmd.args, target_path, on_progress)
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

pub fn scan_testable_targets() -> Vec<String> {
    scan_testable_targets_at(Path::new("."))
}

pub fn scan_testable_targets_at(base_dir: &Path) -> Vec<String> {
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

    let is_testable = path.join("go.mod").exists()
        || path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join("pubspec.yaml").exists();

    if is_testable {
        if let Some(path_str) = path.to_str() {
            targets.push(path_str.replace('\\', "/"));
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
    fn test_test_kind_label() {
        assert_eq!(TestKind::Unit.label(), "unit");
        assert_eq!(TestKind::Integration.label(), "integration");
        assert_eq!(TestKind::All.label(), "all");
    }

    #[test]
    fn test_test_config_creation() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec!["regions/system/server/rust/auth".to_string()],
        };
        assert_eq!(config.kind, TestKind::Unit);
        assert_eq!(config.targets.len(), 1);
    }

    #[test]
    fn test_scan_testable_targets_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(scan_testable_targets_at(tmp.path()).is_empty());
    }

    #[test]
    fn test_scan_testable_targets_with_projects() {
        let tmp = TempDir::new().unwrap();
        let rust_path = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&rust_path).unwrap();
        fs::write(rust_path.join("Cargo.toml"), "[package]\n").unwrap();

        let targets = scan_testable_targets_at(tmp.path());
        assert_eq!(targets.len(), 1);
    }

    #[test]
    fn test_go_test_command_unit() {
        let cmd = resolve_test_command(TestKind::Unit, Some(ProjectLang::Go)).unwrap();
        assert_eq!(cmd.cmd, "go");
        assert_eq!(cmd.args, vec!["test", "./..."]);
    }

    #[test]
    fn test_rust_test_command_integration_uses_tests_flag() {
        let cmd = resolve_test_command(TestKind::Integration, Some(ProjectLang::Rust)).unwrap();
        assert_eq!(cmd.cmd, "cargo");
        assert_eq!(cmd.args, vec!["test", "--tests"]);
    }

    #[test]
    fn test_execute_test_nonexistent_target_is_error() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec!["/nonexistent/path".to_string()],
        };
        assert!(execute_test(&config).is_err());
    }

    #[test]
    fn test_execute_test_with_progress_nonexistent_target_marks_failure() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_test_with_progress(&config, move |event| {
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
    fn test_execute_test_with_progress_empty_targets_is_error() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec![],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let result = execute_test_with_progress(&config, move |event| {
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
    fn test_resolve_targets_for_all_scans_workspace() {
        let tmp = TempDir::new().unwrap();
        let rust_path = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&rust_path).unwrap();
        fs::write(rust_path.join("Cargo.toml"), "[package]\n").unwrap();

        let config = TestConfig {
            kind: TestKind::All,
            targets: vec![],
        };

        let targets = resolve_targets(&config, tmp.path()).unwrap();
        assert_eq!(
            targets,
            vec![rust_path.to_string_lossy().replace('\\', "/").to_string()]
        );
    }
}
