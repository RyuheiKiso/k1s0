use anyhow::Result;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::deploy::*;

const ENV_LABELS: &[&str] = &["dev", "staging", "prod"];
const ALL_ENVS: &[Environment] = &[Environment::Dev, Environment::Staging, Environment::Prod];

/// デプロイフローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Environment,
    Targets,
    ProdConfirm,
    Confirm,
}

/// デプロイコマンドを実行する。
///
/// CLIフロー.md の「デプロイ」セクションに準拠:
/// [1] 環境の選択 (dev / staging / prod)
/// [2] 対象の選択（複数選択可）
/// [3] (prod時) "deploy" 入力確認
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
pub fn run() -> Result<()> {
    println!("\n--- デプロイ ---\n");

    let mut step = Step::Environment;
    let mut env = Environment::Dev;
    let mut targets: Vec<String> = Vec::new();

    loop {
        match step {
            Step::Environment => {
                match step_environment()? {
                    Some(e) => {
                        env = e;
                        step = Step::Targets;
                    }
                    None => return Ok(()), // 最初のステップで Esc → メインメニューに戻る
                }
            }
            Step::Targets => {
                match step_select_targets()? {
                    Some(t) => {
                        if t.is_empty() {
                            println!("デプロイ対象が見つかりません。");
                            return Ok(());
                        }
                        targets = t;
                        // prod の場合は ProdConfirm、それ以外は Confirm へ
                        step = if env.is_prod() {
                            Step::ProdConfirm
                        } else {
                            Step::Confirm
                        };
                    }
                    None => step = Step::Environment, // Esc → Environment に戻る
                }
            }
            Step::ProdConfirm => {
                match step_prod_confirmation()? {
                    Some(true) => {
                        step = Step::Confirm; // 通過
                    }
                    _ => {
                        println!("キャンセルしました。");
                        step = Step::Targets; // Esc → Targets に戻る
                    }
                }
            }
            Step::Confirm => {
                let config = DeployConfig {
                    environment: env,
                    targets: targets.clone(),
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_deploy(&config)?;
                        println!("\nデプロイが完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        // GoBack → ProdConfirm (prodの場合) or Targets (非prodの場合)
                        step = if env.is_prod() {
                            Step::ProdConfirm
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

/// ステップ1: 環境選択
fn step_environment() -> Result<Option<Environment>> {
    let idx = prompt::select_prompt("デプロイ先の環境を選択してください", ENV_LABELS)?;
    Ok(idx.map(|i| ALL_ENVS[i]))
}

/// ステップ2: 対象選択
fn step_select_targets() -> Result<Option<Vec<String>>> {
    let all_targets = scan_deployable_targets();
    if all_targets.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items: Vec<&str> = vec!["すべて"];
    for t in &all_targets {
        items.push(t.as_str());
    }

    let selected = prompt::multi_select_prompt(
        "デプロイ対象を選択してください（複数選択可）",
        &items,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つの対象を選択してください。");
                step_select_targets()
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

/// ステップ3: 本番環境の追加確認。"deploy" と入力させる。
fn step_prod_confirmation() -> Result<Option<bool>> {
    println!("\n⚠ 本番環境へのデプロイです。");
    let input = prompt::input_prompt_raw(
        "本当にデプロイしますか？ \"deploy\" と入力してください",
    )?;
    if input.trim() == "deploy" {
        Ok(Some(true))
    } else {
        println!("入力が一致しません。デプロイをキャンセルします。");
        Ok(Some(false))
    }
}

/// 確認内容を表示する。
fn print_confirmation(config: &DeployConfig) {
    println!("\n[確認] 以下の内容でデプロイします。よろしいですか？");
    println!("    環境: {}", config.environment.as_str());
    for target in &config.targets {
        println!("    対象: {}", target);
    }
}
