use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::CliConfig;
use crate::template::context::TemplateContextBuilder;
use crate::template::TemplateEngine;
use super::types::*;
use super::retry::{RetryConfig, run_with_retry};
use super::scaffold::{generate_server, generate_client, generate_library, generate_database};

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

    // service Tier + GraphQL の場合は BFF を追加生成
    if config.kind == Kind::Server
        && config.tier == Tier::Service
        && config.detail.api_styles.contains(&ApiStyle::GraphQL)
    {
        if let Some(bff_lang) = config.detail.bff_language {
            // BFF 言語が指定されている場合はテンプレートエンジンで生成を試みる
            let bff_path = output_path.join("bff");
            fs::create_dir_all(&bff_path)?;
            let tpl_dir = resolve_template_dir(base_dir);
            let bff_tpl_dir = tpl_dir.join("bff").join(bff_lang.dir_name());
            if bff_tpl_dir.exists() {
                let bff_ctx = TemplateContextBuilder::new(
                    config.detail.name.as_deref().unwrap_or("service"),
                    config.tier.as_str(),
                    bff_lang.dir_name(),
                    "bff",
                )
                .api_style("graphql")
                .build();
                let mut engine = TemplateEngine::new(&tpl_dir)?;
                let _ = engine.render_to_dir(&bff_ctx, &bff_path);
            }
        } else {
            // BFF 言語未指定の場合は空ディレクトリのみ作成（後方互換）
            let bff_path = output_path.join("bff");
            fs::create_dir_all(&bff_path)?;
        }
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

    // service Tier + GraphQL の場合は BFF を追加生成
    if config.kind == Kind::Server
        && config.tier == Tier::Service
        && config.detail.api_styles.contains(&ApiStyle::GraphQL)
    {
        if let Some(bff_lang) = config.detail.bff_language {
            let bff_path = output_path.join("bff");
            fs::create_dir_all(&bff_path)?;
            let bff_tpl_dir = tpl_dir.join("bff").join(bff_lang.dir_name());
            if bff_tpl_dir.exists() {
                let bff_ctx = TemplateContextBuilder::new(
                    config.detail.name.as_deref().unwrap_or("service"),
                    config.tier.as_str(),
                    bff_lang.dir_name(),
                    "bff",
                )
                .api_style("graphql")
                .build();
                match TemplateEngine::new(&tpl_dir) {
                    Ok(mut engine) => {
                        match engine.render_to_dir(&bff_ctx, &bff_path) {
                            Ok(files) => {
                                println!("BFF テンプレートを生成しました: {} ファイル", files.len());
                            }
                            Err(e) => {
                                eprintln!("BFF テンプレートの生成に失敗しました: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("BFF テンプレートエンジンの初期化に失敗しました: {}", e);
                    }
                }
            }
        } else {
            let bff_path = output_path.join("bff");
            fs::create_dir_all(&bff_path)?;
        }
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

/// D-08: 後処理コマンドを実行する（best-effort、リトライ付き）。
///
/// リトライ対象コマンド（go mod tidy, cargo check, npm install, flutter pub get, buf generate）
/// は指数バックオフで最大3回リトライする。
/// 非対象コマンド（oapi-codegen, cargo xtask codegen, gqlgen generate, sqlx database create）
/// は1回だけ実行する。
fn run_post_processing(config: &GenerateConfig, output_path: &Path) {
    let commands = determine_post_commands(config);
    let retry_config = RetryConfig::default();

    for (cmd, args) in &commands {
        let args_refs: Vec<&str> = args.iter().map(|s| *s).collect();
        match run_with_retry(cmd, &args_refs, output_path, &retry_config) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e);
                eprintln!(
                    "手動で実行してください: cd {} && {} {}",
                    output_path.display(),
                    cmd,
                    args.join(" ")
                );
            }
        }
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

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
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
                bff_language: None,
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        // BFF ディレクトリが存在するか確認
        let bff_path = tmp.path().join("regions/service/order/server/go/bff");
        assert!(bff_path.exists(), "service Tier + GraphQL should create bff/ directory");
    }

    // --- BFF 生成: Tier別テスト ---

    #[test]
    fn test_bff_not_created_for_system_tier_graphql() {
        // system Tier の GraphQL では BFF ディレクトリは作成されない
        let tmp = TempDir::new().unwrap();
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("gateway".to_string()),
                api_styles: vec![ApiStyle::GraphQL],
                db: None,
                kafka: false,
                redis: false,
                bff_language: Some(Language::Go),
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        let bff_path = tmp.path().join("regions/system/server/go/gateway/bff");
        assert!(!bff_path.exists(), "system Tier では BFF ディレクトリは作成されない");
    }

    #[test]
    fn test_bff_not_created_for_business_tier_graphql() {
        // business Tier の GraphQL では BFF ディレクトリは作成されない
        let tmp = TempDir::new().unwrap();
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("ledger".to_string()),
                api_styles: vec![ApiStyle::GraphQL],
                db: None,
                kafka: false,
                redis: false,
                bff_language: Some(Language::Go),
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        let bff_path = tmp.path().join("regions/business/accounting/server/go/ledger/bff");
        assert!(!bff_path.exists(), "business Tier では BFF ディレクトリは作成されない");
    }

    #[test]
    fn test_bff_directory_created_with_language() {
        // service Tier + GraphQL + bff_language=Go の場合に bff/ が作成される
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
                bff_language: Some(Language::Go),
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        let bff_path = tmp.path().join("regions/service/order/server/go/bff");
        assert!(bff_path.exists(), "service Tier + GraphQL + bff_language=Go で bff/ が作成される");
    }

    #[test]
    fn test_bff_not_created_when_no_graphql() {
        // GraphQL なしの場合は BFF は作成されない
        let tmp = TempDir::new().unwrap();
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
                bff_language: None,
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        let bff_path = tmp.path().join("regions/service/order/server/go/bff");
        assert!(!bff_path.exists(), "GraphQL なしでは BFF ディレクトリは作成されない");
    }

    #[test]
    fn test_bff_not_created_when_bff_language_none() {
        // bff_language=None でも service+GraphQL では空ディレクトリが作成される（既存の互換性）
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
                bff_language: None,
            },
        };
        execute_generate_at(&config, tmp.path()).unwrap();
        let bff_path = tmp.path().join("regions/service/order/server/go/bff");
        assert!(bff_path.exists(), "bff_language=None でも service+GraphQL では空 BFF ディレクトリが作成される（互換性維持）");
    }
}
