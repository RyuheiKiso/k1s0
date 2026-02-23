use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::progress::ProgressEvent;

/// テスト種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestKind {
    Unit,
    Integration,
    E2e,
    All,
}

impl TestKind {
    pub fn label(&self) -> &'static str {
        match self {
            TestKind::Unit => "ユニットテスト",
            TestKind::Integration => "統合テスト",
            TestKind::E2e => "E2Eテスト",
            TestKind::All => "すべて",
        }
    }
}

/// テスト設定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestConfig {
    /// テスト種別
    pub kind: TestKind,
    /// テスト対象のパス一覧
    pub targets: Vec<String>,
}

/// プロジェクトの言語/フレームワーク種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectLang {
    Go,
    Rust,
    Node,
    Flutter,
}

/// テストコマンド情報。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCommand {
    /// 実行コマンド
    pub cmd: String,
    /// 引数
    pub args: Vec<String>,
}

/// ディレクトリ内のファイルからプロジェクト言語を検出する。
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

/// テスト種別とプロジェクト言語からテストコマンドを解決する。
///
/// E2Eテストの場合は lang を無視して pytest を返す。
pub fn resolve_test_command(kind: TestKind, lang: Option<ProjectLang>) -> Option<TestCommand> {
    if kind == TestKind::E2e {
        return Some(TestCommand {
            cmd: "pytest".to_string(),
            args: vec![".".to_string()],
        });
    }

    let lang = lang?;
    let (cmd, args) = match (lang, kind) {
        (ProjectLang::Go, TestKind::Unit | TestKind::All) => ("go", vec!["test", "./..."]),
        (ProjectLang::Go, TestKind::Integration) => {
            ("go", vec!["test", "-tags=integration", "./..."])
        }
        (ProjectLang::Rust, TestKind::Unit) => ("cargo", vec!["test"]),
        (ProjectLang::Rust, TestKind::Integration) => ("cargo", vec!["test", "--test", "*"]),
        (ProjectLang::Rust, TestKind::All) => ("cargo", vec!["test", "--all"]),
        (ProjectLang::Node, _) => ("npm", vec!["test"]),
        (ProjectLang::Flutter, _) => ("flutter", vec!["test"]),
        _ => return None,
    };

    Some(TestCommand {
        cmd: cmd.to_string(),
        args: args
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
    })
}

/// テスト実行。
///
/// # Errors
/// エラーが発生した場合。
pub fn execute_test(config: &TestConfig) -> Result<()> {
    for target in &config.targets {
        println!("\nテスト実行中: {} ({})", target, config.kind.label());
        let target_path = Path::new(target);

        if !target_path.is_dir() {
            println!("  警告: ディレクトリが見つかりません: {target}");
            continue;
        }

        // E2Eテストか判定
        if target.starts_with("e2e/") || target.starts_with("e2e\\") {
            run_e2e_test(target_path);
            continue;
        }

        // 言語/フレームワーク種別を検出してテストコマンドを決定
        let lang = detect_project_lang(target_path);
        if let Some(test_cmd) = resolve_test_command(config.kind, lang) {
            let args_refs: Vec<&str> = test_cmd
                .args
                .iter()
                .map(std::string::String::as_str)
                .collect();
            run_command(&test_cmd.cmd, &args_refs, target_path);
        } else {
            println!("  警告: テスト方法が不明です: {target}");
        }
    }
    Ok(())
}

/// E2Eテストを実行する。
fn run_e2e_test(path: &Path) {
    // Python + pytest を想定
    run_command("pytest", &["."], path);
}

/// 外部コマンドを実行する。
fn run_command(cmd: &str, args: &[&str], cwd: &Path) {
    println!("  実行: {} {}", cmd, args.join(" "));
    let status = Command::new(cmd).args(args).current_dir(cwd).status();
    match status {
        Ok(s) if s.success() => {
            println!("  完了");
        }
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            println!("  警告: テストがエラーで終了しました (exit code: {code})");
        }
        Err(e) => {
            println!("  警告: コマンド '{cmd}' の実行に失敗しました: {e}");
        }
    }
}

/// プログレスコールバック付きテスト実行。
///
/// # Errors
/// エラーが発生した場合。
pub fn execute_test_with_progress(
    config: &TestConfig,
    on_progress: impl Fn(ProgressEvent),
) -> Result<()> {
    let total = config.targets.len();
    for (i, target) in config.targets.iter().enumerate() {
        let step = i + 1;
        on_progress(ProgressEvent::StepStarted {
            step,
            total,
            message: format!("テスト実行中: {} ({})", target, config.kind.label()),
        });

        let target_path = Path::new(target);
        if !target_path.is_dir() {
            on_progress(ProgressEvent::Warning {
                message: format!("ディレクトリが見つかりません: {target}"),
            });
            on_progress(ProgressEvent::StepCompleted {
                step,
                total,
                message: format!("スキップ: {target}"),
            });
            continue;
        }

        if target.starts_with("e2e/") || target.starts_with("e2e\\") {
            on_progress(ProgressEvent::Log {
                message: "実行: pytest .".to_string(),
            });
            run_command("pytest", &["."], target_path);
        } else {
            let lang = detect_project_lang(target_path);
            if let Some(test_cmd) = resolve_test_command(config.kind, lang) {
                on_progress(ProgressEvent::Log {
                    message: format!("実行: {} {}", test_cmd.cmd, test_cmd.args.join(" ")),
                });
                let args_refs: Vec<&str> = test_cmd
                    .args
                    .iter()
                    .map(std::string::String::as_str)
                    .collect();
                run_command(&test_cmd.cmd, &args_refs, target_path);
            } else {
                on_progress(ProgressEvent::Warning {
                    message: format!("テスト方法が不明です: {target}"),
                });
            }
        }

        on_progress(ProgressEvent::StepCompleted {
            step,
            total,
            message: format!("テスト完了: {target}"),
        });
    }
    on_progress(ProgressEvent::Finished {
        success: true,
        message: "すべてのテストが完了しました".to_string(),
    });
    Ok(())
}

