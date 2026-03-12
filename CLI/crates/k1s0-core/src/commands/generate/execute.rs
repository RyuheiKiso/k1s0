use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::retry::{run_with_retry, RetryConfig};
use super::scaffold::{generate_client, generate_database, generate_library, generate_server};
use super::types::{ApiStyle, Framework, GenerateConfig, Kind, LangFw, Language, Rdbms, Tier};
use crate::commands::template_migrate::parser::{
    collect_project_files, compute_checksum, snapshot_dir, write_manifest, write_snapshot,
    CURRENT_TEMPLATE_VERSION,
};
use crate::commands::template_migrate::types::TemplateManifest;
use crate::config::CliConfig;
use crate::template::context::TemplateContextBuilder;
use crate::template::TemplateEngine;

// ============================================================================
// 生成実行
// ============================================================================

/// ひな形生成を実行する。
///
/// # Errors
///
/// カレントディレクトリの取得に失敗した場合、またはひな形生成処理でエラーが発生した場合にエラーを返す。
pub fn execute_generate(config: &GenerateConfig) -> Result<()> {
    execute_generate_with_config(config, &std::env::current_dir()?, &CliConfig::default())
}

/// 指定されたベースディレクトリを起点にひな形生成を実行する。
/// GUI などのワークスペース明示呼び出し向け。
///
/// # Errors
///
/// ディレクトリの作成またはファイルの書き込みに失敗した場合にエラーを返す。
pub fn execute_generate_at(config: &GenerateConfig, base_dir: &Path) -> Result<()> {
    execute_generate_with_config(config, base_dir, &CliConfig::default())
}

