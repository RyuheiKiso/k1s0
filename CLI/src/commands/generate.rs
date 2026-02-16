use anyhow::Result;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::CliConfig;
use crate::prompt::{self, ConfirmResult};
use crate::template::context::TemplateContextBuilder;
use crate::template::TemplateEngine;

// ============================================================================
// 種別
// ============================================================================

/// 生成する種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Server,
    Client,
    Library,
    Database,
}

impl Kind {
    pub fn label(&self) -> &'static str {
        match self {
            Kind::Server => "サーバー",
            Kind::Client => "クライアント",
            Kind::Library => "ライブラリ",
            Kind::Database => "データベース",
        }
    }

    /// 選択可能なTier一覧を返す。
    pub fn available_tiers(&self) -> Vec<Tier> {
        match self {
            Kind::Server => vec![Tier::System, Tier::Business, Tier::Service],
            Kind::Client => vec![Tier::Business, Tier::Service],
            Kind::Library => vec![Tier::System, Tier::Business],
            Kind::Database => vec![Tier::System, Tier::Business, Tier::Service],
        }
    }
}

const KIND_LABELS: &[&str] = &["サーバー", "クライアント", "ライブラリ", "データベース"];
const ALL_KINDS: &[Kind] = &[Kind::Server, Kind::Client, Kind::Library, Kind::Database];

// ============================================================================
// Tier
// ============================================================================

/// Tier種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    System,
    Business,
    Service,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::System => "system",
            Tier::Business => "business",
            Tier::Service => "service",
        }
    }

    pub fn label(&self) -> &'static str {
        self.as_str()
    }
}

// ============================================================================
// 言語 / フレームワーク
// ============================================================================

/// サーバー・ライブラリの言語選択。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Go,
    Rust,
    TypeScript,
    Dart,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Go => "Go",
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::Dart => "Dart",
        }
    }

    pub fn dir_name(&self) -> &'static str {
        match self {
            Language::Go => "go",
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Dart => "dart",
        }
    }
}

/// クライアントのフレームワーク選択。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framework {
    React,
    Flutter,
}

impl Framework {
    pub fn as_str(&self) -> &'static str {
        match self {
            Framework::React => "React",
            Framework::Flutter => "Flutter",
        }
    }

    pub fn dir_name(&self) -> &'static str {
        match self {
            Framework::React => "react",
            Framework::Flutter => "flutter",
        }
    }
}

// ============================================================================
// API方式
// ============================================================================

/// API方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiStyle {
    Rest,
    Grpc,
    GraphQL,
}

impl ApiStyle {
    pub fn short_label(&self) -> &'static str {
        match self {
            ApiStyle::Rest => "REST",
            ApiStyle::Grpc => "gRPC",
            ApiStyle::GraphQL => "GraphQL",
        }
    }
}

const API_LABELS: &[&str] = &["REST (OpenAPI)", "gRPC (protobuf)", "GraphQL"];
const ALL_API_STYLES: &[ApiStyle] = &[ApiStyle::Rest, ApiStyle::Grpc, ApiStyle::GraphQL];

// ============================================================================
// RDBMS
// ============================================================================

/// RDBMS種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rdbms {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl Rdbms {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rdbms::PostgreSQL => "PostgreSQL",
            Rdbms::MySQL => "MySQL",
            Rdbms::SQLite => "SQLite",
        }
    }
}

const RDBMS_LABELS: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
const ALL_RDBMS: &[Rdbms] = &[Rdbms::PostgreSQL, Rdbms::MySQL, Rdbms::SQLite];

// ============================================================================
// DB情報
// ============================================================================

/// データベース情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbInfo {
    pub name: String,
    pub rdbms: Rdbms,
}

impl fmt::Display for DbInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.rdbms.as_str())
    }
}

// ============================================================================
// 生成設定
// ============================================================================

/// ひな形生成の設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateConfig {
    /// 種別
    pub kind: Kind,
    /// Tier
    pub tier: Tier,
    /// 配置先 (business: 領域名, service: サービス名)
    pub placement: Option<String>,
    /// 言語・FW の選択結果
    pub lang_fw: LangFw,
    /// 詳細設定
    pub detail: DetailConfig,
}

/// 言語/FW 列挙
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangFw {
    Language(Language),
    Framework(Framework),
    Database { name: String, rdbms: Rdbms },
}

/// 詳細設定
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailConfig {
    /// サーバー: サービス名 / クライアント: アプリ名 / ライブラリ: ライブラリ名
    pub name: Option<String>,
    /// サーバー: API方式
    pub api_styles: Vec<ApiStyle>,
    /// サーバー: DB設定
    pub db: Option<DbInfo>,
    /// サーバー: Kafka有効
    pub kafka: bool,
    /// サーバー: Redis有効
    pub redis: bool,
}

