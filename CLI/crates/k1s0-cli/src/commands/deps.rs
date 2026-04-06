use anyhow::Result;

use crate::prompt::{self, ConfirmResult};

pub use k1s0_core::commands::deps::*;

/// 対話フローのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Scope,
    TierSelect,
    ServiceSelect,
    OutputFormat,
    MermaidPath,
    Confirm,
}

/// 依存関係マップコマンドを実行する。
///
/// 対話フロー:
/// [1] 表示対象選択: 全体 / Tier指定 / サービス指定
/// [2] (Tier指定時) Tier選択
/// [3] (サービス指定時) サービス複数選択
/// [4] 出力形式選択: ターミナル / Mermaid / 両方
/// [5] (Mermaid時) 出力先パス入力
/// [確認] 確認画面
///
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
///
/// # 既知の制限（L-4 監査対応）
///
/// このコマンドは dialoguer クレートを使った対話型 TTY プロンプトを必要とする。
/// 非 TTY 環境（CI パイプライン、パイプ入力、nohup 等）では
/// `IoError: Not a terminal` エラーが発生し、コマンドが実行できない。
/// 非 TTY 環境でバッチ処理として実行したい場合は、将来的に
/// `--scope=all --output=mermaid --output-path=<path>` 形式のフラグ対応を予定している。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、または解析実行に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- 依存関係マップ ---\n");

    let mut step = Step::Scope;
    let mut scope = DepsScope::All;
    let mut output_format = DepsOutputFormat::Terminal;

    loop {
        match step {
            Step::Scope => match step_scope()? {
                Some(s) => {
                    scope = s;
                    step = match &scope {
                        DepsScope::All => Step::OutputFormat,
                        DepsScope::Tier(_) => Step::TierSelect,
                        DepsScope::Services(_) => Step::ServiceSelect,
                    };
                }
                None => return Ok(()),
            },
            Step::TierSelect => match step_tier_select()? {
                Some(tier) => {
                    scope = DepsScope::Tier(tier);
                    step = Step::OutputFormat;
                }
                None => step = Step::Scope,
            },
            Step::ServiceSelect => match step_service_select()? {
                Some(services) => {
                    if services.is_empty() {
                        println!("サービスが見つかりません。");
                        return Ok(());
                    }
                    scope = DepsScope::Services(services);
                    step = Step::OutputFormat;
                }
                None => step = Step::Scope,
            },
            Step::OutputFormat => match step_output_format()? {
                Some(fmt) => {
                    output_format = fmt;
                    step = match &output_format {
                        DepsOutputFormat::Terminal => Step::Confirm,
                        DepsOutputFormat::Mermaid(_) | DepsOutputFormat::Both(_) => {
                            Step::MermaidPath
                        }
                    };
                }
                None => {
                    // Esc → スコープに応じて戻る
                    step = match &scope {
                        DepsScope::All => Step::Scope,
                        DepsScope::Tier(_) => Step::TierSelect,
                        DepsScope::Services(_) => Step::ServiceSelect,
                    };
                }
            },
            Step::MermaidPath => match step_mermaid_path()? {
                Some(path) => {
                    let path_buf = std::path::PathBuf::from(path);
                    output_format = match &output_format {
                        DepsOutputFormat::Mermaid(_) => DepsOutputFormat::Mermaid(path_buf),
                        DepsOutputFormat::Both(_) => DepsOutputFormat::Both(path_buf),
                        DepsOutputFormat::Terminal => DepsOutputFormat::Terminal,
                    };
                    step = Step::Confirm;
                }
                None => step = Step::OutputFormat,
            },
            Step::Confirm => {
                print_confirmation(&scope, &output_format);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        let config = DepsConfig {
                            scope,
                            output: output_format,
                            no_cache: false,
                        };
                        execute_command(&config)?;
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        step = match &output_format {
                            DepsOutputFormat::Terminal => Step::OutputFormat,
                            DepsOutputFormat::Mermaid(_) | DepsOutputFormat::Both(_) => {
                                Step::MermaidPath
                            }
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

/// ステップ1: 表示対象選択
fn step_scope() -> Result<Option<DepsScope>> {
    let items = &["全体", "Tier指定", "サービス指定"];
    let idx = prompt::select_prompt("表示対象を選択してください", items)?;
    Ok(idx.map(|i| match i {
        0 => DepsScope::All,
        1 => DepsScope::Tier(String::new()), // TierSelectステップで決定
        2 => DepsScope::Services(Vec::new()), // ServiceSelectステップで決定
        _ => unreachable!(),
    }))
}

/// ステップ2a: Tier選択
fn step_tier_select() -> Result<Option<String>> {
    let items = &["system", "business", "service"];
    let idx = prompt::select_prompt("Tierを選択してください", items)?;
    Ok(idx.map(|i| items[i].to_string()))
}

/// ステップ2b: サービス複数選択
///
/// 空選択の場合は再入力を促すループを使用する（再帰呼び出しを避ける）。
fn step_service_select() -> Result<Option<Vec<String>>> {
    let services = scan_services(std::path::Path::new("."));
    if services.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let names: Vec<String> = services.iter().map(|s| s.name.clone()).collect();
    let labels: Vec<&str> = names.iter().map(String::as_str).collect();

    loop {
        let selected =
            prompt::multi_select_prompt("サービスを選択してください（複数選択可）", &labels)?;

        match selected {
            // Esc が押された場合は前のステップに戻る
            None => return Ok(None),
            Some(indices) => {
                if indices.is_empty() {
                    // 未選択の場合は警告を出して再入力を促す
                    println!("少なくとも1つのサービスを選択してください。");
                } else {
                    let selected_names: Vec<String> =
                        indices.iter().map(|&i| names[i].clone()).collect();
                    return Ok(Some(selected_names));
                }
            }
        }
    }
}

/// ステップ3: 出力形式選択
fn step_output_format() -> Result<Option<DepsOutputFormat>> {
    let items = &["ターミナル", "Mermaid (ファイル出力)", "両方"];
    let idx = prompt::select_prompt("出力形式を選択してください", items)?;
    Ok(idx.map(|i| match i {
        0 => DepsOutputFormat::Terminal,
        1 => DepsOutputFormat::Mermaid(std::path::PathBuf::new()),
        2 => DepsOutputFormat::Both(std::path::PathBuf::new()),
        _ => unreachable!(),
    }))
}

/// ステップ4: Mermaid出力先パス入力
fn step_mermaid_path() -> Result<Option<String>> {
    let path = prompt::input_prompt_raw(
        "Mermaid出力先パス (デフォルト: docs/diagrams/dependency-map.md)",
    )?;
    let path = path.trim().to_string();
    if path.is_empty() {
        Ok(Some("docs/diagrams/dependency-map.md".to_string()))
    } else {
        Ok(Some(path))
    }
}

/// 確認内容を表示する。
fn print_confirmation(scope: &DepsScope, output: &DepsOutputFormat) {
    println!("\n[確認] 以下の内容で依存関係マップを生成します。よろしいですか？");

    let scope_str = match scope {
        DepsScope::All => "全体".to_string(),
        DepsScope::Tier(tier) => format!("Tier: {tier}"),
        DepsScope::Services(services) => format!("サービス: {}", services.join(", ")),
    };
    println!("    対象: {scope_str}");

    let output_str = match output {
        DepsOutputFormat::Terminal => "ターミナル".to_string(),
        DepsOutputFormat::Mermaid(path) => format!("Mermaid ({})", path.display()),
        DepsOutputFormat::Both(path) => format!("両方 ({})", path.display()),
    };
    println!("    出力: {output_str}");
}

/// MED-008/HIGH-008 監査対応: 非インタラクティブモード用の deps コマンド実行関数。
/// --scope / --tier / --output / --output-path フラグで対話プロンプトをスキップして直接実行する。
///
/// # 使用例（CI/CD）
///
/// ```bash
/// # 全サービスをターミナルに出力
/// k1s0-cli deps --scope all --output terminal
///
/// # system tier を Mermaid ファイルに出力
/// k1s0-cli deps --scope tier --tier system --output mermaid --output-path docs/diagrams/system.md
///
/// # --non-interactive で実行（すべてのデフォルト値を使用）
/// k1s0-cli --non-interactive deps
/// ```
pub fn run_non_interactive(
    scope_str: Option<String>,
    tier: Option<String>,
    output_str: Option<String>,
    output_path: Option<std::path::PathBuf>,
) -> Result<()> {
    let scope = match scope_str.as_deref() {
        Some("all") | None => DepsScope::All,
        Some("tier") => {
            let t = tier.ok_or_else(|| {
                anyhow::anyhow!("--scope tier を指定する場合は --tier も必須です（例: --tier system）")
            })?;
            DepsScope::Tier(t)
        }
        Some("services") => {
            anyhow::bail!(
                "--scope services はサービス名の指定が必要です。非インタラクティブモードでは \
                --scope all または --scope tier を使用してください。"
            )
        }
        Some(other) => anyhow::bail!("不明なスコープ: '{other}'。'all' / 'tier' を指定してください。"),
    };

    let output = match output_str.as_deref() {
        Some("mermaid") => {
            let path = output_path.unwrap_or_else(|| {
                std::path::PathBuf::from("docs/diagrams/dependency-map.md")
            });
            DepsOutputFormat::Mermaid(path)
        }
        Some("both") => {
            let path = output_path.unwrap_or_else(|| {
                std::path::PathBuf::from("docs/diagrams/dependency-map.md")
            });
            DepsOutputFormat::Both(path)
        }
        Some("terminal") | None => DepsOutputFormat::Terminal,
        Some(other) => {
            anyhow::bail!("不明な出力形式: '{other}'。'terminal' / 'mermaid' / 'both' を指定してください。")
        }
    };

    let config = DepsConfig {
        scope,
        output,
        no_cache: false,
    };
    execute_command(&config)
}

/// 依存関係マップコマンドを実行する。
fn execute_command(config: &DepsConfig) -> Result<()> {
    println!("\n依存関係を解析しています...");

    let result = execute_deps(config)?;

    match &config.output {
        DepsOutputFormat::Terminal => {
            output::print_terminal(&result);
        }
        DepsOutputFormat::Mermaid(path) => {
            output::write_mermaid(&result, path)?;
        }
        DepsOutputFormat::Both(path) => {
            output::print_terminal(&result);
            output::write_mermaid(&result, path)?;
        }
    }

    // サマリー
    println!(
        "解析完了: サービス {} 件、依存関係 {} 件、違反 {} 件",
        result.services.len(),
        result.dependencies.len(),
        result.violations.len()
    );

    Ok(())
}