/// `CliConfig` を指定してひな形生成を実行する。
/// テンプレートエンジン + 後処理コマンド付き。
///
/// # Errors
///
/// ディレクトリの作成、ファイルの書き込み、またはテンプレートのレンダリングに失敗した場合にエラーを返す。
pub fn execute_generate_with_config(
    config: &GenerateConfig,
    base_dir: &Path,
    cli_config: &CliConfig,
) -> Result<()> {
    ensure_generate_targets_available(config, base_dir)?;

    let tpl_dir = resolve_template_dir(base_dir);
    let output_path = render_scaffold_preview(config, base_dir, cli_config, &tpl_dir)?;
    let template_context = build_template_context(config, cli_config);
    let generated_files = collect_project_files(&output_path)?;
    let checksum = compute_checksum(&output_path, &generated_files)?;
    let manifest = TemplateManifest::from_generate_config(
        config,
        cli_config,
        CURRENT_TEMPLATE_VERSION,
        &checksum,
    );
    let snapshot_path = snapshot_dir(&output_path, &checksum);
    write_snapshot(&output_path, &generated_files, &snapshot_path)?;
    write_manifest(&output_path, &manifest)?;

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
                        eprintln!("Helm Chart の生成に失敗しました: {e}");
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
                        eprintln!("CI/CD ワークフローの生成に失敗しました: {e}");
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
pub(crate) fn resolve_template_dir(base_dir: &Path) -> PathBuf {
    for ancestor in base_dir.ancestors() {
        for candidate in template_dir_candidates(ancestor) {
            if is_template_dir(&candidate) {
                return candidate;
            }
        }
    }

    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    for candidate in template_dir_candidates(manifest_path) {
        if is_template_dir(&candidate) {
            return candidate;
        }
    }

    let sibling_cli_templates = manifest_path.join("..").join("k1s0-cli").join("templates");
    if is_template_dir(&sibling_cli_templates) {
        return sibling_cli_templates;
    }

    base_dir
        .join("CLI")
        .join("crates")
        .join("k1s0-cli")
        .join("templates")
}

/// テンプレートエンジンを使って生成を試みる。成功した場合 true を返す。
fn try_generate_from_templates(
    config: &GenerateConfig,
    output_path: &Path,
    template_dir: &Path,
    cli_config: &CliConfig,
) -> bool {
    let Some(ctx) = build_template_context(config, cli_config) else {
        return false;
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

    let Ok(mut engine) = TemplateEngine::new(template_dir) else {
        return false;
    };

    match engine.render_to_dir(&ctx, output_path) {
        Ok(files) => !files.is_empty(),
        Err(_) => false,
    }
}

/// プロジェクト本体の scaffold を生成する。
///
/// Helm/CI/CD や後処理は行わず、対象モジュール配下のみを作成する。
///
/// # Errors
///
/// ディレクトリ作成またはファイル生成に失敗した場合にエラーを返す。
pub(crate) fn render_scaffold_preview(
    config: &GenerateConfig,
    base_dir: &Path,
    cli_config: &CliConfig,
    template_dir: &Path,
) -> Result<PathBuf> {
    let output_path = build_output_path(config, base_dir);
    fs::create_dir_all(&output_path)?;

    let template_generated = if template_dir.exists() {
        try_generate_from_templates(config, &output_path, template_dir, cli_config)
    } else {
        false
    };

    if !template_generated {
        generate_inline_scaffold(config, &output_path)?;
    }

    render_bff_if_needed(config, template_dir, &output_path)?;
    normalize_generated_scaffold(config, &output_path)?;

    Ok(output_path)
}

fn template_dir_candidates(root: &Path) -> [PathBuf; 4] {
    [
        root.join("CLI")
            .join("crates")
            .join("k1s0-cli")
            .join("templates"),
        root.join("CLI").join("templates"),
        root.join("crates").join("k1s0-cli").join("templates"),
        root.join("templates"),
    ]
}

fn is_template_dir(candidate: &Path) -> bool {
    candidate.join("server").is_dir()
        && candidate.join("client").is_dir()
        && candidate.join("library").is_dir()
        && candidate.join("database").is_dir()
        && candidate.join("bff").is_dir()
}

fn generate_inline_scaffold(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    match config.kind {
        Kind::Server => generate_server(config, output_path)?,
        Kind::Client => generate_client(config, output_path)?,
        Kind::Library => generate_library(config, output_path)?,
        Kind::Database => generate_database(config, output_path)?,
    }

    Ok(())
}

fn render_bff_if_needed(
    config: &GenerateConfig,
    template_dir: &Path,
    output_path: &Path,
) -> Result<()> {
    if config.kind != Kind::Server
        || config.tier != Tier::Service
        || !config.detail.api_styles.contains(&ApiStyle::GraphQL)
    {
        return Ok(());
    }

    let Some(bff_lang) = config.detail.bff_language else {
        return Ok(());
    };

    let bff_tpl_dir = template_dir.join("bff").join(bff_lang.dir_name());
    if !bff_tpl_dir.exists() {
        return Ok(());
    }

    let bff_path = output_path.join("bff");
    fs::create_dir_all(&bff_path)?;
    let bff_ctx = TemplateContextBuilder::new(
        config.detail.name.as_deref().unwrap_or("service"),
        config.tier.as_str(),
        bff_lang.dir_name(),
        "bff",
    )
    .api_style("graphql")
    .build();

    match TemplateEngine::new(template_dir) {
        Ok(mut engine) => {
            let _ = engine.render_to_dir(&bff_ctx, &bff_path);
        }
        Err(error) => {
            eprintln!("BFF テンプレートエンジンの初期化に失敗しました: {error}");
        }
    }

    Ok(())
}

fn normalize_generated_scaffold(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    match config.kind {
        Kind::Database => normalize_database_layout(config, output_path),
        Kind::Library => normalize_library_layout(config, output_path),
        _ => Ok(()),
    }
}

fn normalize_database_layout(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let LangFw::Database { name, rdbms } = &config.lang_fw else {
        return Ok(());
    };

    let migrations_dir = output_path.join("migrations");
    fs::create_dir_all(&migrations_dir)?;
    fs::create_dir_all(output_path.join("seeds"))?;
    fs::create_dir_all(output_path.join("schema"))?;

    for file_name in ["001_init.up.sql", "001_init.down.sql"] {
        let root_path = output_path.join(file_name);
        let migration_path = migrations_dir.join(file_name);
        if root_path.is_file() && !migration_path.exists() {
            fs::rename(root_path, migration_path)?;
        }
    }

    let database_yaml = output_path.join("database.yaml");
    if !database_yaml.exists() {
        fs::write(
            database_yaml,
            format!("name: {name}\nrdbms: {}\n", rdbms.as_str()),
        )?;
    }

    Ok(())
}

fn normalize_library_layout(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    if !matches!(config.lang_fw, LangFw::Language(Language::Dart)) {
        return Ok(());
    }

    let Some(name) = config.detail.name.as_deref() else {
        return Ok(());
    };

    let module_name = to_snake_case(name);
    let lib_dir = output_path.join("lib");
    fs::create_dir_all(&lib_dir)?;

    let legacy_entry = lib_dir.join(format!("{name}.dart"));
    let expected_entry = lib_dir.join(format!("{module_name}.dart"));

    if legacy_entry.is_file() && !expected_entry.exists() {
        fs::rename(&legacy_entry, &expected_entry)?;
    }

    if !expected_entry.exists() {
        fs::write(
            &expected_entry,
            format!("library {module_name};\n\nexport 'src/{module_name}.dart';\n"),
        )?;
    }

    Ok(())
}

fn to_snake_case(value: &str) -> String {
    let mut snake = String::with_capacity(value.len());
    let mut previous_was_separator = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            snake.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            snake.push('_');
            previous_was_separator = true;
        }
    }

    snake.trim_matches('_').to_string()
}

