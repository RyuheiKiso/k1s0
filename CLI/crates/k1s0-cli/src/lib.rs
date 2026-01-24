//! k1s0 CLI ライブラリ
//!
//! このクレートは k1s0 CLI のコア機能を提供します。
//!
//! # コマンド
//!
//! - `init`: リポジトリ初期化
//! - `new-feature`: 新規サービスの雛形生成
//! - `lint`: 規約違反の検査
//! - `upgrade`: テンプレート更新

use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;

pub mod commands;

/// k1s0 バージョン（k1s0-version.txt から取得）
static VERSION_STRING: Lazy<String> = Lazy::new(|| {
    include_str!("../../../../k1s0-version.txt").trim().to_string()
});

/// k1s0 バージョンを取得する
pub fn version() -> &'static str {
    &VERSION_STRING
}

/// k1s0 CLI - 雛形生成・導入・アップグレード支援ツール
#[derive(Parser, Debug)]
#[command(
    name = "k1s0",
    version = version(),
    author = "k1s0 Team",
    about = "k1s0 - 高速な開発サイクルを実現する framework / templates / CLI",
    long_about = "k1s0 は framework / templates / CLI を含むモノレポの開発支援ツールです。\n\n\
                  サービスの雛形生成、規約チェック、アップグレード支援などの機能を提供します。"
)]
pub struct Cli {
    /// サブコマンド
    #[command(subcommand)]
    pub command: Commands,

    /// 詳細な出力を有効にする
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// カラー出力を無効にする
    #[arg(long, global = true)]
    pub no_color: bool,
}

/// 利用可能なサブコマンド
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// リポジトリを初期化し、.k1s0/ ディレクトリを作成する
    Init(commands::init::InitArgs),

    /// 新規サービスの雛形を生成する
    #[command(name = "new-feature")]
    NewFeature(commands::new_feature::NewFeatureArgs),

    /// 規約違反を検査する
    Lint(commands::lint::LintArgs),

    /// テンプレートの更新を確認・適用する
    Upgrade(commands::upgrade::UpgradeArgs),

    /// シェル補完スクリプトを生成する
    Completions(commands::completions::CompletionsArgs),
}
