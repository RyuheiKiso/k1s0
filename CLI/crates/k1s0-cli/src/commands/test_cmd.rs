use anyhow::Result;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::test_cmd::{
    execute_test, scan_e2e_suites, scan_testable_targets, TestConfig, TestKind,
};

const TEST_KIND_LABELS: &[&str] = &["ユニットテスト", "統合テスト", "E2Eテスト", "すべて"];
const ALL_TEST_KINDS: &[TestKind] = &[
    TestKind::Unit,
    TestKind::Integration,
    TestKind::E2e,
    TestKind::All,
];

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
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはテスト実行に失敗した場合にエラーを返す。
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

    let selected =
        prompt::multi_select_prompt("テスト対象を選択してください（複数選択可）", &items)?;

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

    let selected =
        prompt::multi_select_prompt("テストスイートを選択してください（複数選択可）", &items)?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つのスイートを選択してください。");
                step_select_e2e_suites()
            } else if indices.contains(&0) {
                Ok(Some(all_suites))
            } else {
                let targets: Vec<String> =
                    indices.iter().map(|&i| all_suites[i - 1].clone()).collect();
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
        println!("    対象: {target}");
    }
}
