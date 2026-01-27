//! `k1s0 registry` コマンド
//!
//! テンプレートレジストリの操作を行う。
//!
//! # サブコマンド
//!
//! - `list`: テンプレート一覧を表示
//! - `info <name>`: テンプレートの詳細を表示
//! - `fetch <name>`: テンプレートをダウンロード
//! - `publish`: テンプレートを公開
//! - `cache clear`: キャッシュをクリア

use std::path::PathBuf;

use clap::{Args, Subcommand};
use k1s0_generator::registry::{
    LocalTemplateManager, RegistryClient, RegistryConfig, TemplateFilter,
};

use crate::error::{CliError, Result};
use crate::output::output;
use crate::settings::Settings;

/// `k1s0 registry` の引数
#[derive(Args, Debug)]
pub struct RegistryArgs {
    /// サブコマンド
    #[command(subcommand)]
    pub command: RegistryCommand,

    /// レジストリ URL を上書き
    #[arg(long, global = true)]
    pub registry_url: Option<String>,
}

/// レジストリサブコマンド
#[derive(Subcommand, Debug)]
pub enum RegistryCommand {
    /// テンプレート一覧を表示
    List(ListArgs),

    /// テンプレートの詳細を表示
    Info(InfoArgs),

    /// テンプレートをダウンロード
    Fetch(FetchArgs),

    /// テンプレートを公開
    Publish(PublishArgs),

    /// キャッシュ操作
    Cache(CacheArgs),
}

/// `registry list` の引数
#[derive(Args, Debug)]
pub struct ListArgs {
    /// 言語でフィルタ（rust, go, typescript, dart）
    #[arg(long)]
    pub language: Option<String>,

    /// サービスタイプでフィルタ（backend, frontend, bff）
    #[arg(long, name = "type")]
    pub service_type: Option<String>,

    /// 検索クエリ
    #[arg(long)]
    pub search: Option<String>,

    /// ローカルテンプレートのみ表示
    #[arg(long)]
    pub local: bool,
}

/// `registry info` の引数
#[derive(Args, Debug)]
pub struct InfoArgs {
    /// テンプレート名
    pub name: String,
}

/// `registry fetch` の引数
#[derive(Args, Debug)]
pub struct FetchArgs {
    /// テンプレート名
    pub name: String,

    /// バージョン
    #[arg(long, default_value = "latest")]
    pub version: String,

    /// 出力ディレクトリ
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}

/// `registry publish` の引数
#[derive(Args, Debug)]
pub struct PublishArgs {
    /// テンプレートディレクトリ
    #[arg(default_value = ".")]
    pub path: String,

    /// 認証トークン
    #[arg(long, env = "K1S0_REGISTRY_TOKEN")]
    pub token: Option<String>,
}

/// `registry cache` の引数
#[derive(Args, Debug)]
pub struct CacheArgs {
    /// サブコマンド
    #[command(subcommand)]
    pub command: CacheCommand,
}

/// キャッシュサブコマンド
#[derive(Subcommand, Debug)]
pub enum CacheCommand {
    /// キャッシュをクリア
    Clear,
    /// キャッシュの場所を表示
    Path,
}

/// `k1s0 registry` を実行する
pub fn execute(args: RegistryArgs) -> Result<()> {
    match args.command {
        RegistryCommand::List(list_args) => execute_list(list_args, args.registry_url),
        RegistryCommand::Info(info_args) => execute_info(info_args, args.registry_url),
        RegistryCommand::Fetch(fetch_args) => execute_fetch(fetch_args, args.registry_url),
        RegistryCommand::Publish(publish_args) => execute_publish(publish_args, args.registry_url),
        RegistryCommand::Cache(cache_args) => execute_cache(cache_args),
    }
}

/// テンプレート一覧を表示
fn execute_list(args: ListArgs, registry_url: Option<String>) -> Result<()> {
    let out = output();

    if args.local {
        // ローカルテンプレート一覧
        out.header("ローカルテンプレート");
        out.newline();

        let templates_dir = get_local_templates_dir()?;
        let manager = LocalTemplateManager::new(&templates_dir);

        match manager.list_local_templates() {
            Ok(templates) => {
                if templates.is_empty() {
                    out.info("ローカルテンプレートはありません");
                } else {
                    for name in templates {
                        out.list_item("", &name);
                    }
                }
            }
            Err(e) => {
                out.warning(&format!("ローカルテンプレートの取得に失敗: {}", e));
            }
        }

        return Ok(());
    }

    // リモートテンプレート一覧
    out.header("テンプレート一覧");
    out.newline();

    let config = build_registry_config(registry_url)?;
    let client = RegistryClient::new(config);

    // フィルター構築
    let filter = {
        let mut f = TemplateFilter::new();
        if let Some(lang) = args.language {
            f = f.with_language(lang);
        }
        if let Some(stype) = args.service_type {
            f = f.with_service_type(stype);
        }
        if let Some(query) = args.search {
            f = f.with_search(query);
        }
        Some(f)
    };

    let response = client
        .list_templates(filter.as_ref())
        .map_err(|e| CliError::internal(format!("テンプレート一覧の取得に失敗: {}", e)))?;

    if response.templates.is_empty() {
        out.info("条件に一致するテンプレートはありません");
        return Ok(());
    }

    // テーブル形式で表示
    out.hint(&format!("{:20} {:10} {:10} {:8} {}", "名前", "バージョン", "言語", "タイプ", "説明"));
    out.hint(&"-".repeat(70));

    for template in &response.templates {
        out.hint(&format!(
            "{:20} {:10} {:10} {:8} {}",
            template.name,
            template.version,
            template.language,
            template.service_type,
            truncate_str(&template.description, 30)
        ));
    }

    out.newline();
    out.info(&format!("合計: {} 件", response.total));

    Ok(())
}