impl Default for DetailConfig {
    fn default() -> Self {
        Self {
            name: None,
            api_styles: Vec::new(),
            db: None,
            kafka: false,
            redis: false,
        }
    }
}

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
/// LangFw ステップから Esc で戻るときの戻り先を Tier にする。
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

            Step::Detail => match step_detail(kind, tier, &placement, &lang_fw)? {
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
                        execute_generate(&config)?;
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

// ============================================================================
// 各ステップ
// ============================================================================

enum StepResult<T> {
    Value(T),
    Skip,
    Back,
}

/// ステップ1: 種別選択
fn step_kind() -> Result<Option<Kind>> {
    let idx = prompt::select_prompt("何を生成しますか？", KIND_LABELS)?;
    Ok(idx.map(|i| ALL_KINDS[i]))
}

/// ステップ2: Tier選択
fn step_tier(kind: Kind) -> Result<Option<Tier>> {
    let available = kind.available_tiers();
    let labels: Vec<&str> = available.iter().map(|t| t.label()).collect();
    let idx = prompt::select_prompt("Tier を選択してください", &labels)?;
    Ok(idx.map(|i| available[i]))
}

/// ステップ3: 配置先指定
///
/// Tier::System の場合は配置先不要のためスキップ (StepResult::Skip)。
/// Esc が押された場合は StepResult::Back を返す。
fn step_placement(tier: Tier) -> Result<StepResult<Option<String>>> {
    match tier {
        Tier::System => Ok(StepResult::Skip),
        Tier::Business => {
            let existing = scan_existing_dirs("regions/business");
            let name = prompt_name_or_select(
                "領域名を入力または選択してください",
                "領域名を入力してください",
                &existing,
            )?;
            match name {
                Some(n) => Ok(StepResult::Value(Some(n))),
                None => Ok(StepResult::Back),
            }
        }
        Tier::Service => {
            let existing = scan_existing_dirs("regions/service");
            let name = prompt_name_or_select(
                "サービス名を入力または選択してください",
                "サービス名を入力してください",
                &existing,
            )?;
            match name {
                Some(n) => Ok(StepResult::Value(Some(n))),
                None => Ok(StepResult::Back),
            }
        }
    }
}

/// ステップ4: 言語/FW選択
fn step_lang_fw(kind: Kind) -> Result<Option<LangFw>> {
    match kind {
        Kind::Server => {
            let items = &["Go", "Rust"];
            let idx = prompt::select_prompt("言語を選択してください", items)?;
            Ok(idx.map(|i| {
                LangFw::Language(match i {
                    0 => Language::Go,
                    1 => Language::Rust,
                    _ => unreachable!(),
                })
            }))
        }
        Kind::Client => {
            let items = &["React", "Flutter"];
            let idx = prompt::select_prompt("フレームワークを選択してください", items)?;
            Ok(idx.map(|i| {
                LangFw::Framework(match i {
                    0 => Framework::React,
                    1 => Framework::Flutter,
                    _ => unreachable!(),
                })
            }))
        }
        Kind::Library => {
            let items = &["Go", "Rust", "TypeScript", "Dart"];
            let idx = prompt::select_prompt("言語を選択してください", items)?;
            Ok(idx.map(|i| {
                LangFw::Language(match i {
                    0 => Language::Go,
                    1 => Language::Rust,
                    2 => Language::TypeScript,
                    3 => Language::Dart,
                    _ => unreachable!(),
                })
            }))
        }
        Kind::Database => {
            let db_name = prompt::input_prompt("データベース名を入力してください");
            match db_name {
                Ok(name) => {
                    let idx = prompt::select_prompt("RDBMS を選択してください", RDBMS_LABELS)?;
                    match idx {
                        Some(i) => Ok(Some(LangFw::Database {
                            name,
                            rdbms: ALL_RDBMS[i],
                        })),
                        None => Ok(None),
                    }
                }
                Err(_) => Ok(None),
            }
        }
    }
}

/// ステップ5: 詳細設定
fn step_detail(
    kind: Kind,
    tier: Tier,
    placement: &Option<String>,
    _lang_fw: &LangFw,
) -> Result<Option<DetailConfig>> {
    match kind {
        Kind::Server => step_detail_server(tier, placement),
        Kind::Client => step_detail_client(tier, placement),
        Kind::Library => step_detail_library(),
        Kind::Database => Ok(Some(DetailConfig::default())),
    }
}

/// サーバー詳細設定
fn step_detail_server(
    tier: Tier,
    placement: &Option<String>,
) -> Result<Option<DetailConfig>> {
    // サービス名: service Tier ではステップ3 のサービス名を使う
    let service_name = if tier == Tier::Service {
        placement.clone()
    } else {
        match prompt::input_prompt("サービス名を入力してください") {
            Ok(n) => Some(n),
            Err(_) => return Ok(None),
        }
    };

    // API方式
    let api_indices = prompt::multi_select_prompt(
        "API 方式を選択してください（複数選択可）",
        API_LABELS,
    )?;
    let api_styles: Vec<ApiStyle> = match api_indices {
        Some(indices) => indices.iter().map(|&i| ALL_API_STYLES[i]).collect(),
        None => return Ok(None),
    };

    // DB追加
    let add_db = prompt::yes_no_prompt("データベースを追加しますか？")?;
    let db = match add_db {
        Some(true) => {
            // 既存DBの探索
            let existing_dbs = scan_existing_databases();
            let db_info = prompt_db_selection(&existing_dbs)?;
            db_info
        }
        Some(false) => None,
        None => return Ok(None),
    };

    // Kafka
    let kafka = match prompt::yes_no_prompt("メッセージング (Kafka) を有効にしますか？")? {
        Some(v) => v,
        None => return Ok(None),
    };

    // Redis
    let redis = match prompt::yes_no_prompt("キャッシュ (Redis) を有効にしますか？")? {
        Some(v) => v,
        None => return Ok(None),
    };

    Ok(Some(DetailConfig {
        name: service_name,
        api_styles,
        db,
        kafka,
        redis,
    }))
}

/// クライアント詳細設定
fn step_detail_client(
    tier: Tier,
    placement: &Option<String>,
) -> Result<Option<DetailConfig>> {
    let app_name = if tier == Tier::Service {
        // service Tier: ステップ3のサービス名をアプリ名として使用
        placement.clone()
    } else {
        // business Tier: アプリ名入力
        match prompt::input_prompt("アプリ名を入力してください") {
            Ok(n) => Some(n),
            Err(_) => return Ok(None),
        }
    };

    Ok(Some(DetailConfig {
        name: app_name,
        ..DetailConfig::default()
    }))
}

/// ライブラリ詳細設定
fn step_detail_library() -> Result<Option<DetailConfig>> {
    let lib_name = match prompt::input_prompt("ライブラリ名を入力してください") {
        Ok(n) => n,
        Err(_) => return Ok(None),
    };

    Ok(Some(DetailConfig {
        name: Some(lib_name),
        ..DetailConfig::default()
    }))
}

// ============================================================================
// 確認表示
// ============================================================================

fn print_confirmation(config: &GenerateConfig) {
    println!("\n[確認] 以下の内容で生成します。よろしいですか？");
    println!("    種別:     {}", config.kind.label());
    println!("    Tier:     {}", config.tier.as_str());

    // 配置先
    if let Some(ref p) = config.placement {
        match config.tier {
            Tier::Business => println!("    領域:     {}", p),
            Tier::Service => println!("    サービス: {}", p),
            _ => {}
        }
    }

    match config.kind {
        Kind::Server => {
            // service Tier では placement で既にサービス名を表示済みのため、
            // detail.name の表示をスキップする
            if config.tier != Tier::Service {
                if let Some(ref name) = config.detail.name {
                    println!("    サービス: {}", name);
                }
            }
            if let LangFw::Language(lang) = config.lang_fw {
                println!("    言語:     {}", lang.as_str());
            }
            if !config.detail.api_styles.is_empty() {
                let api_strs: Vec<&str> =
                    config.detail.api_styles.iter().map(|a| a.short_label()).collect();
                println!("    API:      {}", api_strs.join(", "));
            }
            match &config.detail.db {
                Some(db) => println!("    DB:       {}", db),
                None => println!("    DB:       なし"),
            }
            println!(
                "    Kafka:    {}",
                if config.detail.kafka { "有効" } else { "無効" }
            );
            println!(
                "    Redis:    {}",
                if config.detail.redis { "有効" } else { "無効" }
            );
        }
        Kind::Client => {
            if let LangFw::Framework(fw) = config.lang_fw {
                println!("    フレームワーク: {}", fw.as_str());
            }
            if let Some(ref name) = config.detail.name {
                println!("    アプリ名:       {}", name);
            }
        }
        Kind::Library => {
            if let LangFw::Language(lang) = config.lang_fw {
                println!("    言語:         {}", lang.as_str());
            }
            if let Some(ref name) = config.detail.name {
                println!("    ライブラリ名: {}", name);
            }
        }
        Kind::Database => {
            if let LangFw::Database { ref name, rdbms } = config.lang_fw {
                println!("    データベース名: {}", name);
                println!("    RDBMS:          {}", rdbms.as_str());
            }
        }
    }
}

// ============================================================================
// 生成実行
// ============================================================================

/// ひな形生成を実行する。
pub fn execute_generate(config: &GenerateConfig) -> Result<()> {
    execute_generate_with_config(config, &std::env::current_dir()?, &CliConfig::default())
}

/// 指定されたベースディレクトリを起点にひな形生成を実行する。
/// テンプレートエンジンを使わず、インライン生成のみ行う（テスト後方互換用）。
pub fn execute_generate_at(config: &GenerateConfig, base_dir: &Path) -> Result<()> {
    let output_path = build_output_path(config, base_dir);
    fs::create_dir_all(&output_path)?;

    match config.kind {
        Kind::Server => generate_server(config, &output_path)?,
        Kind::Client => generate_client(config, &output_path)?,
        Kind::Library => generate_library(config, &output_path)?,
        Kind::Database => generate_database(config, &output_path)?,
    }

    // service Tier + GraphQL の場合は BFF ディレクトリを追加生成
    if config.kind == Kind::Server
        && config.tier == Tier::Service
        && config.detail.api_styles.contains(&ApiStyle::GraphQL)
    {
        let bff_path = output_path.join("bff");
        fs::create_dir_all(&bff_path)?;
    }

    Ok(())
}

/// CliConfig を指定してひな形生成を実行する。
/// テンプレートエンジン + 後処理コマンド付き。
pub fn execute_generate_with_config(
    config: &GenerateConfig,
    base_dir: &Path,
    cli_config: &CliConfig,
) -> Result<()> {
    let output_path = build_output_path(config, base_dir);
    fs::create_dir_all(&output_path)?;

    // D-03: テンプレートエンジンによる生成を試み、失敗時はインライン生成にフォールバック
    let tpl_dir = resolve_template_dir(base_dir);
    let template_context = build_template_context(config, cli_config);
    let template_generated = if tpl_dir.exists() {
        try_generate_from_templates(config, &output_path, &tpl_dir, cli_config)
    } else {
        false
    };

    // テンプレートで生成できなかった場合はインライン生成にフォールバック
    if !template_generated {
        match config.kind {
            Kind::Server => generate_server(config, &output_path)?,
            Kind::Client => generate_client(config, &output_path)?,
            Kind::Library => generate_library(config, &output_path)?,
            Kind::Database => generate_database(config, &output_path)?,
        }
    }

    // service Tier + GraphQL の場合は BFF ディレクトリを追加生成
    if config.kind == Kind::Server
        && config.tier == Tier::Service
        && config.detail.api_styles.contains(&ApiStyle::GraphQL)
    {
        let bff_path = output_path.join("bff");
        fs::create_dir_all(&bff_path)?;
    }

    // Helm Chart 生成（server のみ）
    if config.kind == Kind::Server {
        let helm_output = build_helm_output_path(config, base_dir);
        if let Some(ref ctx) = template_context {
            let helm_tpl_dir = tpl_dir.join("helm");
            if helm_tpl_dir.exists() {
                match generate_helm_chart(&helm_tpl_dir, ctx, &helm_output) {
                    Ok(files) => {
                        println!("Helm Chart を生成しました: {} ファイル", files.len());
                    }
                    Err(e) => {
                        eprintln!("Helm Chart の生成に失敗しました: {}", e);
                    }
                }
            }
        }
    }

    // CI/CD ワークフロー生成（全 kind）
    {
        let cicd_output = build_cicd_output_path(config, base_dir);
        if let Some(ref ctx) = template_context {
            let cicd_tpl_dir = tpl_dir.join("cicd");
            if cicd_tpl_dir.exists() {
                match generate_cicd_workflows(&cicd_tpl_dir, ctx, config, &cicd_output) {
                    Ok(files) => {
                        println!("CI/CD ワークフローを生成しました: {} ファイル", files.len());
                    }
                    Err(e) => {
                        eprintln!("CI/CD ワークフローの生成に失敗しました: {}", e);
                    }
                }
            }
        }
    }

    // D-08: 後処理コマンドの実行（best-effort）
    run_post_processing(config, &output_path);

    Ok(())
}

/// テンプレートディレクトリのパスを解決する。
fn resolve_template_dir(base_dir: &Path) -> PathBuf {
    // まず CLI/templates/ を探す
    let cli_templates = base_dir.join("CLI").join("templates");
    if cli_templates.exists() {
        return cli_templates;
    }
    // CARGO_MANIFEST_DIR からの相対パスも試す（テスト時など）
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_templates = Path::new(&manifest_dir).join("templates");
        if manifest_templates.exists() {
            return manifest_templates;
        }
    }
    // 見つからない場合はデフォルトのパスを返す（exists() で false になる）
    cli_templates
}

