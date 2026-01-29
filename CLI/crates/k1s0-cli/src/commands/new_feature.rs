//! `k1s0 new-feature` コマンド
//!
//! 新規サービスの雛形を生成する。

use std::path::PathBuf;

use chrono::Utc;
use clap::{Args, ValueEnum};

use k1s0_generator::fingerprint::calculate_fingerprint;
use k1s0_generator::manifest::{
    Dependencies, LayerType, Manifest, ServiceInfo, TemplateInfo, UpdatePolicy, SCHEMA_VERSION,
};
use k1s0_generator::template::TemplateRenderer;
use k1s0_generator::Context;

use crate::error::{CliError, Result};
use crate::output::output;
use crate::prompts;
use crate::version;

/// サービスタイプ
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ServiceType {
    /// Rust バックエンド
    #[value(name = "backend-rust")]
    BackendRust,
    /// Go バックエンド
    #[value(name = "backend-go")]
    BackendGo,
    /// React フロントエンド
    #[value(name = "frontend-react")]
    FrontendReact,
    /// C# バックエンド
    #[value(name = "backend-csharp")]
    BackendCsharp,
    /// Python バックエンド
    #[value(name = "backend-python")]
    BackendPython,
    /// Flutter フロントエンド
    #[value(name = "frontend-flutter")]
    FrontendFlutter,
}

impl ServiceType {
    /// テンプレートディレクトリの相対パスを取得
    pub fn template_path(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "CLI/templates/backend-rust/feature",
            ServiceType::BackendGo => "CLI/templates/backend-go/feature",
            ServiceType::BackendCsharp => "CLI/templates/backend-csharp/feature",
            ServiceType::BackendPython => "CLI/templates/backend-python/feature",
            ServiceType::FrontendReact => "CLI/templates/frontend-react/feature",
            ServiceType::FrontendFlutter => "CLI/templates/frontend-flutter/feature",
        }
    }

    /// 出力ディレクトリのベースパスを取得
    pub fn output_base(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "feature/backend/rust",
            ServiceType::BackendGo => "feature/backend/go",
            ServiceType::BackendCsharp => "feature/backend/csharp",
            ServiceType::BackendPython => "feature/backend/python",
            ServiceType::FrontendReact => "feature/frontend/react",
            ServiceType::FrontendFlutter => "feature/frontend/flutter",
        }
    }

    /// domain ディレクトリのベースパスを取得
    pub fn domain_base(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "domain/backend/rust",
            ServiceType::BackendGo => "domain/backend/go",
            ServiceType::BackendCsharp => "domain/backend/csharp",
            ServiceType::BackendPython => "domain/backend/python",
            ServiceType::FrontendReact => "domain/frontend/react",
            ServiceType::FrontendFlutter => "domain/frontend/flutter",
        }
    }

    /// 言語名を取得
    pub fn language(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "rust",
            ServiceType::BackendGo => "go",
            ServiceType::BackendCsharp => "csharp",
            ServiceType::BackendPython => "python",
            ServiceType::FrontendReact => "typescript",
            ServiceType::FrontendFlutter => "dart",
        }
    }

    /// サービスタイプ名を取得
    pub fn service_type_name(&self) -> &'static str {
        match self {
            ServiceType::BackendRust | ServiceType::BackendGo | ServiceType::BackendCsharp | ServiceType::BackendPython => "backend",
            ServiceType::FrontendReact | ServiceType::FrontendFlutter => "frontend",
        }
    }
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::BackendRust => write!(f, "backend-rust"),
            ServiceType::BackendGo => write!(f, "backend-go"),
            ServiceType::BackendCsharp => write!(f, "backend-csharp"),
            ServiceType::BackendPython => write!(f, "backend-python"),
            ServiceType::FrontendReact => write!(f, "frontend-react"),
            ServiceType::FrontendFlutter => write!(f, "frontend-flutter"),
        }
    }
}

/// `k1s0 new-feature` の引数
#[derive(Args, Debug)]
pub struct NewFeatureArgs {
    /// サービスタイプ
    #[arg(short = 't', long = "type", value_enum)]
    pub service_type: Option<ServiceType>,

