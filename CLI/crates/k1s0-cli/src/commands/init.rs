//! `k1s0 init` コマンド
//!
//! リポジトリを初期化し、.k1s0/ ディレクトリを作成する。

use clap::Args;

use crate::error::Result;
use crate::output::output;

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
    let out = output();

    out.header("k1s0 init");
    out.newline();

    out.list_item("path", &args.path);
    out.list_item("force", &args.force.to_string());
    out.list_item("template_source", &args.template_source);
    out.newline();

    out.info("TODO: 実装予定（フェーズ11）");
    out.newline();

    out.header("実行内容:");
    out.hint("1. .k1s0/ ディレクトリを作成");
    out.hint("2. 初期設定ファイルを生成");
    out.hint("3. プロジェクト共通テンプレートを展開");

    Ok(())
}
