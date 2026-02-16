use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::prompt::{self, ConfirmResult};

/// テスト種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

const TEST_KIND_LABELS: &[&str] = &["ユニットテスト", "統合テスト", "E2Eテスト", "すべて"];
const ALL_TEST_KINDS: &[TestKind] = &[
    TestKind::Unit,
    TestKind::Integration,
    TestKind::E2e,
    TestKind::All,
];

/// テスト設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestConfig {
    /// テスト種別
    pub kind: TestKind,
    /// テスト対象のパス一覧
    pub targets: Vec<String>,
}

/// テスト実行フローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Kind,
    Targets,
    Confirm,
}

/// テスト実行コマンド。
///
/// CLIフロー.md の「テスト実行」セクションに準拠:
/// [1] テスト種別の選択
/// [2] 対象の選択（種別=All の場合はスキップ）
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
pub fn run() -> Result<()> {
    println!("\n--- テスト実行 ---\n");

    let mut step = Step::Kind;
    let mut kind = TestKind::All;
    let mut targets: Vec<String> = Vec::new();

    loop {
        match step {
            Step::Kind => {
                match step_test_kind()? {
                    Some(k) => {
                        kind = k;
                        // All の場合は Targets をスキップして Confirm へ
                        if kind == TestKind::All {
                            let mut all = scan_testable_targets();
                            let e2e = scan_e2e_suites();
                            all.extend(e2e);
                            targets = all;
                            step = Step::Confirm;
                        } else {
                            step = Step::Targets;
                        }
                    }
                    None => return Ok(()), // 最初のステップで Esc → メインメニューに戻る
                }
            }
            Step::Targets => {
                let result = match kind {
                    TestKind::E2e => step_select_e2e_suites()?,
                    _ => step_select_test_targets()?,
                };
                match result {
                    Some(t) => {
                        if t.is_empty() {
                            println!("テスト対象が見つかりません。");
                            return Ok(());
                        }
                        targets = t;
                        step = Step::Confirm;
                    }
                    None => step = Step::Kind, // Esc → Kind に戻る
                }
            }
            Step::Confirm => {
                let config = TestConfig {
                    kind,
                    targets: targets.clone(),
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_test(&config)?;
                        println!("\nテスト実行が完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        // GoBack → Targets に戻る（All の場合は Kind に戻る）
                        step = if kind == TestKind::All {
                            Step::Kind
                        } else {
                            Step::Targets
                        };
                    }
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// ステップ1: テスト種別選択
fn step_test_kind() -> Result<Option<TestKind>> {
    let idx = prompt::select_prompt("テスト種別を選択してください", TEST_KIND_LABELS)?;
    Ok(idx.map(|i| ALL_TEST_KINDS[i]))
}

/// ステップ2: テスト対象選択（ユニット/統合テスト用）
fn step_select_test_targets() -> Result<Option<Vec<String>>> {
    let all_targets = scan_testable_targets();
    if all_targets.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    for t in &all_targets {
        items.push(t.as_str());
    }

    let selected = prompt::multi_select_prompt(
        "テスト対象を選択してください（複数選択可）",
        &items,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つの対象を選択してください。");
                step_select_test_targets()
            } else if indices.contains(&0) {
                Ok(Some(all_targets))
            } else {
                let targets: Vec<String> = indices
                    .iter()
                    .map(|&i| all_targets[i - 1].clone())
                    .collect();
                Ok(Some(targets))
            }
        }
    }
}

/// ステップ2: E2Eテストスイート選択
fn step_select_e2e_suites() -> Result<Option<Vec<String>>> {
    let all_suites = scan_e2e_suites();
    if all_suites.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    for s in &all_suites {
        items.push(s.as_str());
    }

    let selected = prompt::multi_select_prompt(
        "テストスイートを選択してください（複数選択可）",
        &items,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つのスイートを選択してください。");
                step_select_e2e_suites()
            } else if indices.contains(&0) {
                Ok(Some(all_suites))
            } else {
                let targets: Vec<String> = indices
                    .iter()
                    .map(|&i| all_suites[i - 1].clone())
                    .collect();
                Ok(Some(targets))
            }
        }
    }
}

/// 確認内容を表示する。
fn print_confirmation(config: &TestConfig) {
    println!("\n[確認] 以下のテストを実行します。よろしいですか？");
    println!("    種別: {}", config.kind.label());
    for target in &config.targets {
        println!("    対象: {}", target);
    }
}

/// プロジェクトの言語/フレームワーク種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectLang {
    Go,
    Rust,
    Node,
    Flutter,
}

/// テストコマンド情報。
#[derive(Debug, Clone, PartialEq, Eq)]
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
        (ProjectLang::Go, TestKind::Unit) => ("go", vec!["test", "./..."]),
        (ProjectLang::Go, TestKind::Integration) => ("go", vec!["test", "-tags=integration", "./..."]),
        (ProjectLang::Go, TestKind::All) => ("go", vec!["test", "./..."]),
        (ProjectLang::Rust, TestKind::Unit) => ("cargo", vec!["test"]),
        (ProjectLang::Rust, TestKind::Integration) => ("cargo", vec!["test", "--test", "*"]),
        (ProjectLang::Rust, TestKind::All) => ("cargo", vec!["test", "--all"]),
        (ProjectLang::Node, _) => ("npm", vec!["test"]),
        (ProjectLang::Flutter, _) => ("flutter", vec!["test"]),
        _ => return None,
    };

    Some(TestCommand {
        cmd: cmd.to_string(),
        args: args.into_iter().map(|s| s.to_string()).collect(),
    })
}

/// テスト実行。
pub fn execute_test(config: &TestConfig) -> Result<()> {
    for target in &config.targets {
        println!(
            "\nテスト実行中: {} ({})",
            target,
            config.kind.label()
        );
        let target_path = Path::new(target);

        if !target_path.is_dir() {
            println!("  警告: ディレクトリが見つかりません: {}", target);
            continue;
        }

        // E2Eテストか判定
        if target.starts_with("e2e/") || target.starts_with("e2e\\") {
            run_e2e_test(target_path)?;
            continue;
        }

        // 言語/フレームワーク種別を検出してテストコマンドを決定
        let lang = detect_project_lang(target_path);
        if let Some(test_cmd) = resolve_test_command(config.kind, lang) {
            let args_refs: Vec<&str> = test_cmd.args.iter().map(|s| s.as_str()).collect();
            run_command(&test_cmd.cmd, &args_refs, target_path)?;
        } else {
            println!("  警告: テスト方法が不明です: {}", target);
        }
    }
    Ok(())
}

/// E2Eテストを実行する。
fn run_e2e_test(path: &Path) -> Result<()> {
    // Python + pytest を想定
    run_command("pytest", &["."], path)
}

/// 外部コマンドを実行する。
fn run_command(cmd: &str, args: &[&str], cwd: &Path) -> Result<()> {
    println!("  実行: {} {}", cmd, args.join(" "));
    let status = Command::new(cmd).args(args).current_dir(cwd).status();
    match status {
        Ok(s) if s.success() => {
            println!("  完了");
            Ok(())
        }
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            println!("  警告: テストがエラーで終了しました (exit code: {})", code);
            Ok(())
        }
        Err(e) => {
            println!("  警告: コマンド '{}' の実行に失敗しました: {}", cmd, e);
            Ok(())
        }
    }
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
            targets: vec!["regions/system/server/go/auth".to_string()],
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

        let go_path = tmp.path().join("regions/system/server/go/auth");
        fs::create_dir_all(&go_path).unwrap();
        fs::write(go_path.join("go.mod"), "module auth\n").unwrap();

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
}