/// テンプレートエンジンを使って生成を試みる。成功した場合 true を返す。
fn try_generate_from_templates(
    config: &GenerateConfig,
    output_path: &Path,
    template_dir: &Path,
    cli_config: &CliConfig,
) -> bool {
    let ctx = match build_template_context(config, cli_config) {
        Some(ctx) => ctx,
        None => return false,
    };

    // kind + language/framework に対応するテンプレートサブディレクトリの存在確認
    // client の場合はフレームワーク名 (react/flutter) でディレクトリを引く
    let sub_dir = if ctx.kind == "client" && !ctx.framework.is_empty() {
        &ctx.framework
    } else {
        &ctx.language
    };
    let kind_lang_dir = template_dir.join(&ctx.kind).join(sub_dir);
    if !kind_lang_dir.exists() {
        return false;
    }

    let mut engine = match TemplateEngine::new(template_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    match engine.render_to_dir(&ctx, output_path) {
        Ok(files) => !files.is_empty(),
        Err(_) => false,
    }
}

/// GenerateConfig から TemplateContext を構築する。
fn build_template_context(
    config: &GenerateConfig,
    cli_config: &CliConfig,
) -> Option<crate::template::context::TemplateContext> {
    let service_name = config.detail.name.as_deref().unwrap_or("service");
    let tier = config.tier.as_str();

    let (language, kind) = match config.kind {
        Kind::Server => {
            let lang = match &config.lang_fw {
                LangFw::Language(l) => l.dir_name(),
                _ => return None,
            };
            (lang, "server")
        }
        Kind::Client => {
            let lang = match &config.lang_fw {
                LangFw::Framework(Framework::React) => "typescript",
                LangFw::Framework(Framework::Flutter) => "dart",
                _ => return None,
            };
            (lang, "client")
        }
        Kind::Library => {
            let lang = match &config.lang_fw {
                LangFw::Language(l) => l.dir_name(),
                _ => return None,
            };
            (lang, "library")
        }
        Kind::Database => {
            let db_type = match &config.lang_fw {
                LangFw::Database { rdbms, .. } => match rdbms {
                    Rdbms::PostgreSQL => "postgresql",
                    Rdbms::MySQL => "mysql",
                    Rdbms::SQLite => "sqlite",
                },
                _ => return None,
            };
            (db_type, "database")
        }
    };

    let api_styles_strs: Vec<String> = config
        .detail
        .api_styles
        .iter()
        .map(|a| match a {
            ApiStyle::Rest => "rest".to_string(),
            ApiStyle::Grpc => "grpc".to_string(),
            ApiStyle::GraphQL => "graphql".to_string(),
        })
        .collect();

    let fw_name = match &config.lang_fw {
        LangFw::Framework(Framework::React) => "react",
        LangFw::Framework(Framework::Flutter) => "flutter",
        _ => "",
    };

    let mut builder = TemplateContextBuilder::new(service_name, tier, language, kind)
        .framework(fw_name)
        .api_styles(api_styles_strs)
        .docker_registry(&cli_config.docker_registry)
        .go_module_base(&cli_config.go_module_base);

    if let Some(ref db) = config.detail.db {
        let db_type = match db.rdbms {
            Rdbms::PostgreSQL => "postgresql",
            Rdbms::MySQL => "mysql",
            Rdbms::SQLite => "sqlite",
        };
        builder = builder.with_database(db_type);
    }

    if config.detail.kafka {
        builder = builder.with_kafka();
    }

    if config.detail.redis {
        builder = builder.with_redis();
    }

    Some(builder.build())
}

/// D-08: 後処理コマンドを実行する（best-effort、最大3回リトライ）。
fn run_post_processing(config: &GenerateConfig, output_path: &Path) {
    let commands = determine_post_commands(config);
    for (cmd, args) in &commands {
        let max_retries = 3;
        let mut success = false;
        for attempt in 1..=max_retries {
            match Command::new(cmd).args(args).current_dir(output_path).output() {
                Ok(output) => {
                    if output.status.success() {
                        success = true;
                        break;
                    }
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if attempt < max_retries {
                        eprintln!(
                            "後処理コマンド '{} {}' が失敗しました（{}/{} 回目）: {}",
                            cmd, args.join(" "), attempt, max_retries, stderr.trim()
                        );
                    } else {
                        eprintln!(
                            "後処理コマンド '{} {}' が {} 回のリトライ後も失敗しました: {}",
                            cmd, args.join(" "), max_retries, stderr.trim()
                        );
                        eprintln!("手動で実行してください: cd {} && {} {}", output_path.display(), cmd, args.join(" "));
                    }
                }
                Err(e) => {
                    if attempt < max_retries {
                        eprintln!(
                            "後処理コマンド '{} {}' の実行に失敗しました（{}/{} 回目）: {}",
                            cmd, args.join(" "), attempt, max_retries, e
                        );
                    } else {
                        eprintln!(
                            "後処理コマンド '{} {}' が {} 回のリトライ後も実行に失敗しました: {}",
                            cmd, args.join(" "), max_retries, e
                        );
                        eprintln!("手動で実行してください: cd {} && {} {}", output_path.display(), cmd, args.join(" "));
                    }
                }
            }
        }
        let _ = success; // best-effort: 成功/失敗に関わらず次のコマンドへ
    }
}

/// 後処理コマンドのリストを決定する。
///
/// テンプレートエンジン仕様.md の「生成後の後処理」セクションに準拠:
///   1. 言語固有の依存解決
///   2. コード生成 (buf generate / oapi-codegen / cargo xtask codegen)
///   3. SQL マイグレーション初期化 (DB ありの場合)
fn determine_post_commands(config: &GenerateConfig) -> Vec<(&'static str, Vec<&'static str>)> {
    let mut commands: Vec<(&str, Vec<&str>)> = Vec::new();

    match config.kind {
        Kind::Server => {
            // 1. 言語固有の依存解決
            match &config.lang_fw {
                LangFw::Language(Language::Go) => {
                    commands.push(("go", vec!["mod", "tidy"]));
                }
                LangFw::Language(Language::Rust) => {
                    commands.push(("cargo", vec!["check"]));
                }
                _ => {}
            }
            // 2. コード生成
            // gRPC の場合は buf generate
            if config.detail.api_styles.contains(&ApiStyle::Grpc) {
                commands.push(("buf", vec!["generate"]));
            }
            // REST (OpenAPI) の場合はコード生成
            if config.detail.api_styles.contains(&ApiStyle::Rest) {
                match &config.lang_fw {
                    LangFw::Language(Language::Go) => {
                        commands.push(("oapi-codegen", vec!["-generate", "types,server", "-package", "handler", "-o", "internal/handler/openapi.gen.go", "api/openapi/openapi.yaml"]));
                    }
                    LangFw::Language(Language::Rust) => {
                        commands.push(("cargo", vec!["xtask", "codegen"]));
                    }
                    _ => {}
                }
            }
            // GraphQL の場合は gqlgen generate
            if config.detail.api_styles.contains(&ApiStyle::GraphQL) {
                match &config.lang_fw {
                    LangFw::Language(Language::Go) => {
                        commands.push(("go", vec!["run", "github.com/99designs/gqlgen", "generate"]));
                    }
                    _ => {}
                }
            }
            // 3. DB ありの場合は SQL マイグレーション初期化
            if config.detail.db.is_some() {
                commands.push(("sqlx", vec!["database", "create"]));
            }
        }
        Kind::Client => {
            match &config.lang_fw {
                LangFw::Framework(Framework::React) => {
                    commands.push(("npm", vec!["install"]));
                }
                LangFw::Framework(Framework::Flutter) => {
                    commands.push(("flutter", vec!["pub", "get"]));
                }
                _ => {}
            }
        }
        Kind::Library => {
            match &config.lang_fw {
                LangFw::Language(Language::Go) => {
                    commands.push(("go", vec!["mod", "tidy"]));
                }
                LangFw::Language(Language::Rust) => {
                    commands.push(("cargo", vec!["check"]));
                }
                LangFw::Language(Language::TypeScript) => {
                    commands.push(("npm", vec!["install"]));
                }
                LangFw::Language(Language::Dart) => {
                    commands.push(("flutter", vec!["pub", "get"]));
                }
                _ => {}
            }
        }
        Kind::Database => {
            // no post-processing for database
        }
    }

    commands
}

/// 出力先パスを構築する。
pub fn build_output_path(config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    let mut path = base_dir.join("regions");
    path.push(config.tier.as_str());

    // 配置先
    if let Some(ref placement) = config.placement {
        path.push(placement);
    }

    // 種別ディレクトリ
    match config.kind {
        Kind::Server => {
            path.push("server");
            if let LangFw::Language(lang) = config.lang_fw {
                path.push(lang.dir_name());
            }
            // system / business の場合はサービス名をサブディレクトリに
            if config.tier != Tier::Service {
                if let Some(ref name) = config.detail.name {
                    path.push(name);
                }
            }
        }
        Kind::Client => {
            path.push("client");
            if let LangFw::Framework(fw) = config.lang_fw {
                path.push(fw.dir_name());
            }
            // business の場合はアプリ名をサブディレクトリに
            if config.tier == Tier::Business {
                if let Some(ref name) = config.detail.name {
                    path.push(name);
                }
            }
        }
        Kind::Library => {
            path.push("library");
            if let LangFw::Language(lang) = config.lang_fw {
                path.push(lang.dir_name());
            }
            if let Some(ref name) = config.detail.name {
                path.push(name);
            }
        }
        Kind::Database => {
            path.push("database");
            if let LangFw::Database { ref name, .. } = config.lang_fw {
                path.push(name);
            }
        }
    }

    path
}

/// Helm Chart の出力先パスを構築する。
fn build_helm_output_path(config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    let mut path = base_dir.join("infra").join("helm").join("services");
    path.push(config.tier.as_str());

    // business Tier の場合はドメインディレクトリを挟む
    if config.tier == Tier::Business {
        if let Some(ref placement) = config.placement {
            path.push(placement);
        }
    }

    // サービス名
    if let Some(ref name) = config.detail.name {
        path.push(name);
    }

    path
}

/// CI/CD ワークフローの出力先パスを構築する。
fn build_cicd_output_path(_config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    base_dir.join(".github").join("workflows")
}

/// Helm Chart テンプレートをレンダリングする。
fn generate_helm_chart(
    helm_tpl_dir: &Path,
    ctx: &crate::template::context::TemplateContext,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut engine = TemplateEngine::new(helm_tpl_dir.parent().unwrap())?;
    // helm ディレクトリ直下のテンプレートを直接レンダリング
    let tera_ctx = ctx.to_tera_context();
    let mut generated = Vec::new();

    for entry in walkdir::WalkDir::new(helm_tpl_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|e| e.to_str()) != Some("tera") {
            continue;
        }

        let relative = path.strip_prefix(helm_tpl_dir)?;
        let template_content = fs::read_to_string(path)?;
        let template_name = relative.to_string_lossy().replace('\\', "/");

        engine.tera.add_raw_template(&template_name, &template_content)?;
        let rendered = engine.tera.render(&template_name, &tera_ctx)?;

        // .tera 拡張子を除去
        let output_relative = relative.to_string_lossy().replace('\\', "/");
        let output_relative = if output_relative.ends_with(".tera") {
            &output_relative[..output_relative.len() - 5]
        } else {
            &output_relative
        };
        let output_path = output_dir.join(output_relative);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, rendered)?;
        generated.push(output_path);
    }

    Ok(generated)
}

