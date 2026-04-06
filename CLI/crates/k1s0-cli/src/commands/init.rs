use anyhow::{bail, Result};
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

/// 非インタラクティブモードでプロジェクト初期化を実行する。
///
/// `--name` が指定されていない場合はエラーを返す。
/// `git_init`・`sparse_checkout` はデフォルト値（false）を使用し、
/// 全Tierをチェックアウト対象とする。
///
/// # Errors
///
/// `--name` が未指定の場合、またはプロジェクト初期化に失敗した場合にエラーを返す。
pub fn run_non_interactive(name: Option<String>) -> Result<()> {
    // --name が未指定の場合は使用方法を案内してエラーを返す
    let project_name = match name {
        Some(n) if !n.is_empty() => n,
        _ => bail!(
            "非インタラクティブモードでは --name が必須です。\n\
            使用例: k1s0 init --name my-project --non-interactive"
        ),
    };

    // プロジェクト名のバリデーション: 英小文字・数字・ハイフンのみ許可（パストラバーサル防止）
    // L-001 監査対応: format string にエラー変数を直接埋め込む（冗長な {} + e を {e} に変更）。
    prompt::validate_name(&project_name)
        .map_err(|e| anyhow::anyhow!("プロジェクト名が無効です: {e}"))?;

    // 既にディレクトリが存在する場合は上書きを防ぐためエラーを返す
    if Path::new(&project_name).exists() {
        bail!("'{project_name}' は既に存在します。別の名前を指定してください。");
    }

    // MED-007 監査対応: 作成先の絶対パスを実行前に明示してユーザーの混乱を防ぐ
    // Path::new(".") はプロセスの cwd に解決されるため、ここで事前に表示する
    let output_dir = std::env::current_dir()
        .map(|d| d.join(&project_name))
        .map_or_else(|_| project_name.clone(), |p| p.display().to_string());
    println!("プロジェクトを '{output_dir}' に作成します...");

    // 非インタラクティブモードのデフォルト設定: git_init=false, sparse_checkout=false, 全Tier
    let config = InitConfig {
        project_name: project_name.clone(),
        git_init: false,
        sparse_checkout: false,
        tiers: vec![Tier::System, Tier::Business, Tier::Service],
    };

    execute_init(&config)?;
    println!("\nプロジェクト '{project_name}' の初期化が完了しました。\n作成先: {output_dir}");
    Ok(())
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
    // プロジェクトルートから実行していることを案内する
    println!("  ヒント: このコマンドは k1s0 プロジェクトのルートディレクトリで実行してください。");

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
    // sparse-checkout の概念説明を表示する
    println!("  sparse-checkout: 必要な Tier のファイルのみをローカルに取得します。");
    println!("  大規模リポジトリでのクローン時間を短縮できます。");
    prompt::yes_no_prompt("sparse-checkout を有効にしますか？")
}

/// ステップ4: Tier選択
///
/// 空選択の場合は再入力を促すループを使用する（再帰呼び出しを避ける）。
fn step_tier_selection() -> Result<Option<Vec<Tier>>> {
    loop {
        let selected = prompt::multi_select_prompt(
            "チェックアウトするTierを選択してください（複数選択可）",
            TIER_LABELS,
        )?;

        match selected {
            // Esc が押された場合は前のステップに戻る
            None => return Ok(None),
            Some(indices) => {
                if indices.is_empty() {
                    // 未選択の場合は警告を出して再入力を促す
                    println!("少なくとも1つのTierを選択してください。");
                } else {
                    let tiers: Vec<Tier> = indices.iter().map(|&i| ALL_TIERS[i]).collect();
                    return Ok(Some(tiers));
                }
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
    // MED-007 監査対応: 作成先の絶対パスを確認画面に表示してどこに生成されるかを明示する
    let output_dir = std::env::current_dir()
        .map(|d| d.join(&config.project_name))
        .map_or_else(|_| config.project_name.clone(), |p| p.display().to_string());
    println!("\n[確認] 以下の内容で初期化します。よろしいですか？");
    println!("    プロジェクト名: {}", config.project_name);
    println!("    作成先パス:     {output_dir}");
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