/// `GenerateConfig` から `TemplateContext` を構築する。
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
        let args_refs: Vec<&str> = args.clone();
        match run_with_retry(cmd, &args_refs, output_path, &retry_config) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{e}");
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
                        commands.push((
                            "oapi-codegen",
                            vec![
                                "-generate",
                                "types,server",
                                "-package",
                                "handler",
                                "-o",
                                "internal/handler/openapi.gen.go",
                                "api/openapi/openapi.yaml",
                            ],
                        ));
                    }
                    LangFw::Language(Language::Rust) => {
                        commands.push(("cargo", vec!["xtask", "codegen"]));
                    }
                    _ => {}
                }
            }
            // GraphQL の場合は gqlgen generate
            if config.detail.api_styles.contains(&ApiStyle::GraphQL) {
                if let LangFw::Language(Language::Go) = &config.lang_fw {
                    commands.push(("go", vec!["run", "github.com/99designs/gqlgen", "generate"]));
                }
            }
            // 3. DB ありの場合は SQL マイグレーション初期化
            if config.detail.db.is_some() {
                commands.push(("sqlx", vec!["database", "create"]));
            }
        }
        Kind::Client => match &config.lang_fw {
            LangFw::Framework(Framework::React) => {
                commands.push(("npm", vec!["install"]));
            }
            LangFw::Framework(Framework::Flutter) => {
                commands.push(("flutter", vec!["pub", "get"]));
            }
            _ => {}
        },
        Kind::Library => match &config.lang_fw {
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
        },
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