    /// サービス名（kebab-case）
    #[arg(short, long)]
    pub name: Option<String>,

    /// 所属する domain 名（省略時は domain に属さない独立した feature として作成）
    #[arg(long)]
    pub domain: Option<String>,

    /// 生成先ディレクトリ（デフォルト: feature/{type}/{name}/）
    #[arg(short, long)]
    pub output: Option<String>,

    /// 既存のディレクトリを上書きする
    #[arg(short, long)]
    pub force: bool,

    /// gRPC API を含める
    #[arg(long)]
    pub with_grpc: bool,

    /// REST API を含める
    #[arg(long)]
    pub with_rest: bool,

    /// DB マイグレーションを含める
    #[arg(long)]
    pub with_db: bool,

    /// 対話モードを強制する
    #[arg(short = 'i', long)]
    pub interactive: bool,
}

impl NewFeatureArgs {
    /// 必須引数がすべて提供されているかどうか
    fn has_required_args(&self) -> bool {
        self.service_type.is_some() && self.name.is_some()
    }
}

/// 解決済みの引数（対話入力後）
struct ResolvedArgs {
    service_type: ServiceType,
    name: String,
    domain: Option<String>,
    output: Option<String>,
    force: bool,
    with_grpc: bool,
    with_rest: bool,
    with_db: bool,
}

/// Domain 情報
struct DomainInfo {
    /// domain 名
    name: String,
    /// domain のバージョン
    version: String,
    /// domain へのパス（将来の拡張用）
    #[allow(dead_code)]
    path: PathBuf,
}

/// `k1s0 new-feature` を実行する
pub fn execute(args: NewFeatureArgs) -> Result<()> {
    // 対話モードを判定
    let use_interactive = prompts::should_use_interactive_mode(
        args.interactive,
        args.has_required_args(),
    )?;

    // 引数を解決（対話入力または引数から）
    let resolved = if use_interactive {
        resolve_args_interactive(args)?
    } else {
        resolve_args_from_cli(args)?
    };

    // 生成を実行
    execute_generation(resolved)
}

/// CLI 引数から解決済み引数を構築
fn resolve_args_from_cli(args: NewFeatureArgs) -> Result<ResolvedArgs> {
    let service_type = args.service_type.ok_or_else(|| {
        CliError::missing_required_args("--type / -t オプションが必要です")
    })?;

    let name = args.name.ok_or_else(|| {
        CliError::missing_required_args("--name / -n オプションが必要です")
    })?;

    Ok(ResolvedArgs {
        service_type,
        name,
        domain: args.domain,
        output: args.output,
        force: args.force,
        with_grpc: args.with_grpc,
        with_rest: args.with_rest,
        with_db: args.with_db,
    })
}

/// 対話モードで引数を解決
fn resolve_args_interactive(args: NewFeatureArgs) -> Result<ResolvedArgs> {
    let out = output();

    // バナー表示
    out.header("k1s0 new-feature");
    out.newline();
    out.info("対話モードで feature を作成します");
    out.newline();

    // 1. service_type が未指定 → テンプレート選択プロンプト
    let service_type = if let Some(st) = args.service_type {
        st
    } else {
        prompts::template_type::select_service_type()?
    };

    // 2. name が未指定 → 名前入力プロンプト
    let name = if let Some(n) = args.name {
        // CLI から提供された名前をバリデーション
        if !is_valid_kebab_case(&n) {
            return Err(CliError::invalid_service_name(&n));
        }
        n
    } else {
        prompts::name_input::input_feature_name()?
    };

    // 3. domain が未指定 + 既存 domain 存在 → ドメイン選択プロンプト
    let domain = if args.domain.is_some() {
        args.domain
    } else {
        let domain_base = service_type.domain_base();
        prompts::options::select_domain(domain_base)?
    };

    // 4. オプション未指定 → オプション選択プロンプト
    let (with_grpc, with_rest, with_db) = if args.with_grpc || args.with_rest || args.with_db {
        // 既にオプションが指定されている場合はそのまま使用
        (args.with_grpc, args.with_rest, args.with_db)
    } else {
        let options = prompts::options::select_feature_options()?;
        (options.with_grpc, options.with_rest, options.with_db)
    };

    out.newline();

    Ok(ResolvedArgs {
        service_type,
        name,
        domain,
        output: args.output,
        force: args.force,
        with_grpc,
        with_rest,
        with_db,
    })
}

