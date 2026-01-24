//! `k1s0 lint` コマンド
//!
//! 規約違反を検査する。

use clap::Args;

use crate::error::Result;
use crate::output::output;

/// `k1s0 lint` の引数
#[derive(Args, Debug)]
pub struct LintArgs {
    /// 検査するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 特定のルールのみ実行（カンマ区切り）
    #[arg(long)]
    pub rules: Option<String>,

    /// 特定のルールを除外（カンマ区切り）
    #[arg(long)]
    pub exclude_rules: Option<String>,

    /// 警告をエラーとして扱う
    #[arg(long)]
    pub strict: bool,

    /// 自動修正を試みる
    #[arg(long)]
    pub fix: bool,
}

/// `k1s0 lint` を実行する
pub fn execute(args: LintArgs) -> Result<()> {
    let out = output();

    out.header("k1s0 lint");
    out.newline();

    out.list_item("path", &args.path);
    if let Some(rules) = &args.rules {
        out.list_item("rules", rules);
    }
    if let Some(exclude) = &args.exclude_rules {
        out.list_item("exclude_rules", exclude);
    }
    out.list_item("strict", &args.strict.to_string());
    out.list_item("fix", &args.fix.to_string());
    out.newline();

    out.info("TODO: 実装予定（フェーズ14）");
    out.newline();

    out.header("検査項目:");
    out.hint("manifest: .k1s0/manifest.json の存在・整合性");
    out.hint("structure: 必須ディレクトリ/ファイルの存在");
    out.hint("env-var: 環境変数参照の禁止");
    out.hint("secrets: config/*.yaml への機密値直書き禁止");
    out.hint("dependency: Clean Architecture の依存方向");

    Ok(())
}
