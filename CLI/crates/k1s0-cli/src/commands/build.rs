use anyhow::Result;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::build::{
    execute_build, scan_buildable_targets, BuildConfig, BuildMode,
};

const BUILD_MODE_LABELS: &[&str] = &["development", "production"];

/// ビルドフローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Targets,
    Mode,
    Confirm,
}

/// ビルドコマンドを実行する。
///
/// CLIフロー.md の「ビルド」セクションに準拠:
/// [1] ビルド対象の選択（複数選択可、「すべて」あり）
/// [2] ビルドモードの選択 (development / production)
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはビルド実行に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- ビルド ---\n");

    let mut step = Step::Targets;
    let mut targets: Vec<String> = Vec::new();
    let mut mode = BuildMode::Development;

    loop {
        match step {
            Step::Targets => {
                match step_select_targets()? {
                    Some(t) => {
                        if t.is_empty() {
                            println!("ビルド対象が見つかりません。先にひな形を生成してください。");
                            return Ok(());
                        }
                        targets = t;
                        step = Step::Mode;
                    }
                    None => return Ok(()), // 最初のステップで Esc → メインメニューに戻る
                }
            }
            Step::Mode => {
                match step_build_mode()? {
                    Some(m) => {
                        mode = m;
                        step = Step::Confirm;
                    }
                    None => step = Step::Targets, // Esc → Targets に戻る
                }
            }
            Step::Confirm => {
                let config = BuildConfig {
                    targets: targets.clone(),
                    mode,
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_build(&config)?;
                        println!("\nビルドが完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => step = Step::Mode, // GoBack → Mode に戻る
                    ConfirmResult::Cancel => {
                        println!("キャンセルしました。");
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// ステップ1: ビルド対象の選択
fn step_select_targets() -> Result<Option<Vec<String>>> {
    let all_targets = scan_buildable_targets();
    if all_targets.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    for t in &all_targets {
        items.push(t.as_str());
    }

    let selected =
        prompt::multi_select_prompt("ビルド対象を選択してください（複数選択可）", &items)?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つの対象を選択してください。");
                step_select_targets()
            } else if indices.contains(&0) {
                // 「すべて」が選択されている
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

/// ステップ2: ビルドモード選択
fn step_build_mode() -> Result<Option<BuildMode>> {
    let idx = prompt::select_prompt("ビルドモードを選択してください", BUILD_MODE_LABELS)?;
    Ok(idx.map(|i| match i {
        0 => BuildMode::Development,
        1 => BuildMode::Production,
        _ => unreachable!(),
    }))
}

/// 確認内容を表示する。
fn print_confirmation(config: &BuildConfig) {
    println!("\n[確認] 以下の対象をビルドします。よろしいですか？");
    for target in &config.targets {
        println!("    対象:   {target}");
    }
    println!("    モード: {}", config.mode.as_str());
}