/// 生成を実行する
fn execute_generation(args: ResolvedArgs) -> Result<()> {
    let out = output();

    // サービス名のバリデーション（kebab-case）
    if !is_valid_kebab_case(&args.name) {
        return Err(CliError::invalid_service_name(&args.name));
    }

    // domain が指定されている場合、存在チェックとバージョン取得
    let domain_info = if let Some(ref domain_name) = args.domain {
        Some(validate_and_get_domain_info(args.service_type, domain_name)?)
    } else {
        None
    };

    // 出力パスを決定
    let output_path = args.output.clone().unwrap_or_else(|| {
        format!("{}/{}", args.service_type.output_base(), args.name)
    });
    let output_dir = PathBuf::from(&output_path);

    out.header("k1s0 new-feature");
    out.newline();

    out.list_item("type", &args.service_type.to_string());
    out.list_item("name", &args.name);
    out.list_item("output", &output_path);
    out.list_item("layer", "feature");
    if let Some(ref info) = domain_info {
        out.list_item("domain", &info.name);
        out.list_item("domain_version", &format!("^{}", &info.version));
    }
    out.list_item("with_grpc", &args.with_grpc.to_string());
    out.list_item("with_rest", &args.with_rest.to_string());
    out.list_item("with_db", &args.with_db.to_string());
    out.newline();

    // 既存衝突検査
    if output_dir.exists() {
        if args.force {
            out.warning(&format!(
                "既存のディレクトリを削除します: {}",
                output_dir.display()
            ));
            std::fs::remove_dir_all(&output_dir).map_err(|e| {
                CliError::io(format!("ディレクトリの削除に失敗: {}: {}", output_dir.display(), e))
            })?;
        } else {
            return Err(CliError::conflict(format!(
                "ディレクトリが既に存在します: {}",
                output_dir.display()
            ))
            .with_target(output_dir.display().to_string())
            .with_hint("--force オプションで上書きするか、別の名前を指定してください"));
        }
    }

    // テンプレートディレクトリを特定
    let template_dir = find_template_dir(args.service_type)?;
    out.info(&format!("テンプレート: {}", template_dir.display()));

    // fingerprint を算出
    let fingerprint = calculate_fingerprint(&template_dir).map_err(|e| {
        CliError::internal(format!("fingerprint の算出に失敗: {}", e))
    })?;
    out.list_item("fingerprint", &fingerprint[..16]);

    out.newline();
    out.info("テンプレートを展開中...");

    // Tera コンテキストを作成
    let context = create_template_context(&args, domain_info.as_ref());

    // テンプレートを展開
    let renderer = TemplateRenderer::new(&template_dir).map_err(|e| {
        CliError::internal(format!("テンプレートの読み込みに失敗: {}", e))
    })?;

    let render_result = renderer.render_directory(&output_dir, &context).map_err(|e| {
        CliError::internal(format!("テンプレートの展開に失敗: {}", e))
    })?;

    // 結果を表示
    out.newline();
    for file in &render_result.created_files {
        out.file_added(file);
    }

    // manifest.json を作成
    let manifest = create_manifest(&args, &fingerprint, domain_info.as_ref())?;
    let k1s0_dir = output_dir.join(".k1s0");
    std::fs::create_dir_all(&k1s0_dir).map_err(|e| {
        CliError::io(format!(".k1s0 ディレクトリの作成に失敗: {}: {}", k1s0_dir.display(), e))
    })?;

    let manifest_path = k1s0_dir.join("manifest.json");
    manifest.save(&manifest_path).map_err(|e| {
        CliError::internal(format!("manifest.json の保存に失敗: {}", e))
    })?;
    out.file_added(".k1s0/manifest.json");

    out.newline();
    out.success(&format!(
        "サービス '{}' を生成しました",
        args.name
    ));

    out.newline();
    out.header("次のステップ:");
    out.hint(&format!("cd {}", output_path));
    out.hint("k1s0 lint でサービスの規約準拠を確認");

    Ok(())
}