/// CI/CD ワークフローテンプレートをレンダリングする。
fn generate_cicd_workflows(
    cicd_tpl_dir: &Path,
    ctx: &crate::template::context::TemplateContext,
    config: &GenerateConfig,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let tera_ctx = ctx.to_tera_context();
    let mut generated = Vec::new();
    let mut tera = tera::Tera::default();
    crate::template::filters::register_filters(&mut tera);

    // CI ワークフロー（全 kind）
    let ci_template = cicd_tpl_dir.join("ci.yaml.tera");
    if ci_template.exists() {
        let template_content = fs::read_to_string(&ci_template)?;
        tera.add_raw_template("ci.yaml", &template_content)?;
        let rendered = tera.render("ci.yaml", &tera_ctx)?;

        let service_name = config.detail.name.as_deref().unwrap_or("service");
        let output_path = output_dir.join(format!("{}-ci.yaml", service_name));
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, rendered)?;
        generated.push(output_path);
    }

    // Deploy ワークフロー（server のみ）
    if config.kind == Kind::Server {
        let deploy_template = cicd_tpl_dir.join("deploy.yaml.tera");
        if deploy_template.exists() {
            let template_content = fs::read_to_string(&deploy_template)?;
            tera.add_raw_template("deploy.yaml", &template_content)?;
            let rendered = tera.render("deploy.yaml", &tera_ctx)?;

            let service_name = config.detail.name.as_deref().unwrap_or("service");
            let output_path = output_dir.join(format!("{}-deploy.yaml", service_name));
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, rendered)?;
            generated.push(output_path);
        }
    }

    Ok(generated)
}

/// サーバーひな形を生成する。
fn generate_server(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let lang = match config.lang_fw {
        LangFw::Language(l) => l,
        _ => unreachable!(),
    };
    let service_name = config.detail.name.as_deref().unwrap_or("service");

    match lang {
        Language::Go => generate_go_server(output_path, service_name, config)?,
        Language::Rust => generate_rust_server(output_path, service_name, config)?,
        _ => unreachable!("サーバーの言語は Go/Rust のみ"),
    }

    Ok(())
}

