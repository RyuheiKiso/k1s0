use anyhow::Result;
use std::path::Path;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::init::{execute_init, InitConfig, Tier};

const ALL_TIERS: &[Tier] = &[Tier::System, Tier::Business, Tier::Service];
const TIER_LABELS: &[&str] = &["system", "business", "service"];

/// 初期化ウィザードのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    ProjectName,
    GitInit,
    SparseCheckout,
    TierSelection,
    Confirm,
}

/// プロジェクト初期化コマンドを実行する。
///
/// CLIフロー.md の「プロジェクト初期化」セクションに準拠:
/// [1] プロジェクト名入力
/// [2] Git リポジトリ初期化 (はい/いいえ)
/// [3] sparse-checkout 有効化 (はい/いいえ)
/// [4] (sparse-checkout時) Tier選択
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc を押すとメインメニューに戻る。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはプロジェクト初期化に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- プロジェクト初期化 ---\n");

    let mut step = Step::ProjectName;
    let mut project_name = String::new();
    let mut git_init = false;
    let mut sparse_checkout = false;
    let mut tiers: Vec<Tier> = vec![];

    loop {
        match step {
            Step::ProjectName => match step_project_name()? {
                Some(name) => {
                    project_name = name;
                    step = Step::GitInit;
                }
                None => return Ok(()), // 最初のステップ → メインメニューに戻る
            },
            Step::GitInit => match step_git_init()? {
                Some(g) => {
                    git_init = g;
                    step = Step::SparseCheckout;
                }
                None => {
                    step = Step::ProjectName; // 前のステップに戻る
                }
            },
            Step::SparseCheckout => match step_sparse_checkout()? {
                Some(s) => {
                    sparse_checkout = s;
                    if sparse_checkout {
                        step = Step::TierSelection;
                    } else {
                        tiers = vec![Tier::System, Tier::Business, Tier::Service];
                        step = Step::Confirm;
                    }
                }
                None => {
                    step = Step::GitInit; // 前のステップに戻る
                }
            },
            Step::TierSelection => match step_tier_selection()? {
                Some(t) => {
                    tiers = t;
                    step = Step::Confirm;
                }
                None => {
                    step = Step::SparseCheckout; // 前のステップに戻る
                }
            },
            Step::Confirm => {
                let config = InitConfig {
                    project_name: project_name.clone(),
                    git_init,
                    sparse_checkout,
                    tiers: tiers.clone(),
                };
                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute_init(&config)?;
                        println!(
                            "\nプロジェクト '{}' の初期化が完了しました。",
                            config.project_name
                        );
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        // 前のステップに戻る
                        if sparse_checkout {
                            step = Step::TierSelection;
                        } else {
                            step = Step::SparseCheckout;
                        }
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

/// ステップ1: プロジェクト名入力
///
/// 入力されたプロジェクト名が既にカレントディレクトリに存在する場合は
/// エラーメッセージを表示して再入力を促す。
fn step_project_name() -> Result<Option<String>> {
    loop {
        let name = match prompt::input_prompt("プロジェクト名を入力してください") {
            Ok(n) => n,
            Err(e) => {
                if is_dialoguer_escape(&e) {
                    return Ok(None);
                }
                return Err(e);
            }
        };
        if Path::new(&name).exists() {
            println!("'{name}' は既に存在します。別の名前を入力してください。");
            continue;
        }
        return Ok(Some(name));
    }
}

/// ステップ2: Git初期化
fn step_git_init() -> Result<Option<bool>> {
    prompt::yes_no_prompt("Git リポジトリを初期化しますか？")
}

/// ステップ3: sparse-checkout
fn step_sparse_checkout() -> Result<Option<bool>> {
    prompt::yes_no_prompt("sparse-checkout を有効にしますか？")
}

/// ステップ4: Tier選択
fn step_tier_selection() -> Result<Option<Vec<Tier>>> {
    let selected = prompt::multi_select_prompt(
        "チェックアウトするTierを選択してください（複数選択可）",
        TIER_LABELS,
    )?;

    match selected {
        None => Ok(None),
        Some(indices) => {
            if indices.is_empty() {
                println!("少なくとも1つのTierを選択してください。");
                // 再帰的にリトライ
                step_tier_selection()
            } else {
                let tiers: Vec<Tier> = indices.iter().map(|&i| ALL_TIERS[i]).collect();
                Ok(Some(tiers))
            }
        }
    }
}

/// 確認内容を表示する。
fn print_confirmation(config: &InitConfig) {
    let tiers_str: Vec<&str> = config
        .tiers
        .iter()
        .map(k1s0_core::commands::init::Tier::display)
        .collect();
    println!("\n[確認] 以下の内容で初期化します。よろしいですか？");
    println!("    プロジェクト名: {}", config.project_name);
    println!(
        "    Git 初期化:     {}",
        if config.git_init {
            "はい"
        } else {
            "いいえ"
        }
    );
    if config.sparse_checkout {
        println!("    sparse-checkout: はい ({})", tiers_str.join(", "));
    } else {
        println!("    sparse-checkout: いいえ");
    }
}

/// dialoguer のエスケープ/Ctrl+C を検出するヘルパー
fn is_dialoguer_escape(e: &anyhow::Error) -> bool {
    let msg = format!("{e}");
    msg.contains("interrupted") || msg.contains("Escape")
}