/// domain の存在チェックとバージョン取得
fn validate_and_get_domain_info(
    service_type: ServiceType,
    domain_name: &str,
) -> Result<DomainInfo> {
    let domain_base = service_type.domain_base();
    let domain_path = PathBuf::from(format!("{}/{}", domain_base, domain_name));

    // domain ディレクトリの存在チェック
    if !domain_path.exists() {
        return Err(CliError::config(format!(
            "domain '{}' が見つかりません",
            domain_name
        ))
        .with_target(domain_path.display().to_string())
        .with_hint(format!(
            "まず 'k1s0 new-domain --type {} --name {}' で domain を作成してください",
            service_type, domain_name
        )));
    }

    // manifest.json の存在チェック
    let manifest_path = domain_path.join(".k1s0/manifest.json");
    if !manifest_path.exists() {
        return Err(CliError::config(format!(
            "domain '{}' の manifest.json が見つかりません",
            domain_name
        ))
        .with_target(manifest_path.display().to_string())
        .with_hint("domain が正しく作成されていることを確認してください"));
    }

    // manifest.json からバージョンを取得
    let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        CliError::io(format!("manifest.json の読み込みに失敗: {}", e))
    })?;

    let version = extract_domain_version(&manifest_content)?;

    Ok(DomainInfo {
        name: domain_name.to_string(),
        version,
        path: domain_path,
    })
}

/// manifest.json からバージョンを抽出する
fn extract_domain_version(manifest_content: &str) -> Result<String> {
    // JSON からバージョンを抽出
    let json: serde_json::Value = serde_json::from_str(manifest_content).map_err(|e| {
        CliError::config(format!("manifest.json のパースに失敗: {}", e))
    })?;

    // まず service.version を探す（既存のスキーマ）
    if let Some(version) = json.get("service").and_then(|s| s.get("version")).and_then(|v| v.as_str()) {
        return Ok(version.to_string());
    }

    // 次に k1s0_version を使用（domain の初期バージョンとして）
    if json.get("k1s0_version").and_then(|v| v.as_str()).is_some() {
        // domain の場合は 0.1.0 から開始するのがデフォルト
        return Ok("0.1.0".to_string());
    }

    // バージョンが見つからない場合はデフォルト
    Ok("0.1.0".to_string())
}

/// バージョン制約を生成する（^major.minor.0 形式）
fn generate_version_constraint(version: &str) -> String {
    // SemVer をパースして ^major.minor.0 形式に変換
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        format!("^{}.{}.0", parts[0], parts[1])
    } else {
        format!("^{}", version)
    }
}