fn generate_go_server(output_path: &Path, service_name: &str, config: &GenerateConfig) -> Result<()> {
    // cmd/
    let cmd_dir = output_path.join("cmd");
    fs::create_dir_all(&cmd_dir)?;
    fs::write(
        cmd_dir.join("main.go"),
        format!(
            r#"package main

import "fmt"

func main() {{
	fmt.Println("Starting {} server...")
}}
"#,
            service_name
        ),
    )?;

    // internal/
    let internal_dir = output_path.join("internal");
    fs::create_dir_all(internal_dir.join("handler"))?;
    fs::create_dir_all(internal_dir.join("service"))?;
    fs::create_dir_all(internal_dir.join("repository"))?;

    fs::write(
        internal_dir.join("handler/handler.go"),
        "package handler\n",
    )?;
    fs::write(
        internal_dir.join("service/service.go"),
        "package service\n",
    )?;
    fs::write(
        internal_dir.join("repository/repository.go"),
        "package repository\n",
    )?;

    // go.mod
    fs::write(
        output_path.join("go.mod"),
        format!("module {}\n\ngo 1.21\n", service_name),
    )?;

    // Dockerfile
    fs::write(output_path.join("Dockerfile"), generate_go_dockerfile(service_name))?;

    // API定義
    for api in &config.detail.api_styles {
        match api {
            ApiStyle::Rest => {
                let api_dir = output_path.join("api/openapi");
                fs::create_dir_all(&api_dir)?;
                fs::write(api_dir.join("openapi.yaml"), generate_openapi_stub(service_name))?;
            }
            ApiStyle::Grpc => {
                let proto_dir = output_path.join("api/proto");
                fs::create_dir_all(&proto_dir)?;
                fs::write(
                    proto_dir.join(format!("{}.proto", service_name)),
                    generate_proto_stub(service_name),
                )?;
            }
            ApiStyle::GraphQL => {
                let gql_dir = output_path.join("api/graphql");
                fs::create_dir_all(&gql_dir)?;
                fs::write(gql_dir.join("schema.graphql"), generate_graphql_stub(service_name))?;
            }
        }
    }

    Ok(())
}

fn generate_rust_server(output_path: &Path, service_name: &str, config: &GenerateConfig) -> Result<()> {
    // src/
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        src_dir.join("main.rs"),
        format!(
            r#"fn main() {{
    println!("Starting {} server...");
}}
"#,
            service_name
        ),
    )?;

    // Cargo.toml
    fs::write(
        output_path.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
            service_name
        ),
    )?;

    // Dockerfile
    fs::write(output_path.join("Dockerfile"), generate_rust_dockerfile(service_name))?;

    // API定義
    for api in &config.detail.api_styles {
        match api {
            ApiStyle::Rest => {
                let api_dir = output_path.join("api/openapi");
                fs::create_dir_all(&api_dir)?;
                fs::write(api_dir.join("openapi.yaml"), generate_openapi_stub(service_name))?;
            }
            ApiStyle::Grpc => {
                let proto_dir = output_path.join("api/proto");
                fs::create_dir_all(&proto_dir)?;
                fs::write(
                    proto_dir.join(format!("{}.proto", service_name)),
                    generate_proto_stub(service_name),
                )?;
            }
            ApiStyle::GraphQL => {
                let gql_dir = output_path.join("api/graphql");
                fs::create_dir_all(&gql_dir)?;
                fs::write(gql_dir.join("schema.graphql"), generate_graphql_stub(service_name))?;
            }
        }
    }

    Ok(())
}

/// クライアントひな形を生成する。
fn generate_client(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let fw = match config.lang_fw {
        LangFw::Framework(f) => f,
        _ => unreachable!(),
    };
    let app_name = config.detail.name.as_deref().unwrap_or("app");

    match fw {
        Framework::React => generate_react_client(output_path, app_name)?,
        Framework::Flutter => generate_flutter_client(output_path, app_name)?,
    }

    Ok(())
}

fn generate_react_client(output_path: &Path, app_name: &str) -> Result<()> {
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(
        output_path.join("package.json"),
        format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }}
}}
"#,
            app_name
        ),
    )?;

    fs::write(
        src_dir.join("App.tsx"),
        format!(
            r#"function App() {{
  return <div>{}</div>;
}}

export default App;
"#,
            app_name
        ),
    )?;

    fs::write(
        src_dir.join("main.tsx"),
        r#"import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"#,
    )?;

    fs::write(output_path.join("index.html"), format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head><meta charset="UTF-8"><title>{}</title></head>
<body><div id="root"></div><script type="module" src="/src/main.tsx"></script></body>
</html>
"#, app_name))?;

    Ok(())
}

fn generate_flutter_client(output_path: &Path, app_name: &str) -> Result<()> {
    let lib_dir = output_path.join("lib");
    fs::create_dir_all(&lib_dir)?;

    fs::write(
        output_path.join("pubspec.yaml"),
        format!(
            r#"name: {}
description: A Flutter application
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"

dependencies:
  flutter:
    sdk: flutter
"#,
            app_name
        ),
    )?;

    fs::write(
        lib_dir.join("main.dart"),
        format!(
            r#"import 'package:flutter/material.dart';

void main() {{
  runApp(const MyApp());
}}

class MyApp extends StatelessWidget {{
  const MyApp({{super.key}});

  @override
  Widget build(BuildContext context) {{
    return MaterialApp(
      title: '{}',
      home: const Scaffold(
        body: Center(child: Text('{}')),
      ),
    );
  }}
}}
"#,
            app_name, app_name
        ),
    )?;

    Ok(())
}

/// ライブラリひな形を生成する。
fn generate_library(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let lang = match config.lang_fw {
        LangFw::Language(l) => l,
        _ => unreachable!(),
    };
    let lib_name = config.detail.name.as_deref().unwrap_or("lib");

    match lang {
        Language::Go => {
            fs::write(
                output_path.join("go.mod"),
                format!("module {}\n\ngo 1.21\n", lib_name),
            )?;
            fs::write(
                output_path.join(format!("{}.go", lib_name.replace('-', "_"))),
                format!("package {}\n", lib_name.replace('-', "_")),
            )?;
            fs::write(
                output_path.join(format!("{}_test.go", lib_name.replace('-', "_"))),
                format!(
                    r#"package {}

import "testing"

func TestPlaceholder(t *testing.T) {{
	// TODO: implement
}}
"#,
                    lib_name.replace('-', "_")
                ),
            )?;
        }
        Language::Rust => {
            fs::write(
                output_path.join("Cargo.toml"),
                format!(
                    r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
"#,
                    lib_name
                ),
            )?;
            let src_dir = output_path.join("src");
            fs::create_dir_all(&src_dir)?;
            fs::write(
                src_dir.join("lib.rs"),
                r#"#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
            )?;
        }
        Language::TypeScript => {
            fs::write(
                output_path.join("package.json"),
                format!(
                    r#"{{
  "name": "{}",
  "version": "0.1.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc",
    "test": "vitest"
  }}
}}
"#,
                    lib_name
                ),
            )?;
            let src_dir = output_path.join("src");
            fs::create_dir_all(&src_dir)?;
            fs::write(src_dir.join("index.ts"), "export {};\n")?;
            fs::write(
                output_path.join("tsconfig.json"),
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "declaration": true,
    "outDir": "dist",
    "strict": true
  },
  "include": ["src"]
}
"#,
            )?;
        }
        Language::Dart => {
            fs::write(
                output_path.join("pubspec.yaml"),
                format!(
                    r#"name: {}
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"
"#,
                    lib_name
                ),
            )?;
            let lib_dir = output_path.join("lib");
            fs::create_dir_all(&lib_dir)?;
            fs::write(
                lib_dir.join(format!("{}.dart", lib_name.replace('-', "_"))),
                format!("library {};\n", lib_name.replace('-', "_")),
            )?;
        }
    }

    Ok(())
}

