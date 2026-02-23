mod execute;
mod helpers;
mod steps;
mod types;
pub use types::*;

use crate::prompt::{self, ConfirmResult};
use anyhow::Result;

use steps::{
    print_confirmation, step_detail, step_kind, step_lang_fw, step_placement, step_tier, StepResult,
};

// ============================================================================
// ステートマシン
// ============================================================================

/// ステートマシンのステップ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Kind,
    Tier,
    Placement,
    LangFw,
    Detail,
    Confirm,
}

/// Placement ステップがスキップされたかどうかを判定する。
///
/// System Tier の場合は配置先指定がスキップされるため、
/// `LangFw` ステップから Esc で戻るときの戻り先を Tier にする。
fn placement_was_skipped(tier: Tier) -> bool {
    tier == Tier::System
}

// ============================================================================
// run()
// ============================================================================

/// ひな形生成コマンドを実行する。
///
/// CLIフロー.md の「ひな形生成」セクションに完全準拠。
/// 各ステップで Esc を押すと前のステップに戻る。
/// 最初のステップで Esc → メインメニューに戻る。
///
/// # Errors
///
/// プロンプトの入出力に失敗した場合、またはひな形生成に失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- ひな形生成 ---\n");

    let mut step = Step::Kind;

    // 各ステップの選択結果を保持する変数
    let mut kind = Kind::Server;
    let mut tier = Tier::System;
    let mut placement: Option<String> = None;
    let mut lang_fw = LangFw::Language(Language::Go);
    let mut detail = DetailConfig::default();

    loop {
        match step {
            Step::Kind => match step_kind()? {
                Some(k) => {
                    kind = k;
                    step = Step::Tier;
                }
                None => return Ok(()),
            },

            Step::Tier => match step_tier(kind)? {
                Some(t) => {
                    tier = t;
                    step = Step::Placement;
                }
                None => {
                    step = Step::Kind;
                }
            },

            Step::Placement => match step_placement(tier)? {
                StepResult::Value(p) => {
                    placement = p;
                    step = Step::LangFw;
                }
                StepResult::Skip => {
                    placement = None;
                    step = Step::LangFw;
                }
                StepResult::Back => {
                    step = Step::Tier;
                }
            },

            Step::LangFw => match step_lang_fw(kind)? {
                Some(lf) => {
                    lang_fw = lf;
                    step = Step::Detail;
                }
                None => {
                    // Placement がスキップだった場合は Tier に戻る
                    if placement_was_skipped(tier) {
                        step = Step::Tier;
                    } else {
                        step = Step::Placement;
                    }
                }
            },

            Step::Detail => match step_detail(kind, tier, placement.as_deref(), &lang_fw)? {
                Some(d) => {
                    detail = d;
                    step = Step::Confirm;
                }
                None => {
                    step = Step::LangFw;
                }
            },

            Step::Confirm => {
                let config = GenerateConfig {
                    kind,
                    tier,
                    placement: placement.clone(),
                    lang_fw: lang_fw.clone(),
                    detail: detail.clone(),
                };

                print_confirmation(&config);
                match prompt::confirm_prompt()? {
                    ConfirmResult::Yes => {
                        execute::execute_generate(&config)?;
                        println!("\nひな形の生成が完了しました。");
                        return Ok(());
                    }
                    ConfirmResult::GoBack => {
                        step = Step::Detail;
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