/// テスト可能な対象を走査する (regions/ 配下)。
pub fn scan_testable_targets() -> Vec<String> {
    scan_testable_targets_at(Path::new("."))
}

/// 指定ディレクトリを基点にテスト可能な対象を走査する (regions/ 配下)。
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
        if let Some(p) = path.to_str() {
            targets.push(p.to_string());
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

/// E2Eテストスイートを走査する (e2e/tests/ 配下)。
pub fn scan_e2e_suites() -> Vec<String> {
    scan_e2e_suites_at(Path::new("."))
}

/// 指定ディレクトリを基点にE2Eテストスイートを走査する (e2e/tests/ 配下)。
pub fn scan_e2e_suites_at(base_dir: &Path) -> Vec<String> {
    let mut suites = Vec::new();
    let e2e_tests = base_dir.join("e2e/tests");
    if !e2e_tests.is_dir() {
        return suites;
    }

    if let Ok(entries) = fs::read_dir(e2e_tests) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(p) = entry.path().to_str() {
                    suites.push(p.to_string());
                }
            }
        }
    }
    suites.sort();
    suites
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- TestKind ---

    #[test]
    fn test_test_kind_label() {
        assert_eq!(TestKind::Unit.label(), "ユニットテスト");
        assert_eq!(TestKind::Integration.label(), "統合テスト");
        assert_eq!(TestKind::E2e.label(), "E2Eテスト");
        assert_eq!(TestKind::All.label(), "すべて");
    }

    // --- TestConfig ---

    #[test]
    fn test_test_config_creation() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec!["regions/system/server/rust/auth".to_string()],
        };
        assert_eq!(config.kind, TestKind::Unit);
        assert_eq!(config.targets.len(), 1);
    }

    // --- scan ---

    #[test]
    fn test_scan_testable_targets_empty() {
        let tmp = TempDir::new().unwrap();
        let targets = scan_testable_targets_at(tmp.path());
        assert!(targets.is_empty());
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
    fn test_scan_e2e_suites_empty() {
        let tmp = TempDir::new().unwrap();
        let suites = scan_e2e_suites_at(tmp.path());
        assert!(suites.is_empty());
    }

    #[test]
    fn test_scan_e2e_suites_with_tests() {
        let tmp = TempDir::new().unwrap();

        fs::create_dir_all(tmp.path().join("e2e/tests/order")).unwrap();
        fs::create_dir_all(tmp.path().join("e2e/tests/auth")).unwrap();

        let suites = scan_e2e_suites_at(tmp.path());
        assert_eq!(suites.len(), 2);
    }

    // --- resolve_test_command ---

    #[test]
    fn test_go_test_command_unit() {
        // Goのユニットテストコマンドが "go test ./..." であること
        let cmd = resolve_test_command(TestKind::Unit, Some(ProjectLang::Go)).unwrap();
        assert_eq!(cmd.cmd, "go");
        assert_eq!(cmd.args, vec!["test", "./..."]);
    }

    #[test]
    fn test_rust_test_command() {
        // Rustのテストコマンドが "cargo test" であること
        let cmd = resolve_test_command(TestKind::Unit, Some(ProjectLang::Rust)).unwrap();
        assert_eq!(cmd.cmd, "cargo");
        assert_eq!(cmd.args, vec!["test"]);
    }

    #[test]
    fn test_python_e2e_command() {
        // E2Eテストコマンドが "pytest" であること
        let cmd = resolve_test_command(TestKind::E2e, None).unwrap();
        assert_eq!(cmd.cmd, "pytest");
        assert_eq!(cmd.args, vec!["."]);
    }

    // --- execute_test ---

    #[test]
    fn test_execute_test_nonexistent_target() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let result = execute_test(&config);
        assert!(result.is_ok());
    }

    // --- execute_test_with_progress ---

    #[test]
    fn test_execute_test_with_progress_nonexistent_target() {
        let config = TestConfig {
            kind: TestKind::Unit,
            targets: vec!["/nonexistent/path".to_string()],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_test_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());
        let collected = events.lock().unwrap();
        assert!(matches!(
            &collected[0],
            ProgressEvent::StepStarted {
                step: 1,
                total: 1,
                ..
            }
        ));
        assert!(matches!(
            collected.last().unwrap(),
            ProgressEvent::Finished { success: true, .. }
        ));
    }

    #[test]
    fn test_execute_test_with_progress_empty_targets() {
        let config = TestConfig {
            kind: TestKind::All,
            targets: vec![],
        };
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let result = execute_test_with_progress(&config, move |event| {
            events_clone.lock().unwrap().push(event);
        });
        assert!(result.is_ok());
        let collected = events.lock().unwrap();
        assert_eq!(collected.len(), 1);
        assert!(matches!(
            &collected[0],
            ProgressEvent::Finished { success: true, .. }
        ));
    }
}
