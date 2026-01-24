//! `k1s0 lint` コマンド
//!
//! 規約違反を検査する。

use anyhow::Result;
use clap::Args;

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

    /// JSON 形式で出力する
    #[arg(long)]
    pub json: bool,

    /// 自動修正を試みる
    #[arg(long)]
    pub fix: bool,
}

/// `k1s0 lint` を実行する
pub fn execute(args: LintArgs) -> Result<()> {
    println!("k1s0 lint");
    println!("  path: {}", args.path);
    if let Some(rules) = &args.rules {
        println!("  rules: {}", rules);
    }
    if let Some(exclude) = &args.exclude_rules {
        println!("  exclude_rules: {}", exclude);
    }
    println!("  strict: {}", args.strict);
    println!("  json: {}", args.json);
    println!("  fix: {}", args.fix);
    println!();
    println!("TODO: 実装予定（フェーズ14）");
    println!();
    println!("検査項目:");
    println!("  - manifest: .k1s0/manifest.json の存在・整合性");
    println!("  - structure: 必須ディレクトリ/ファイルの存在");
    println!("  - env-var: 環境変数参照の禁止");
    println!("  - secrets: config/*.yaml への機密値直書き禁止");
    println!("  - dependency: Clean Architecture の依存方向");

    Ok(())
}