/// データベースひな形を生成する。
fn generate_database(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let (db_name, rdbms) = match &config.lang_fw {
        LangFw::Database { name, rdbms } => (name.as_str(), *rdbms),
        _ => unreachable!(),
    };

    let migrations_dir = output_path.join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    // D-11: seeds/ と schema/ ディレクトリを作成
    let seeds_dir = output_path.join("seeds");
    fs::create_dir_all(&seeds_dir)?;
    let schema_dir = output_path.join("schema");
    fs::create_dir_all(&schema_dir)?;

    // D-12: 3桁プレフィックスに修正 (000001_init -> 001_init)
    fs::write(
        migrations_dir.join("001_init.up.sql"),
        format!(
            "-- {} の初期マイグレーション ({})\n-- TODO: テーブル定義を追加\n",
            db_name,
            rdbms.as_str()
        ),
    )?;

    fs::write(
        migrations_dir.join("001_init.down.sql"),
        "-- ロールバック\n-- TODO: DROP TABLE 文を追加\n",
    )?;

    // 設定ファイル
    fs::write(
        output_path.join("database.yaml"),
        format!(
            r#"name: {}
rdbms: {}
"#,
            db_name,
            rdbms.as_str()
        ),
    )?;

    Ok(())
}

// ============================================================================
// ヘルパー関数
// ============================================================================

/// 既存ディレクトリを走査して名前一覧を返す。
fn scan_existing_dirs(base: &str) -> Vec<String> {
    let path = Path::new(base);
    if !path.is_dir() {
        return Vec::new();
    }
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// 既存データベースを走査する。
fn scan_existing_databases() -> Vec<DbInfo> {
    let mut dbs = Vec::new();
    let search_paths = &[
        "regions/system/database",
        "regions/business",
        "regions/service",
    ];

    for base in search_paths {
        scan_db_recursive(Path::new(base), &mut dbs);
    }

    dbs
}

fn scan_db_recursive(path: &Path, dbs: &mut Vec<DbInfo>) {
    if !path.is_dir() {
        return;
    }
    // database.yaml を探す
    let config_path = path.join("database.yaml");
    if config_path.is_file() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            // 簡易パース
            let mut name = String::new();
            let mut rdbms_str = String::new();
            for line in content.lines() {
                if let Some(v) = line.strip_prefix("name: ") {
                    name = v.trim().to_string();
                }
                if let Some(v) = line.strip_prefix("rdbms: ") {
                    rdbms_str = v.trim().to_string();
                }
            }
            if !name.is_empty() {
                let rdbms = match rdbms_str.as_str() {
                    "MySQL" => Rdbms::MySQL,
                    "SQLite" => Rdbms::SQLite,
                    _ => Rdbms::PostgreSQL,
                };
                dbs.push(DbInfo { name, rdbms });
            }
        }
    }

    // 再帰的にサブディレクトリを探索
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                scan_db_recursive(&entry.path(), dbs);
            }
        }
    }
}

/// 名前の入力 or 既存選択。
///
/// 新規作成の場合、既存ディレクトリとの重複チェックを行う。
fn prompt_name_or_select(
    select_prompt_text: &str,
    input_prompt_text: &str,
    existing: &[String],
) -> Result<Option<String>> {
    let mut items: Vec<&str> = vec!["(新規作成)"];
    for name in existing {
        items.push(name.as_str());
    }

    let idx = prompt::select_prompt(select_prompt_text, &items)?;
    match idx {
        None => Ok(None),
        Some(0) => {
            // 新規作成: 名前バリデーション + 重複チェック
            let existing_names: Vec<String> = existing.to_vec();
            match prompt::input_with_validation(input_prompt_text, move |input: &String| {
                // まず名前バリデーション
                prompt::validate_name(input)?;
                // 重複チェック
                if existing_names.iter().any(|n| n == input) {
                    return Err(format!("'{}' は既に存在します。別の名前を入力してください。", input));
                }
                Ok(())
            }) {
                Ok(name) => Ok(Some(name)),
                Err(_) => Ok(None),
            }
        }
        Some(i) => Ok(Some(existing[i - 1].clone())),
    }
}

/// DB選択 (既存 or 新規作成)。
fn prompt_db_selection(existing: &[DbInfo]) -> Result<Option<DbInfo>> {
    let mut items: Vec<String> = vec!["(新規作成)".to_string()];
    for db in existing {
        items.push(format!("{} ({})", db.name, db.rdbms.as_str()));
    }
    let items_ref: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

    let idx = prompt::select_prompt("データベース名を入力または選択してください", &items_ref)?;
    match idx {
        None => Ok(None),
        Some(0) => {
            // 新規作成
            let name = match prompt::input_prompt("データベース名を入力してください") {
                Ok(n) => n,
                Err(_) => return Ok(None),
            };
            let rdbms_idx = prompt::select_prompt("RDBMS を選択してください", RDBMS_LABELS)?;
            match rdbms_idx {
                Some(i) => Ok(Some(DbInfo {
                    name,
                    rdbms: ALL_RDBMS[i],
                })),
                None => Ok(None),
            }
        }
        Some(i) => Ok(Some(existing[i - 1].clone())),
    }
}

// --- テンプレートスタブ生成 ---

fn generate_go_dockerfile(service_name: &str) -> String {
    format!(
        r#"FROM golang:1.21-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /bin/{} ./cmd/

FROM alpine:3.19
COPY --from=builder /bin/{} /bin/{}
ENTRYPOINT ["/bin/{}"]
"#,
        service_name, service_name, service_name, service_name
    )
}

fn generate_rust_dockerfile(service_name: &str) -> String {
    format!(
        r#"FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/{} /usr/local/bin/{}
ENTRYPOINT ["{}"]
"#,
        service_name, service_name, service_name
    )
}

fn generate_openapi_stub(service_name: &str) -> String {
    format!(
        r#"openapi: "3.0.3"
info:
  title: {} API
  version: "0.1.0"
paths: {{}}
"#,
        service_name
    )
}

fn generate_proto_stub(service_name: &str) -> String {
    let pkg = service_name.replace('-', "_");
    format!(
        r#"syntax = "proto3";

package {};

service {}Service {{
  // TODO: RPC メソッドを定義
}}
"#,
        pkg, pkg
    )
}