/// テンプレートディレクトリを検索する
fn find_template_dir(service_type: ServiceType) -> Result<PathBuf> {
    let relative_path = service_type.template_path();

    // カレントディレクトリから探す
    let current_dir = std::env::current_dir().map_err(|e| {
        CliError::io(format!("カレントディレクトリの取得に失敗: {}", e))
    })?;

    let template_dir = current_dir.join(relative_path);
    if template_dir.exists() {
        return Ok(template_dir);
    }

    // 親ディレクトリを辿って探す（モノレポ内のどこからでも実行できるように）
    let mut search_dir = current_dir.clone();
    for _ in 0..5 {
        if let Some(parent) = search_dir.parent() {
            let candidate: PathBuf = parent.join(relative_path);
            if candidate.exists() {
                return Ok(candidate);
            }
            search_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(CliError::template_not_found(relative_path)
        .with_hint("k1s0 リポジトリのルートディレクトリから実行してください"))
}

/// テンプレート用のコンテキストを作成する
fn create_template_context(args: &ResolvedArgs, domain_info: Option<&DomainInfo>) -> Context {
    let mut context = Context::new();

    // 基本情報
    context.insert("feature_name", &args.name);
    context.insert("service_name", &args.name);
    context.insert("language", args.service_type.language());
    context.insert("service_type", args.service_type.service_type_name());
    context.insert("layer", "feature");
    context.insert("k1s0_version", version());

    // 命名規則の変換
    context.insert("feature_name_snake", &args.name.replace('-', "_"));
    context.insert("feature_name_pascal", &to_pascal_case(&args.name));
    context.insert("feature_name_kebab", &args.name); // kebab-case はそのまま
    context.insert("feature_name_title", &to_title_case(&args.name));

    // domain 情報
    if let Some(info) = domain_info {
        context.insert("domain", &info.name);
        context.insert("domain_name", &info.name);
        context.insert("domain_name_snake", &info.name.replace('-', "_"));
        context.insert("domain_version", &info.version);
        context.insert("domain_version_constraint", &generate_version_constraint(&info.version));
        context.insert("has_domain", &true);
    } else {
        context.insert("has_domain", &false);
    }

    // オプション
    context.insert("with_grpc", &args.with_grpc);
    context.insert("with_rest", &args.with_rest);
    context.insert("with_db", &args.with_db);

    // 日時
    context.insert("now", &Utc::now());

    context
}

/// manifest.json を作成する
fn create_manifest(
    args: &ResolvedArgs,
    fingerprint: &str,
    domain_info: Option<&DomainInfo>,
) -> Result<Manifest> {
    let managed_paths = get_managed_paths(args.service_type);
    let protected_paths = get_protected_paths(args.service_type);
    let update_policy = get_update_policy(args.service_type);

    let (domain, domain_version, dependencies) = if let Some(info) = domain_info {
        let version_constraint = generate_version_constraint(&info.version);
        let mut domain_deps = std::collections::HashMap::new();
        domain_deps.insert(info.name.clone(), version_constraint.clone());

        (
            Some(info.name.clone()),
            Some(version_constraint),
            Some(Dependencies {
                framework_crates: vec![],
                framework: vec![],
                domain: Some(domain_deps),
            }),
        )
    } else {
        (None, None, None)
    };

    Ok(Manifest {
        schema_version: SCHEMA_VERSION.to_string(),
        k1s0_version: version().to_string(),
        template: TemplateInfo {
            name: args.service_type.to_string(),
            version: version().to_string(),
            source: "local".to_string(),
            path: args.service_type.template_path().to_string(),
            revision: None,
            fingerprint: fingerprint.to_string(),
        },
        service: ServiceInfo {
            service_name: args.name.clone(),
            language: args.service_type.language().to_string(),
            service_type: args.service_type.service_type_name().to_string(),
            framework: None,
        },
        layer: LayerType::Feature,
        domain,
        version: None, // feature 層はバージョンを持たない
        domain_version,
        min_framework_version: None,
        breaking_changes: None,
        deprecated: None,
        generated_at: Utc::now().to_rfc3339(),
        managed_paths,
        protected_paths,
        update_policy,
        checksums: std::collections::HashMap::new(),
        dependencies,
    })
}

/// CLI が管理するパスを取得
fn get_managed_paths(service_type: ServiceType) -> Vec<String> {
    match service_type {
        ServiceType::BackendRust | ServiceType::BackendGo | ServiceType::BackendCsharp | ServiceType::BackendPython => vec![
            "deploy/".to_string(),
            "buf.yaml".to_string(),
            "buf.gen.yaml".to_string(),
        ],
        ServiceType::FrontendReact | ServiceType::FrontendFlutter => vec![
            "deploy/".to_string(),
        ],
    }
}

/// CLI が変更しないパスを取得
fn get_protected_paths(service_type: ServiceType) -> Vec<String> {
    match service_type {
        ServiceType::BackendRust | ServiceType::BackendGo | ServiceType::BackendCsharp | ServiceType::BackendPython => vec![
            "src/domain/".to_string(),
            "src/application/".to_string(),
            "README.md".to_string(),
        ],
        ServiceType::FrontendReact => vec![
            "src/domain/".to_string(),
            "src/application/".to_string(),
            "src/presentation/".to_string(),
            "README.md".to_string(),
        ],
        ServiceType::FrontendFlutter => vec![
            "lib/src/domain/".to_string(),
            "lib/src/application/".to_string(),
            "lib/src/presentation/".to_string(),
            "README.md".to_string(),
        ],
    }
}

/// パス別の更新ポリシーを取得
fn get_update_policy(
    service_type: ServiceType,
) -> std::collections::HashMap<String, UpdatePolicy> {
    let mut policy = std::collections::HashMap::new();

    match service_type {
        ServiceType::BackendRust | ServiceType::BackendGo | ServiceType::BackendCsharp | ServiceType::BackendPython => {
            policy.insert("deploy/".to_string(), UpdatePolicy::Auto);
            policy.insert("buf.yaml".to_string(), UpdatePolicy::Auto);
            policy.insert("src/domain/".to_string(), UpdatePolicy::Protected);
            policy.insert("src/application/".to_string(), UpdatePolicy::Protected);
            policy.insert("README.md".to_string(), UpdatePolicy::SuggestOnly);
            policy.insert("config/".to_string(), UpdatePolicy::SuggestOnly);
        }
        ServiceType::FrontendReact | ServiceType::FrontendFlutter => {
            policy.insert("deploy/".to_string(), UpdatePolicy::Auto);
            policy.insert("README.md".to_string(), UpdatePolicy::SuggestOnly);
            policy.insert("config/".to_string(), UpdatePolicy::SuggestOnly);
        }
    }

    policy
}

/// kebab-case を PascalCase に変換する
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect()
}

fn to_title_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// kebab-case かどうかを検証する
fn is_valid_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let chars: Vec<char> = s.chars().collect();

    // 先頭は小文字
    if !chars[0].is_ascii_lowercase() {
        return false;
    }

    // 末尾はハイフンでない
    if chars.last() == Some(&'-') {
        return false;
    }

    // 連続するハイフンがない
    for i in 0..chars.len() - 1 {
        if chars[i] == '-' && chars[i + 1] == '-' {
            return false;
        }
    }

    // 許可される文字のみ
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_kebab_case() {
        assert!(is_valid_kebab_case("user-management"));
        assert!(is_valid_kebab_case("order"));
        assert!(is_valid_kebab_case("auth-service"));
        assert!(is_valid_kebab_case("api2"));

        assert!(!is_valid_kebab_case("")); // 空
        assert!(!is_valid_kebab_case("UserManagement")); // CamelCase
        assert!(!is_valid_kebab_case("user_management")); // snake_case
        assert!(!is_valid_kebab_case("-user")); // 先頭ハイフン
        assert!(!is_valid_kebab_case("user-")); // 末尾ハイフン
        assert!(!is_valid_kebab_case("user--management")); // 連続ハイフン
        assert!(!is_valid_kebab_case("2user")); // 先頭数字
    }

    #[test]
    fn test_generate_version_constraint() {
        assert_eq!(generate_version_constraint("1.2.3"), "^1.2.0");
        assert_eq!(generate_version_constraint("0.1.0"), "^0.1.0");
        assert_eq!(generate_version_constraint("2.0.0"), "^2.0.0");
        assert_eq!(generate_version_constraint("1.5.10"), "^1.5.0");
    }

    #[test]
    fn test_domain_base() {
        assert_eq!(ServiceType::BackendRust.domain_base(), "domain/backend/rust");
        assert_eq!(ServiceType::BackendGo.domain_base(), "domain/backend/go");
        assert_eq!(ServiceType::FrontendReact.domain_base(), "domain/frontend/react");
        assert_eq!(ServiceType::FrontendFlutter.domain_base(), "domain/frontend/flutter");
    }

    #[test]
    fn test_has_required_args() {
        let args_complete = NewFeatureArgs {
            service_type: Some(ServiceType::BackendRust),
            name: Some("test".to_string()),
            domain: None,
            output: None,
            force: false,
            with_grpc: false,
            with_rest: false,
            with_db: false,
            interactive: false,
        };
        assert!(args_complete.has_required_args());

        let args_missing_type = NewFeatureArgs {
            service_type: None,
            name: Some("test".to_string()),
            domain: None,
            output: None,
            force: false,
            with_grpc: false,
            with_rest: false,
            with_db: false,
            interactive: false,
        };
        assert!(!args_missing_type.has_required_args());

        let args_missing_name = NewFeatureArgs {
            service_type: Some(ServiceType::BackendRust),
            name: None,
            domain: None,
            output: None,
            force: false,
            with_grpc: false,
            with_rest: false,
            with_db: false,
            interactive: false,
        };
        assert!(!args_missing_name.has_required_args());
    }
}
