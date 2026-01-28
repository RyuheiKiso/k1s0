//! `k1s0 init` コマンド
//!
//! リポジトリを初期化し、.k1s0/ ディレクトリを作成する。

use std::path::PathBuf;

use chrono::Utc;
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::error::{CliError, Result};
use crate::output::output;
use crate::prompts;
use crate::version;

/// .k1s0 ディレクトリ名
const K1S0_DIR: &str = ".k1s0";

/// config.json ファイル名
const CONFIG_FILE: &str = "config.json";

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

    /// 対話モードを強制する
    #[arg(short = 'i', long)]
    pub interactive: bool,
}

/// プロジェクト設定（.k1s0/config.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// スキーマバージョン
    pub schema_version: String,

    /// k1s0 バージョン
    pub k1s0_version: String,

    /// テンプレートソース
    pub template_source: String,

    /// 初期化日時
    pub initialized_at: String,

    /// プロジェクト設定
    #[serde(default)]
    pub project: ProjectSettings,
}

/// プロジェクト設定の詳細
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// デフォルトの言語
    #[serde(default = "default_language")]
    pub default_language: String,

    /// デフォルトのサービスタイプ
    #[serde(default = "default_service_type")]
    pub default_service_type: String,
}

fn default_language() -> String {
    "rust".to_string()
}

fn default_service_type() -> String {
    "backend".to_string()
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            default_language: default_language(),
            default_service_type: default_service_type(),
        }
    }
}

impl ProjectConfig {
    /// 新しいプロジェクト設定を作成
    pub fn new(k1s0_version: &str, template_source: &str) -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            k1s0_version: k1s0_version.to_string(),
            template_source: template_source.to_string(),
            initialized_at: Utc::now().to_rfc3339(),
            project: ProjectSettings::default(),
        }
    }

    /// カスタム設定で新しいプロジェクト設定を作成
    pub fn with_settings(
        k1s0_version: &str,
        template_source: &str,
        default_language: &str,
        default_service_type: &str,
    ) -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            k1s0_version: k1s0_version.to_string(),
            template_source: template_source.to_string(),
            initialized_at: Utc::now().to_rfc3339(),
            project: ProjectSettings {
                default_language: default_language.to_string(),
                default_service_type: default_service_type.to_string(),
            },
        }
    }

    /// ファイルに保存
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// ファイルから読み込み
    #[allow(dead_code)]
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
}

/// 解決済みの引数（対話入力後）
struct ResolvedArgs {
    path: String,
    force: bool,
    template_source: String,
    default_language: String,
    default_service_type: String,
}

/// `k1s0 init` を実行する
pub fn execute(args: InitArgs) -> Result<()> {
    // 対話モードを判定（init はデフォルト値があるので --interactive でのみ対話モード起動）
    let use_interactive = args.interactive && prompts::is_interactive();

    if args.interactive && !prompts::is_interactive() {
        return Err(CliError::interactive_required(
            "対話モードが要求されましたが、TTY が利用できません",
        ));
    }

    // 引数を解決
    let resolved = if use_interactive {
        resolve_args_interactive(args)?
    } else {
        resolve_args_from_cli(args)
    };

    // 初期化を実行
    execute_init(resolved)
}

/// CLI 引数から解決済み引数を構築
fn resolve_args_from_cli(args: InitArgs) -> ResolvedArgs {
    ResolvedArgs {
        path: args.path,
        force: args.force,
        template_source: args.template_source,
        default_language: default_language(),
        default_service_type: default_service_type(),
    }
}

/// 対話モードで引数を解決
fn resolve_args_interactive(args: InitArgs) -> Result<ResolvedArgs> {
    let out = output();

    // バナー表示
    out.header("k1s0 init");
    out.newline();
    out.info("対話モードでプロジェクトを初期化します");
    out.newline();

    // 1. path が "." の場合のみプロンプト（既にパスが指定されている場合はそのまま使用）
    let path = if args.path == "." {
        prompts::init_options::input_init_path()?
    } else {
        args.path
    };

    // 2. template_source が "local" の場合のみプロンプト
    let template_source = if args.template_source == "local" {
        prompts::init_options::input_template_source()?
    } else {
        args.template_source
    };

    // 3. デフォルト言語を選択
    let default_language = prompts::init_options::select_language()?;

    // 4. デフォルトサービスタイプを選択
    let default_service_type = prompts::init_options::select_service_type()?;

    out.newline();

    Ok(ResolvedArgs {
        path,
        force: args.force,
        template_source,
        default_language,
        default_service_type,
    })
}

/// 初期化を実行する
fn execute_init(args: ResolvedArgs) -> Result<()> {
    let out = output();

    out.header("k1s0 init");
    out.newline();

    // パスを正規化
    let base_path = PathBuf::from(&args.path);
    let base_path = if base_path.is_absolute() {
        base_path
    } else {
        std::env::current_dir()?.join(&base_path)
    };

    let k1s0_dir = base_path.join(K1S0_DIR);
    let config_path = k1s0_dir.join(CONFIG_FILE);

    out.list_item("path", &base_path.display().to_string());
    out.list_item("k1s0 version", version());
    out.list_item("template_source", &args.template_source);
    out.list_item("default_language", &args.default_language);
    out.list_item("default_service_type", &args.default_service_type);
    out.newline();

    // 既存チェック
    if k1s0_dir.exists() {
        if args.force {
            out.warning(&format!(
                "既存の {} を削除します",
                k1s0_dir.display()
            ));
            std::fs::remove_dir_all(&k1s0_dir)?;
        } else {
            return Err(CliError::conflict(format!(
                "{} は既に存在します",
                k1s0_dir.display()
            ))
            .with_target(k1s0_dir.display().to_string())
            .with_hint("--force オプションで上書きするか、別のディレクトリを指定してください"));
        }
    }

    // .k1s0 ディレクトリを作成
    out.info(&format!("{} を作成中...", k1s0_dir.display()));
    std::fs::create_dir_all(&k1s0_dir)?;
    out.file_added(&format!("{}/", K1S0_DIR));

    // config.json を作成
    let config = ProjectConfig::with_settings(
        version(),
        &args.template_source,
        &args.default_language,
        &args.default_service_type,
    );
    config.save(&config_path)?;
    out.file_added(&format!("{}/{}", K1S0_DIR, CONFIG_FILE));

    out.newline();
    out.success("初期化が完了しました");
    out.newline();

    out.header("次のステップ:");
    out.hint("k1s0 new-feature <name> でサービスの雛形を生成できます");

    Ok(())
}