fn generated_module_identifier(config: &GenerateConfig) -> String {
    let relative = build_output_path(config, Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    let relative = relative.strip_prefix("regions/").unwrap_or(&relative);
    relative.replace('/', "-")
}

fn build_ci_workflow_path(config: &GenerateConfig, base_dir: &Path) -> PathBuf {
    let file_name = format!("{}-ci.yaml", generated_module_identifier(config));
    build_cicd_output_path(config, base_dir).join(file_name)
}

fn build_deploy_workflow_path(config: &GenerateConfig, base_dir: &Path) -> Option<PathBuf> {
    (config.kind == Kind::Server).then(|| {
        let file_name = format!("{}-deploy.yaml", generated_module_identifier(config));
        build_cicd_output_path(config, base_dir).join(file_name)
    })
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

pub fn find_generate_conflicts_at(config: &GenerateConfig, base_dir: &Path) -> Vec<String> {
    let mut conflicts = Vec::new();

    let mut reserved_paths = vec![
        build_output_path(config, base_dir),
        build_ci_workflow_path(config, base_dir),
    ];

    if config.kind == Kind::Server {
        reserved_paths.push(build_helm_output_path(config, base_dir));
    }

    if let Some(deploy_workflow_path) = build_deploy_workflow_path(config, base_dir) {
        reserved_paths.push(deploy_workflow_path);
    }

    for path in reserved_paths {
        if path.exists() {
            conflicts.push(path.to_string_lossy().replace('\\', "/"));
        }
    }

    conflicts.sort();
    conflicts.dedup();
    conflicts
}

pub fn ensure_generate_targets_available(config: &GenerateConfig, base_dir: &Path) -> Result<()> {
    let conflicts = find_generate_conflicts_at(config, base_dir);
    if conflicts.is_empty() {
        return Ok(());
    }

    bail!("generated assets already exist: {}", conflicts.join(", "));
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
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|e| e.to_str()) != Some("tera") {
            continue;
        }

        let relative = path.strip_prefix(helm_tpl_dir)?;
        let template_content = fs::read_to_string(path)?;
        let template_name = relative.to_string_lossy().replace('\\', "/");

        engine
            .tera
            .add_raw_template(&template_name, &template_content)?;
        let rendered = engine.tera.render(&template_name, &tera_ctx)?;

        // .tera 拡張子を除去
        let output_relative = relative.to_string_lossy().replace('\\', "/");
        let output_relative = if Path::new(&output_relative)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("tera"))
        {
            output_relative[..output_relative.len() - 5].to_string()
        } else {
            output_relative.clone()
        };
        let output_path = output_dir.join(&output_relative);

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

        let output_path =
            build_ci_workflow_path(config, output_dir.parent().unwrap().parent().unwrap());
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

            let output_path =
                build_deploy_workflow_path(config, output_dir.parent().unwrap().parent().unwrap())
                    .expect("server deploy workflow path should exist");
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
// 配置先スキャン
// ============================================================================

/// Scan existing placement directories for a given tier.
/// For Business tier, scans `regions/business/` subdirectories.
/// For Service tier, scans `regions/service/` subdirectories.
/// For System tier, returns empty (placement not applicable).
pub fn scan_placements_at(tier: &Tier, base_dir: &Path) -> Vec<String> {
    let sub = match tier {
        Tier::Business => "regions/business",
        Tier::Service => "regions/service",
        Tier::System => return Vec::new(),
    };
    let path = base_dir.join(sub);
    if !path.is_dir() {
        return Vec::new();
    }
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&path) {
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

pub fn scan_placements(tier: &Tier) -> Vec<String> {
    scan_placements_at(tier, Path::new("."))
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::types::{DbInfo, DetailConfig};
    use super::*;
    use tempfile::TempDir;

    // --- build_output_path ---

    #[test]
    fn test_build_output_path_server_system() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(path, PathBuf::from("regions/system/server/rust/auth"));
    }

    #[test]
    fn test_build_output_path_server_business() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Business,
            placement: Some("accounting".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("ledger".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(
            path,
            PathBuf::from("regions/business/accounting/server/rust/ledger")
        );
    }

    #[test]
    fn test_build_output_path_server_service() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("order".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        // service Tier では detail.name をサブディレクトリに追加しない
        assert_eq!(path, PathBuf::from("regions/service/order/server/rust"));
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
        assert_eq!(path, PathBuf::from("regions/service/order/client/react"));
    }

    #[test]
    fn test_build_output_path_library_system() {
        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("authlib".to_string()),
                ..DetailConfig::default()
            },
        };
        let path = build_output_path(&config, Path::new(""));
        assert_eq!(path, PathBuf::from("regions/system/library/rust/authlib"));
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
        assert_eq!(path, PathBuf::from("regions/system/database/auth-db"));
    }

    // --- execute_generate ---

    #[test]
    fn test_execute_generate_rust_server_system() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/server/rust/auth");

        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
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
        assert!(base.join("src/main.rs").is_file());
        assert!(base.join("Cargo.toml").is_file());
        assert!(base.join("Dockerfile").is_file());
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
    fn test_execute_generate_rust_library_system() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("regions/system/library/rust/authlib");

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("authlib".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());

        assert!(result.is_ok());
        assert!(base.join("Cargo.toml").is_file());
        assert!(base.join("src/lib.rs").is_file());
    }

    #[test]
    fn test_execute_generate_rust_library() {
        let tmp = TempDir::new().unwrap();
        let base = tmp
            .path()
            .join("regions/business/accounting/library/rust/ledger-lib");

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
        assert!(
            base.join("schema").is_dir(),
            "schema/ directory should exist"
        );
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
            lang_fw: LangFw::Language(Language::Rust),
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
        assert!(
            cmds.iter().any(|(c, _)| *c == "cargo"),
            "should include 'cargo check'"
        );
        assert!(
            !cmds.iter().any(|(c, _)| *c == "buf"),
            "should not include 'buf generate' without gRPC"
        );
    }

    #[test]
    fn test_determine_post_commands_rust_server_with_grpc() {
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
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
        assert!(
            cmds.iter().any(|(c, _)| *c == "cargo"),
            "should include 'cargo check'"
        );
        assert!(
            cmds.iter().any(|(c, _)| *c == "buf"),
            "should include 'buf generate' with gRPC"
        );
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
        assert!(
            cmds.iter().any(|(c, _)| *c == "npm"),
            "should include 'npm install'"
        );
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
        assert!(
            cmds.iter().any(|(c, _)| *c == "flutter"),
            "should include 'flutter pub get'"
        );
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
        assert!(
            cmds.is_empty(),
            "database should have no post-processing commands"
        );
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
            lang_fw: LangFw::Language(Language::Rust),
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
        assert_eq!(ctx.rust_crate, "order");
        assert_eq!(ctx.module_path, "regions/service/order/server/rust");
    }

    // --- 後処理コマンド: REST (OpenAPI) コード生成 ---

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
            "Rust + REST should include 'cargo check' and 'cargo xtask codegen', got {cargo_cmds:?}"
        );
        assert!(
            cmds.iter()
                .any(|(c, args)| *c == "cargo" && args.contains(&"xtask")),
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
            lang_fw: LangFw::Language(Language::Rust),
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
            lang_fw: LangFw::Language(Language::Rust),
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
        assert_eq!(
            ctx.language, "typescript",
            "React client should have language=typescript"
        );
        assert_eq!(
            ctx.framework, "react",
            "React client should have framework=react"
        );
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
        assert_eq!(
            ctx.language, "dart",
            "Flutter client should have language=dart"
        );
        assert_eq!(
            ctx.framework, "flutter",
            "Flutter client should have framework=flutter"
        );
        assert_eq!(ctx.kind, "client");
    }

    #[test]
    fn test_build_template_context_server_has_empty_framework() {
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
        let cli_config = CliConfig::default();
        let ctx = build_template_context(&config, &cli_config).unwrap();
        assert_eq!(ctx.framework, "", "Server should have empty framework");
        assert_eq!(ctx.language, "rust");
    }

    // --- BFF ディレクトリ生成 ---

    #[test]
    fn test_service_tier_graphql_creates_bff_directory() {
        // service Tier + GraphQL + Go 言語サーバー (BFF 用) で、bff/ ディレクトリが追加生成される
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
        // BFF ディレクトリが存在するか確認
        let bff_path = tmp.path().join("regions/service/order/server/go/bff");
        assert!(
            bff_path.exists(),
            "service Tier + GraphQL should create bff/ directory"
        );
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
        assert!(
            !bff_path.exists(),
            "system Tier では BFF ディレクトリは作成されない"
        );
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
        let bff_path = tmp
            .path()
            .join("regions/business/accounting/server/go/ledger/bff");
        assert!(
            !bff_path.exists(),
            "business Tier では BFF ディレクトリは作成されない"
        );
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
        assert!(
            bff_path.exists(),
            "service Tier + GraphQL + bff_language=Go で bff/ が作成される"
        );
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
        assert!(
            !bff_path.exists(),
            "GraphQL なしでは BFF ディレクトリは作成されない"
        );
    }

    #[test]
    fn test_bff_not_created_when_bff_language_none() {
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
        assert!(
            !bff_path.exists(),
            "bff_language=None では BFF ディレクトリは生成されない"
        );
    }

    #[test]
    fn test_build_ci_workflow_path_uses_unique_module_identifier() {
        let base_dir = Path::new("workspace");
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

        let ci_path = build_ci_workflow_path(&config, base_dir);
        assert_eq!(
            ci_path,
            PathBuf::from("workspace/.github/workflows/service-order-client-react-ci.yaml")
        );
    }

    #[test]
    fn test_find_generate_conflicts_at_detects_existing_output_directory() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("regions/system/library/rust/utils");
        fs::create_dir_all(&output_path).unwrap();

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("utils".to_string()),
                ..DetailConfig::default()
            },
        };

        let conflicts = find_generate_conflicts_at(&config, tmp.path());
        assert_eq!(
            conflicts,
            vec![output_path.to_string_lossy().replace('\\', "/")]
        );
    }

    #[test]
    fn test_find_generate_conflicts_at_detects_server_helm_collision() {
        let tmp = TempDir::new().unwrap();
        let helm_path = tmp.path().join("infra/helm/services/system/auth");
        fs::create_dir_all(&helm_path).unwrap();

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

        let conflicts = find_generate_conflicts_at(&config, tmp.path());
        assert_eq!(
            conflicts,
            vec![helm_path.to_string_lossy().replace('\\', "/")]
        );
    }

    #[test]
    fn test_execute_generate_at_rejects_existing_conflicting_assets() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("regions/system/library/rust/utils");
        fs::create_dir_all(&output_path).unwrap();

        let config = GenerateConfig {
            kind: Kind::Library,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("utils".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&config, tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("generated assets already exist"));
    }

    #[test]
    fn test_execute_generate_at_allows_service_server_and_client_to_coexist() {
        let tmp = TempDir::new().unwrap();

        let server_config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Rust),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
                bff_language: None,
            },
        };
        execute_generate_at(&server_config, tmp.path()).unwrap();

        let client_config = GenerateConfig {
            kind: Kind::Client,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Framework(Framework::React),
            detail: DetailConfig {
                name: Some("order".to_string()),
                ..DetailConfig::default()
            },
        };

        let result = execute_generate_at(&client_config, tmp.path());
        assert!(result.is_ok());
        assert!(tmp
            .path()
            .join("regions/service/order/server/rust")
            .is_dir());
        assert!(tmp
            .path()
            .join("regions/service/order/client/react")
            .is_dir());
        assert!(tmp
            .path()
            .join(".github/workflows/service-order-server-rust-ci.yaml")
            .is_file());
        assert!(tmp
            .path()
            .join(".github/workflows/service-order-client-react-ci.yaml")
            .is_file());
    }

    // --- scan_placements_at ---

    #[test]
    fn test_scan_placements_at_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let result = scan_placements_at(&Tier::Business, tmp.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_placements_at_system_returns_empty() {
        let tmp = TempDir::new().unwrap();
        // Create some dirs that would match if System were scanned
        fs::create_dir_all(tmp.path().join("regions/system/some-dir")).unwrap();
        let result = scan_placements_at(&Tier::System, tmp.path());
        assert!(result.is_empty(), "System tier should always return empty");
    }

    #[test]
    fn test_scan_placements_at_business_with_dirs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("regions/business/accounting")).unwrap();
        fs::create_dir_all(tmp.path().join("regions/business/fa")).unwrap();
        fs::create_dir_all(tmp.path().join("regions/business/hr")).unwrap();
        // Also create a file to ensure it's not included
        fs::write(tmp.path().join("regions/business/.gitkeep"), "").unwrap();
        let result = scan_placements_at(&Tier::Business, tmp.path());
        assert_eq!(result, vec!["accounting", "fa", "hr"]);
    }

    #[test]
    fn test_scan_placements_at_service_with_dirs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("regions/service/order")).unwrap();
        fs::create_dir_all(tmp.path().join("regions/service/payment")).unwrap();
        let result = scan_placements_at(&Tier::Service, tmp.path());
        assert_eq!(result, vec!["order", "payment"]);
    }
}