/// テンプレートの詳細を表示
fn execute_info(args: InfoArgs, registry_url: Option<String>) -> Result<()> {
    let out = output();

    let config = build_registry_config(registry_url)?;
    let client = RegistryClient::new(config);

    // まず一覧から取得を試みる（モックでも動作）
    let response = client
        .list_templates(None)
        .map_err(|e| CliError::internal(format!("テンプレート情報の取得に失敗: {}", e)))?;

    let template = response
        .templates
        .iter()
        .find(|t| t.name == args.name)
        .ok_or_else(|| CliError::validation(format!("テンプレート '{}' が見つかりません", args.name)))?;

    out.header(&format!("テンプレート: {}", template.name));
    out.newline();

    out.list_item("バージョン", &template.version);
    out.list_item("言語", &template.language);
    out.list_item("タイプ", &template.service_type);
    out.list_item("作者", &template.author);
    out.list_item("説明", &template.description);
    out.newline();

    if !template.tags.is_empty() {
        out.list_item("タグ", &template.tags.join(", "));
    }

    out.list_item("作成日時", &template.created_at);
    out.list_item("更新日時", &template.updated_at);
    out.list_item("ダウンロード数", &template.downloads.to_string());

    if !template.variables.is_empty() {
        out.newline();
        out.header("変数:");

        for var in &template.variables {
            let required = if var.required { " (必須)" } else { "" };
            let default = var
                .default
                .as_ref()
                .map(|d| format!(" [デフォルト: {}]", d))
                .unwrap_or_default();

            out.hint(&format!(
                "{}: {}{}{}",
                var.name, var.description, required, default
            ));
        }
    }

    Ok(())
}

/// テンプレートをダウンロード
fn execute_fetch(args: FetchArgs, registry_url: Option<String>) -> Result<()> {
    let out = output();

    out.header(&format!("テンプレートをダウンロード: {}@{}", args.name, args.version));
    out.newline();

    let config = build_registry_config(registry_url)?;
    let client = RegistryClient::new(config);

    match client.fetch_template(&args.name, Some(&args.version)) {
        Ok(path) => {
            out.success(&format!("ダウンロード完了: {}", path.display()));
        }
        Err(e) => {
            out.warning(&format!("ダウンロードに失敗: {}", e));
            out.hint("この機能はリモートレジストリが実装されると利用可能になります");
        }
    }

    Ok(())
}

/// テンプレートを公開
fn execute_publish(args: PublishArgs, registry_url: Option<String>) -> Result<()> {
    let out = output();
    let path = PathBuf::from(&args.path);

    if !path.exists() {
        return Err(CliError::io("指定されたパスが存在しません")
            .with_target(&args.path));
    }

    out.header("テンプレートを公開");
    out.newline();

    let mut config = build_registry_config(registry_url)?;
    config.auth_token = args.token;

    let client = RegistryClient::new(config);

    match client.publish_template(&path) {
        Ok(metadata) => {
            out.success(&format!("公開完了: {}@{}", metadata.name, metadata.version));
        }
        Err(e) => {
            out.warning(&format!("公開に失敗: {}", e));
            out.hint("この機能はリモートレジストリが実装されると利用可能になります");
        }
    }

    Ok(())
}

/// キャッシュ操作
fn execute_cache(args: CacheArgs) -> Result<()> {
    let out = output();
    let config = build_registry_config(None)?;

    match args.command {
        CacheCommand::Clear => {
            let client = RegistryClient::new(config.clone());
            client
                .clear_cache()
                .map_err(|e| CliError::io(format!("キャッシュのクリアに失敗: {}", e)))?;

            out.success("キャッシュをクリアしました");
        }
        CacheCommand::Path => {
            out.info(&config.cache_dir.display().to_string());
        }
    }

    Ok(())
}

/// レジストリ設定を構築
fn build_registry_config(registry_url: Option<String>) -> Result<RegistryConfig> {
    let settings = Settings::load(None).unwrap_or_default();

    let mut config = RegistryConfig::default();

    // 設定ファイルから読み込み
    if let Some(url) = settings.registry.url {
        config.url = url;
    }
    if let Some(cache_dir) = settings.registry.cache_dir {
        config.cache_dir = PathBuf::from(cache_dir);
    }
    if let Some(token) = settings.registry.auth_token {
        config.auth_token = Some(token);
    }
    config.timeout_secs = settings.registry.timeout_secs;

    // CLI 引数で上書き
    if let Some(url) = registry_url {
        config.url = url;
    }

    Ok(config)
}

/// ローカルテンプレートディレクトリを取得
fn get_local_templates_dir() -> Result<PathBuf> {
    // カレントディレクトリから .k1s0/templates を探す
    let cwd = std::env::current_dir()
        .map_err(|e| CliError::io(format!("カレントディレクトリの取得に失敗: {}", e)))?;

    let local_path = cwd.join(".k1s0/templates");
    if local_path.exists() {
        return Ok(local_path);
    }

    // デフォルトのテンプレートディレクトリ
    Ok(dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("k1s0/templates"))
}

/// 文字列を指定長で切り詰める
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
