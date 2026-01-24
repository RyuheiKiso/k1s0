//! `k1s0 init` コマンド
//!
//! リポジトリを初期化し、.k1s0/ ディレクトリを作成する。

use anyhow::Result;
use clap::Args;

/// `k1s0 init` の引数
#[derive(Args, Debug)]
pub struct InitArgs {
    /// 初期化するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 既存の .k1s0/ を上書きする
    #[arg(short, long)]
    pub force: bool,

    /// テンプレートソース（local または registry URL）
    #[arg(long, default_value = "local")]
    pub template_source: String,
}

/// `k1s0 init` を実行する
pub fn execute(args: InitArgs) -> Result<()> {
    println!("k1s0 init");
    println!("  path: {}", args.path);
    println!("  force: {}", args.force);
    println!("  template_source: {}", args.template_source);
    println!();
    println!("TODO: 実装予定（フェーズ11）");
    println!();
    println!("実行内容:");
    println!("  1. .k1s0/ ディレクトリを作成");
    println!("  2. 初期設定ファイルを生成");
    println!("  3. プロジェクト共通テンプレートを展開");

    Ok(())
}