fn generate_graphql_stub(service_name: &str) -> String {
    format!(
        r#"# {} GraphQL Schema

type Query {{
  hello: String!
}}
"#,
        service_name
    )
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- Kind ---

    #[test]
    fn test_kind_label() {
        assert_eq!(Kind::Server.label(), "サーバー");
        assert_eq!(Kind::Client.label(), "クライアント");
        assert_eq!(Kind::Library.label(), "ライブラリ");
        assert_eq!(Kind::Database.label(), "データベース");
    }

    #[test]
    fn test_kind_available_tiers_server() {
        let tiers = Kind::Server.available_tiers();
        assert_eq!(tiers, vec![Tier::System, Tier::Business, Tier::Service]);
    }

    #[test]
    fn test_kind_available_tiers_client() {
        let tiers = Kind::Client.available_tiers();
        assert_eq!(tiers, vec![Tier::Business, Tier::Service]);
    }

    #[test]
    fn test_kind_available_tiers_library() {
        let tiers = Kind::Library.available_tiers();
        assert_eq!(tiers, vec![Tier::System, Tier::Business]);
    }

    #[test]
    fn test_kind_available_tiers_database() {
        let tiers = Kind::Database.available_tiers();
        assert_eq!(tiers, vec![Tier::System, Tier::Business, Tier::Service]);
    }

    // --- Tier ---

    #[test]
    fn test_tier_as_str() {
        assert_eq!(Tier::System.as_str(), "system");
        assert_eq!(Tier::Business.as_str(), "business");
        assert_eq!(Tier::Service.as_str(), "service");
    }

    // --- Language ---

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Go.as_str(), "Go");
        assert_eq!(Language::Rust.as_str(), "Rust");
        assert_eq!(Language::TypeScript.as_str(), "TypeScript");
        assert_eq!(Language::Dart.as_str(), "Dart");
    }

    #[test]
    fn test_language_dir_name() {
        assert_eq!(Language::Go.dir_name(), "go");
        assert_eq!(Language::Rust.dir_name(), "rust");
        assert_eq!(Language::TypeScript.dir_name(), "typescript");
        assert_eq!(Language::Dart.dir_name(), "dart");
    }

    // --- Framework ---

    #[test]
    fn test_framework_as_str() {
        assert_eq!(Framework::React.as_str(), "React");
        assert_eq!(Framework::Flutter.as_str(), "Flutter");
    }

    #[test]
    fn test_framework_dir_name() {
        assert_eq!(Framework::React.dir_name(), "react");
        assert_eq!(Framework::Flutter.dir_name(), "flutter");
    }

    // --- ApiStyle ---

    #[test]
    fn test_api_style_labels() {
        assert_eq!(ApiStyle::Rest.short_label(), "REST");
        assert_eq!(ApiStyle::Grpc.short_label(), "gRPC");
        assert_eq!(ApiStyle::GraphQL.short_label(), "GraphQL");
    }

    // --- Rdbms ---

    #[test]
    fn test_rdbms_as_str() {
        assert_eq!(Rdbms::PostgreSQL.as_str(), "PostgreSQL");
        assert_eq!(Rdbms::MySQL.as_str(), "MySQL");
        assert_eq!(Rdbms::SQLite.as_str(), "SQLite");
    }

    // --- DbInfo ---

    #[test]
    fn test_db_info_display() {
        let db = DbInfo {
            name: "order-db".to_string(),
            rdbms: Rdbms::PostgreSQL,
        };
        assert_eq!(format!("{}", db), "order-db (PostgreSQL)");
    }

    // --- build_output_path ---

    #[test]
    fn test_build_output_path_server_system() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(path, PathBuf::from("regions/system/server/go/auth"));
    }

    #[test]
    fn test_build_output_path_server_business() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("ledger".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(
            path,
            PathBuf::from("regions/business/accounting/server/go/ledger")
        );
    }

    #[test]
    fn test_build_output_path_server_service() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        // service Tier では detail.name をサブディレクトリに追加しない
        assert_eq!(path, PathBuf::from("regions/service/order/server/go"));
    }

    #[test]
    fn test_build_output_path_client_business() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("accounting-web".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(
            path,
            PathBuf::from("regions/business/accounting/client/react/accounting-web")
        );
    }

    #[test]
    fn test_build_output_path_client_service() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("order".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(
            path,
            PathBuf::from("regions/service/order/client/react")
        );
    }

    #[test]
    fn test_build_output_path_library_system() {
        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("authlib".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(
            path,
            PathBuf::from("regions/system/library/go/authlib")
        );
    }

    #[test]
    fn test_build_output_path_database_system() {
        let config = GenerateConfig {
            kind: Kind::Database,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Database {
                name: "auth-db".to_string(),
                rdbms: Rdbms::PostgreSQL,
            },
            detail: DetailConfig::default(),
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(
            path,
            PathBuf::from("regions/system/database/auth-db")
        );
    }

    // --- execute_generate ---

    #[test]
    fn test_execute_generate_go_server() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/server/go/auth");

        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest, ApiStyle::Grpc],
                db: None,
                kafka: false,
                redis: false,
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("cmd/main.go").is_file());
        assert!(base.join("internal/handler/handler.go").is_file());
        assert!(base.join("go.mod").is_file());
        assert!(base.join("Dockerfile").is_file());
        assert!(base.join("api/openapi/openapi.yaml").is_file());
        assert!(base.join("api/proto/auth.proto").is_file());
    }

    #[test]
    fn test_execute_generate_rust_server() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/server/rust/auth");

        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("src/main.rs").is_file());
        assert!(base.join("Cargo.toml").is_file());
        assert!(base.join("Dockerfile").is_file());
    }

    #[test]
    fn test_execute_generate_react_client() {
        let tmp = TempDir::new().unwrap();
        let base = tmp
            .path()
            .join("regions/business/accounting/client/react/accounting-web");

        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("accounting-web".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("package.json").is_file());
        assert!(base.join("src/App.tsx").is_file());
        assert!(base.join("src/main.tsx").is_file());
        assert!(base.join("index.html").is_file());
    }

    #[test]
    fn test_execute_generate_flutter_client() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/service/order/client/flutter");

        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Framework(Framework::Flutter),
            detail: DetailConfig {
                name: Some("order".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("pubspec.yaml").is_file());
        assert!(base.join("lib/main.dart").is_file());
    }

    #[test]
    fn test_execute_generate_go_library() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/library/go/authlib");

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("authlib".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("go.mod").is_file());
        assert!(base.join("authlib.go").is_file());
        assert!(base.join("authlib_test.go").is_file());
    }

    #[test]
    fn test_execute_generate_rust_library() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/business/accounting/library/rust/ledger-lib");

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("ledger-lib".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("Cargo.toml").is_file());
        assert!(base.join("src/lib.rs").is_file());
    }

    #[test]
    fn test_execute_generate_typescript_library() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/library/typescript/utils");

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::TypeScript),
            detail: DetailConfig {
                name: Some("utils".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("package.json").is_file());
        assert!(base.join("src/index.ts").is_file());
        assert!(base.join("tsconfig.json").is_file());
    }

    #[test]
    fn test_execute_generate_dart_library() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/library/dart/my-lib");

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Dart),
            detail: DetailConfig {
                name: Some("my-lib".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("pubspec.yaml").is_file());
        assert!(base.join("lib/my_lib.dart").is_file());
    }

    #[test]
    fn test_execute_generate_database() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/database/auth-db");

        let config = GenerateConfig {
            kind: Kind::Database,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Database {
                name: "auth-db".to_string(),
                rdbms: Rdbms::PostgreSQL,
            },
            detail: DetailConfig::default(),
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("migrations/001_init.up.sql").is_file());
        assert!(base.join("migrations/001_init.down.sql").is_file());
        assert!(base.join("seeds").is_dir());
        assert!(base.join("schema").is_dir());
        assert!(base.join("database.yaml").is_file());
    }

    // --- scan_existing_dirs ---

    #[test]
    fn test_scan_existing_dirs_nonexistent() {
        let dirs = scan_existing_dirs("/nonexistent/path");
        assert!(dirs.is_empty());
    }

    #[test]
    fn test_scan_existing_dirs_with_dirs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("base/aaa")).unwrap();
        fs::create_dir_all(tmp.path().join("base/bbb")).unwrap();
        fs::write(tmp.path().join("base/file.txt"), "").unwrap();

        let dirs = scan_existing_dirs(tmp.path().join("base").to_str().unwrap());
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&"aaa".to_string()));
        assert!(dirs.contains(&"bbb".to_string()));
    }

    // --- detail_config default ---

    #[test]
    fn test_detail_config_default() {
        let d = DetailConfig::default();
        assert!(d.name.is_none());
        assert!(d.api_styles.is_empty());
        assert!(d.db.is_none());
        assert!(!d.kafka);
        assert!(!d.redis);
    }

    // --- graphql stub ---

    #[test]
    fn test_generate_graphql_stub() {
        let gql = generate_graphql_stub("order");
        assert!(gql.contains("order GraphQL Schema"));
        assert!(gql.contains("type Query"));
    }

    // --- openapi stub ---

    #[test]
    fn test_generate_openapi_stub() {
        let yaml = generate_openapi_stub("auth");
        assert!(yaml.contains("auth API"));
        assert!(yaml.contains("openapi:"));
    }

    // --- proto stub ---

    #[test]
    fn test_generate_proto_stub() {
        let proto = generate_proto_stub("order-api");
        assert!(proto.contains("package order_api"));
        assert!(proto.contains("service order_apiService"));
    }

    // --- D-11: seeds/ and schema/ directories ---

    #[test]
    fn test_database_creates_seeds_and_schema_dirs() {
        let tmp = TempDir::new().unwrap();
        let config = GenerateConfig {
            kind: Kind::Database,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Database {
                name: "order-db".to_string(),
                rdbms: Rdbms::MySQL,
            },
            detail: DetailConfig::default(),
        };
        let result = execute_generate_at(&config, tmp.path());
        assert!(result.is_ok());
        let base = tmp.path().join("regions/service/order/database/order-db");
        assert!(base.join("seeds").is_dir(), "seeds/ directory should exist");
        assert!(base.join("schema").is_dir(), "schema/ directory should exist");
    }

    // --- D-12: 3-digit migration prefix ---

    #[test]
    fn test_database_migration_3digit_prefix() {
        let tmp = TempDir::new().unwrap();
        let config = GenerateConfig {
            kind: Kind::Database,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Database {
                name: "test-db".to_string(),
                rdbms: Rdbms::PostgreSQL,
            },
            detail: DetailConfig::default(),
        };
        let result = execute_generate_at(&config, tmp.path());
        assert!(result.is_ok());
        let base = tmp.path().join("regions/system/database/test-db");
        // 3桁プレフィックスであること
        assert!(base.join("migrations/001_init.up.sql").is_file());
        assert!(base.join("migrations/001_init.down.sql").is_file());
        // 旧形式の6桁プレフィックスは存在しないこと
        assert!(!base.join("migrations/000001_init.up.sql").exists());
        assert!(!base.join("migrations/000001_init.down.sql").exists());
    }

    // --- D-04: api_styles Vec ---

    #[test]
    fn test_build_template_context_multiple_api_styles() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest, ApiStyle::Grpc],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cli_config = CliConfig::default();
        let ctx = build_template_context(&config, &cli_config).unwrap();
        assert_eq!(ctx.api_styles, vec!["rest".to_string(), "grpc".to_string()]);
        // 後方互換: api_style は先頭要素
        assert_eq!(ctx.api_style, "rest");
    }

    // --- D-08: post-processing command determination ---

    #[test]
    fn test_determine_post_commands_go_server() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(cmds.iter().any(|(c, _)| *c == "go"), "should include 'go mod tidy'");
        assert!(!cmds.iter().any(|(c, _)| *c == "buf"), "should not include 'buf generate' without gRPC");
    }

    #[test]
    fn test_determine_post_commands_go_server_with_grpc() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest, ApiStyle::Grpc],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(cmds.iter().any(|(c, _)| *c == "go"), "should include 'go mod tidy'");
        assert!(cmds.iter().any(|(c, _)| *c == "buf"), "should include 'buf generate' with gRPC");
    }

    #[test]
    fn test_determine_post_commands_rust_server() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(cmds.iter().any(|(c, _)| *c == "cargo"), "should include 'cargo check'");
    }

    #[test]
    fn test_determine_post_commands_react_client() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("web-app".to_string()),
                ..DetailConfig::default()
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(cmds.iter().any(|(c, _)| *c == "npm"), "should include 'npm install'");
    }

    #[test]
    fn test_determine_post_commands_flutter_client() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Framework(Framework::Flutter),
            detail: DetailConfig {
                name: Some("order".to_string()),
                ..DetailConfig::default()
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(cmds.iter().any(|(c, _)| *c == "flutter"), "should include 'flutter pub get'");
    }

    #[test]
    fn test_determine_post_commands_database_none() {
        let config = GenerateConfig {
            kind: Kind::Database,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Database {
                name: "test-db".to_string(),
                rdbms: Rdbms::PostgreSQL,
            },
            detail: DetailConfig::default(),
        };
        let cmds = determine_post_commands(&config);
        assert!(cmds.is_empty(), "database should have no post-processing commands");
    }

    // --- D-03: template engine wiring ---

    #[test]
    fn test_execute_generate_with_config_fallback() {
        // テンプレートディレクトリが存在しない場合はインライン生成にフォールバック
        let tmp = TempDir::new().unwrap();
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("test-svc".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cli_config = CliConfig::default();
        // テンプレートがなくてもインライン生成で成功する
        let result = execute_generate_with_config(&config, tmp.path(), &cli_config);
        assert!(result.is_ok());
        let base = tmp.path().join("regions/system/server/rust/test-svc");
        assert!(base.join("src/main.rs").is_file());
        assert!(base.join("Cargo.toml").is_file());
    }

    // --- D-09: CliConfig integration ---

    #[test]
    fn test_build_template_context_with_custom_config() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cli_config = CliConfig {
            docker_registry: "my-registry.io".to_string(),
            go_module_base: "github.com/myorg/myrepo".to_string(),
            ..CliConfig::default()
        };
        let ctx = build_template_context(&config, &cli_config).unwrap();
        assert_eq!(ctx.docker_registry, "my-registry.io");
        assert_eq!(ctx.go_module, "github.com/myorg/myrepo/regions/service/order/server/go");
    }

    // --- 後処理コマンド: REST (OpenAPI) コード生成 ---

    #[test]
    fn test_determine_post_commands_go_server_rest_openapi() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(
            cmds.iter().any(|(c, _)| *c == "oapi-codegen"),
            "Go + REST should include 'oapi-codegen'"
        );
    }

    #[test]
    fn test_determine_post_commands_rust_server_rest_openapi() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        // cargo check + cargo xtask codegen
        let cargo_cmds: Vec<_> = cmds.iter().filter(|(c, _)| *c == "cargo").collect();
        assert!(
            cargo_cmds.len() >= 2,
            "Rust + REST should include 'cargo check' and 'cargo xtask codegen', got {:?}",
            cargo_cmds
        );
        assert!(
            cmds.iter().any(|(c, args)| *c == "cargo" && args.contains(&"xtask")),
            "Rust + REST should include 'cargo xtask codegen'"
        );
    }

    // --- 後処理コマンド: DB有効時の SQL マイグレーション初期化 ---

    #[test]
    fn test_determine_post_commands_server_with_db() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: Some(DbInfo {
                    name: "order-db".to_string(),
                    rdbms: Rdbms::PostgreSQL,
                }),
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(
            cmds.iter().any(|(c, _)| *c == "sqlx"),
            "Server with DB should include 'sqlx database create'"
        );
    }

    #[test]
    fn test_determine_post_commands_server_without_db() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cmds = determine_post_commands(&config);
        assert!(
            !cmds.iter().any(|(c, _)| *c == "sqlx"),
            "Server without DB should not include 'sqlx'"
        );
    }

    // --- language / framework フィールド: クライアント生成時 ---

    #[test]
    fn test_build_template_context_react_client_language_framework() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("accounting-web".to_string()),
                ..DetailConfig::default()
            },
        };
        let cli_config = CliConfig::default();
        let ctx = build_template_context(&config, &cli_config).unwrap();
        assert_eq!(ctx.language, "typescript", "React client should have language=typescript");
        assert_eq!(ctx.framework, "react", "React client should have framework=react");
        assert_eq!(ctx.kind, "client");
    }

    #[test]
    fn test_build_template_context_flutter_client_language_framework() {
        let config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Framework(Framework::Flutter),
            detail: DetailConfig {
                name: Some("order-app".to_string()),
                ..DetailConfig::default()
            },
        };
        let cli_config = CliConfig::default();
        let ctx = build_template_context(&config, &cli_config).unwrap();
        assert_eq!(ctx.language, "dart", "Flutter client should have language=dart");
        assert_eq!(ctx.framework, "flutter", "Flutter client should have framework=flutter");
        assert_eq!(ctx.kind, "client");
    }

    #[test]
    fn test_build_template_context_server_has_empty_framework() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        let cli_config = CliConfig::default();
        let ctx = build_template_context(&config, &cli_config).unwrap();
        assert_eq!(ctx.framework, "", "Server should have empty framework");
        assert_eq!(ctx.language, "go");
    }

    // --- BFF ディレクトリ生成 ---

    #[test]
    fn test_service_tier_graphql_creates_bff_directory() {
        // service Tier + GraphQL + Go サーバーで、bff/ ディレクトリが追加生成される
        let tmp = TempDir::new().unwrap();
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::GraphQL],
                db: None,
                kafka: false,
                redis: false,
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        // BFF ディレクトリが存在するか確認
        let bff_path = tmp.path().join("regions/service/order/server/go/bff");
        assert!(bff_path.exists(), "service Tier + GraphQL should create bff/ directory");
    }
}
