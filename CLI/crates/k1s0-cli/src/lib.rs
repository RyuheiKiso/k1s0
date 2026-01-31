//! k1s0 CLI ライブラリ
//!
//! このクレートは k1s0 CLI のコア機能を提供します。
//!
//! # コマンド
//!
//! - `init`: リポジトリ初期化
//! - `new-feature`: 新規サービスの雛形生成
//! - `new-screen`: 画面の雛形生成
//! - `lint`: 規約違反の検査
//! - `upgrade`: テンプレート更新
//!
//! # 設定ファイル
//!
//! `.k1s0/settings.yaml` でプロジェクト設定を管理できます。

use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;

pub mod commands;
pub mod doctor;
pub mod error;
pub mod output;
pub mod prompts;
pub mod settings;

pub use error::{CliError, ExitCode, Result};
pub use output::{init_output, output, Output, OutputConfig, OutputMode};
pub use settings::Settings;

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

    /// JSON 形式で出力する
    #[arg(long, global = true)]
    pub json: bool,
}

impl Cli {
    /// 出力設定を作成
    pub fn output_config(&self) -> OutputConfig {
        OutputConfig {
            mode: if self.json {
                OutputMode::Json
            } else {
                OutputMode::Human
            },
            color: !self.no_color,
            verbose: self.verbose,
        }
    }
}

/// 利用可能なサブコマンド
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// リポジトリを初期化し、.k1s0/ ディレクトリを作成する
    Init(commands::init::InitArgs),

    /// 新しい domain（業務領域共通ライブラリ）を作成する
    #[command(name = "new-domain")]
    NewDomain(commands::new_domain::NewDomainArgs),

    /// 新規サービスの雛形を生成する
    #[command(name = "new-feature")]
    NewFeature(commands::new_feature::NewFeatureArgs),

    /// 画面（Screen）の雛形を生成する
    #[command(name = "new-screen")]
    NewScreen(commands::new_screen::NewScreenArgs),

    /// 規約違反を検査する
    Lint(commands::lint::LintArgs),

    /// テンプレートの更新を確認・適用する
    Upgrade(commands::upgrade::UpgradeArgs),

    /// テンプレートレジストリを操作する
    Registry(commands::registry::RegistryArgs),

    /// シェル補完スクリプトを生成する
    Completions(commands::completions::CompletionsArgs),

    /// 開発環境の健全性をチェックする
    Doctor(commands::doctor::DoctorArgs),

    /// domain のバージョンを更新する
    #[command(name = "domain-version")]
    DomainVersion(commands::domain_version::DomainVersionArgs),

    /// 全ての domain を一覧表示する
    #[command(name = "domain-list")]
    DomainList(commands::domain_list::DomainListArgs),

    /// 指定した domain に依存する feature を一覧表示する
    #[command(name = "domain-dependents")]
    DomainDependents(commands::domain_dependents::DomainDependentsArgs),

    /// domain のバージョンアップによる影響を分析する
    #[command(name = "domain-impact")]
    DomainImpact(commands::domain_impact::DomainImpactArgs),

    /// feature の domain 依存バージョンを更新する
    #[command(name = "feature-update-domain")]
    FeatureUpdateDomain(commands::feature_update_domain::FeatureUpdateDomainArgs),

    /// ドメインカタログ（一覧 + 依存状況）を表示する
    #[command(name = "domain-catalog")]
    DomainCatalog(commands::domain_catalog::DomainCatalogArgs),

    /// ドメイン依存グラフを出力する
    #[command(name = "domain-graph")]
    DomainGraph(commands::domain_graph::DomainGraphArgs),

    /// Docker イメージのビルドや docker-compose の操作を支援する
    Docker(commands::docker::DockerArgs),

    /// playground 環境の起動・停止
    Playground(commands::playground::PlaygroundArgs),

    /// 既存プロジェクトを k1s0 構造に移行する
    Migrate(commands::migrate::MigrateArgs),
}
