//! `k1s0 new-domain` コマンド
//!
//! domain 層の雛形を生成する。

use std::path::PathBuf;

use chrono::Utc;
use clap::{Args, ValueEnum};
use regex::Regex;

use k1s0_generator::fingerprint::calculate_fingerprint;
use k1s0_generator::manifest::{
    LayerType, Manifest, ServiceInfo, TemplateInfo, UpdatePolicy, SCHEMA_VERSION,
};
use k1s0_generator::template::TemplateRenderer;
use k1s0_generator::Context;

use crate::error::{CliError, Result};
use crate::output::output;
use crate::prompts;
use crate::version;

/// ドメインタイプ（テンプレートタイプ）
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum DomainType {
    /// Rust バックエンド
    #[value(name = "backend-rust")]
    BackendRust,
    /// Go バックエンド
    #[value(name = "backend-go")]
    BackendGo,
    /// React フロントエンド
    #[value(name = "frontend-react")]
    FrontendReact,
    /// Flutter フロントエンド
    #[value(name = "frontend-flutter")]
    FrontendFlutter,
}

impl DomainType {
    /// テンプレートディレクトリの相対パスを取得
    pub fn template_path(&self) -> &'static str {
        match self {
            DomainType::BackendRust => "CLI/templates/backend-rust/domain",
            DomainType::BackendGo => "CLI/templates/backend-go/domain",
            DomainType::FrontendReact => "CLI/templates/frontend-react/domain",
            DomainType::FrontendFlutter => "CLI/templates/frontend-flutter/domain",
        }
    }

    /// 出力ディレクトリのベースパスを取得
    pub fn output_base(&self) -> &'static str {
        match self {
            DomainType::BackendRust => "domain/backend/rust",
            DomainType::BackendGo => "domain/backend/go",
            DomainType::FrontendReact => "domain/frontend/react",
            DomainType::FrontendFlutter => "domain/frontend/flutter",
        }
    }

    /// 言語名を取得
    pub fn language(&self) -> &'static str {
        match self {
            DomainType::BackendRust => "rust",
            DomainType::BackendGo => "go",
            DomainType::FrontendReact => "typescript",
            DomainType::FrontendFlutter => "dart",
        }
    }

    /// サービスタイプ名を取得
    pub fn service_type_name(&self) -> &'static str {
        match self {
            DomainType::BackendRust | DomainType::BackendGo => "backend",
            DomainType::FrontendReact | DomainType::FrontendFlutter => "frontend",
        }
    }
}

impl std::fmt::Display for DomainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainType::BackendRust => write!(f, "backend-rust"),
            DomainType::BackendGo => write!(f, "backend-go"),
            DomainType::FrontendReact => write!(f, "frontend-react"),
            DomainType::FrontendFlutter => write!(f, "frontend-flutter"),
        }
    }
}

/// 予約語（ドメイン名として使用不可）
const RESERVED_NAMES: &[&str] = &["framework", "feature", "domain", "k1s0", "common", "shared"];

/// `k1s0 new-domain` の引数
#[derive(Args, Debug)]
pub struct NewDomainArgs {
    /// テンプレートタイプ（例: backend-rust, backend-go, frontend-react, frontend-flutter）
    #[arg(short = 't', long = "type", value_enum)]
    pub domain_type: Option<DomainType>,

    /// ドメイン名（kebab-case）
    #[arg(short, long)]
    pub name: Option<String>,

    /// 出力先ディレクトリ（省略時は自動決定）
    #[arg(short, long)]
    pub output: Option<String>,

    /// 既存ディレクトリを上書きする
    #[arg(short, long)]
    pub force: bool,

    /// 対話モードを強制する
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// ドメインイベント雛形を含める
    #[arg(long)]
    pub with_events: bool,

    /// リポジトリ trait 雛形を含める（デフォルト: true）
    #[arg(long, default_value = "true")]
    pub with_repository: bool,

    /// 初期バージョン（デフォルト: 0.1.0）
    #[arg(long, default_value = "0.1.0")]
    pub version: String,
}

impl NewDomainArgs {
    /// 必須引数がすべて提供されているかどうか
    fn has_required_args(&self) -> bool {
        self.domain_type.is_some() && self.name.is_some()
    }
}

/// 解決済みの引数（対話入力後）
struct ResolvedArgs {
    domain_type: DomainType,
    name: String,
    output: Option<String>,
    force: bool,
    with_events: bool,
    with_repository: bool,
    version: String,
}

/// `k1s0 new-domain` を実行する
pub fn execute(args: NewDomainArgs) -> Result<()> {
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
fn resolve_args_from_cli(args: NewDomainArgs) -> Result<ResolvedArgs> {
    let domain_type = args.domain_type.ok_or_else(|| {
        CliError::missing_required_args("--type / -t オプションが必要です")
    })?;

    let name = args.name.ok_or_else(|| {
        CliError::missing_required_args("--name / -n オプションが必要です")
    })?;

    Ok(ResolvedArgs {
        domain_type,
        name,
        output: args.output,
        force: args.force,
        with_events: args.with_events,
        with_repository: args.with_repository,
        version: args.version,
    })
}

/// 対話モードで引数を解決
fn resolve_args_interactive(args: NewDomainArgs) -> Result<ResolvedArgs> {
    let out = output();

    // バナー表示
    out.header("k1s0 new-domain");
    out.newline();
    out.info("対話モードで domain を作成します");
    out.newline();

    // 1. domain_type が未指定 → テンプレート選択プロンプト
    let domain_type = if let Some(dt) = args.domain_type {
        dt
    } else {
        prompts::template_type::select_domain_type()?
    };

    // 2. name が未指定 → 名前入力プロンプト
    let name = if let Some(n) = args.name {
        // CLI から提供された名前をバリデーション
        validate_domain_name(&n)?;
        n
    } else {
        prompts::name_input::input_domain_name()?
    };

    out.newline();

    Ok(ResolvedArgs {
        domain_type,
        name,
        output: args.output,
        force: args.force,
        with_events: args.with_events,
        with_repository: args.with_repository,
        version: args.version,
    })
}

/// 生成を実行する
fn execute_generation(args: ResolvedArgs) -> Result<()> {
    let out = output();

    // ドメイン名のバリデーション
    validate_domain_name(&args.name)?;

    // 出力パスを決定
    let output_path = args.output.clone().unwrap_or_else(|| {
        format!("{}/{}", args.domain_type.output_base(), args.name)
    });
    let output_dir = PathBuf::from(&output_path);

    out.header("k1s0 new-domain");
    out.newline();

    out.list_item("type", &args.domain_type.to_string());
    out.list_item("name", &args.name);
    out.list_item("output", &output_path);
    out.list_item("layer", "domain");
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
    let template_dir = find_template_dir(args.domain_type)?;
    out.info(&format!("テンプレート: {}", template_dir.display()));

    // fingerprint を算出
    let fingerprint = calculate_fingerprint(&template_dir).map_err(|e| {
        CliError::internal(format!("fingerprint の算出に失敗: {}", e))
    })?;
    out.list_item("fingerprint", &fingerprint[..16]);

    out.newline();
    out.info("テンプレートを展開中...");

    // Tera コンテキストを作成
    let context = create_template_context(&args);

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
    let manifest = create_manifest(&args, &fingerprint)?;
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
        "domain '{}' を作成しました",
        args.name
    ));

    out.newline();
    out.header("次のステップ:");
    out.hint(&format!("cd {}", output_path));
    out.hint(&format!(
        "1. {} にビジネスロジックを実装してください",
        output_dir.join("src").display()
    ));
    out.hint("2. 実装が完了したら 'k1s0 lint' で規約を確認してください");

    Ok(())
}

/// ドメイン名をバリデーションする
fn validate_domain_name(name: &str) -> Result<()> {
    // 空文字チェック
    if name.is_empty() {
        return Err(CliError::usage("ドメイン名が空です")
            .with_hint("ドメイン名は kebab-case で指定してください（例: production, user-management）"));
    }

    // kebab-case チェック
    let kebab_regex = Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$")
        .expect("Invalid regex pattern");
    if !kebab_regex.is_match(name) {
        return Err(CliError::usage(format!(
            "ドメイン名 '{}' は kebab-case ではありません",
            name
        ))
        .with_hint("ドメイン名は kebab-case で指定してください（例: production, user-management）"));
    }

    // 予約語チェック
    if RESERVED_NAMES.contains(&name) {
        return Err(CliError::usage(format!(
            "'{}' は予約語のため使用できません",
            name
        ))
        .with_hint(format!(
            "予約語: {}",
            RESERVED_NAMES.join(", ")
        )));
    }

    Ok(())
}

/// テンプレートディレクトリを検索する
fn find_template_dir(domain_type: DomainType) -> Result<PathBuf> {
    let relative_path = domain_type.template_path();

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
fn create_template_context(args: &ResolvedArgs) -> Context {
    let mut context = Context::new();

    // 基本情報
    context.insert("name", &args.name);
    context.insert("domain_name", &args.name);
    context.insert("template_type", &args.domain_type.to_string());
    context.insert("language", args.domain_type.language());
    context.insert("service_type", args.domain_type.service_type_name());
    context.insert("layer", "domain");
    context.insert("k1s0_version", version());
    context.insert("template_version", version());

    // 命名規則の変換
    context.insert("name_snake", &args.name.replace('-', "_"));
    context.insert("name_pascal", &to_pascal_case(&args.name));
    context.insert("name_kebab", &args.name);
    context.insert("name_title", &to_title_case(&args.name));

    // 日時
    let now = Utc::now();
    context.insert("created_at", &now.to_rfc3339());
    context.insert("now", &now);

    // オプション
    context.insert("with_events", &args.with_events);
    context.insert("with_repository", &args.with_repository);
    context.insert("domain_version", &args.version);

    // fingerprint（テンプレート展開時に上書きされる可能性あり）
    context.insert("fingerprint", "");

    context
}

/// manifest.json を作成する
fn create_manifest(
    args: &ResolvedArgs,
    fingerprint: &str,
) -> Result<Manifest> {
    let managed_paths = get_managed_paths(args.domain_type);
    let protected_paths = get_protected_paths(args.domain_type);
    let update_policy = get_update_policy(args.domain_type);

    Ok(Manifest {
        schema_version: SCHEMA_VERSION.to_string(),
        k1s0_version: version().to_string(),
        template: TemplateInfo {
            name: args.domain_type.to_string(),
            version: version().to_string(),
            source: "local".to_string(),
            path: args.domain_type.template_path().to_string(),
            revision: None,
            fingerprint: fingerprint.to_string(),
        },
        service: ServiceInfo {
            service_name: args.name.clone(),
            language: args.domain_type.language().to_string(),
            service_type: args.domain_type.service_type_name().to_string(),
            framework: None,
        },
        layer: LayerType::Domain,
        domain: None, // domain 層自身は他の domain に所属しない
        version: Some(args.version.clone()),
        domain_version: None,
        min_framework_version: Some(version().to_string()),
        breaking_changes: None,
        deprecated: None,
        generated_at: Utc::now().to_rfc3339(),
        managed_paths,
        protected_paths,
        update_policy,
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    })
}

/// CLI が管理するパスを取得
fn get_managed_paths(domain_type: DomainType) -> Vec<String> {
    match domain_type {
        DomainType::BackendRust => vec![
            "Cargo.toml".to_string(),
        ],
        DomainType::BackendGo => vec![
            "go.mod".to_string(),
        ],
        DomainType::FrontendReact => vec![
            "package.json".to_string(),
            "tsconfig.json".to_string(),
        ],
        DomainType::FrontendFlutter => vec![
            "pubspec.yaml".to_string(),
        ],
    }
}

/// CLI が変更しないパスを取得
fn get_protected_paths(domain_type: DomainType) -> Vec<String> {
    match domain_type {
        DomainType::BackendRust | DomainType::BackendGo => vec![
            "src/domain/".to_string(),
            "src/application/".to_string(),
            "src/infrastructure/".to_string(),
            "README.md".to_string(),
            "CHANGELOG.md".to_string(),
        ],
        DomainType::FrontendReact => vec![
            "src/domain/".to_string(),
            "src/application/".to_string(),
            "README.md".to_string(),
            "CHANGELOG.md".to_string(),
        ],
        DomainType::FrontendFlutter => vec![
            "lib/src/domain/".to_string(),
            "lib/src/application/".to_string(),
            "README.md".to_string(),
            "CHANGELOG.md".to_string(),
        ],
    }
}

/// パス別の更新ポリシーを取得
fn get_update_policy(
    domain_type: DomainType,
) -> std::collections::HashMap<String, UpdatePolicy> {
    let mut policy = std::collections::HashMap::new();

    match domain_type {
        DomainType::BackendRust | DomainType::BackendGo => {
            policy.insert("src/domain/".to_string(), UpdatePolicy::Protected);
            policy.insert("src/application/".to_string(), UpdatePolicy::Protected);
            policy.insert("src/infrastructure/".to_string(), UpdatePolicy::Protected);
            policy.insert("README.md".to_string(), UpdatePolicy::SuggestOnly);
            policy.insert("CHANGELOG.md".to_string(), UpdatePolicy::SuggestOnly);
        }
        DomainType::FrontendReact | DomainType::FrontendFlutter => {
            policy.insert("README.md".to_string(), UpdatePolicy::SuggestOnly);
            policy.insert("CHANGELOG.md".to_string(), UpdatePolicy::SuggestOnly);
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

/// kebab-case を Title Case に変換する
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_domain_name_valid() {
        assert!(validate_domain_name("production").is_ok());
        assert!(validate_domain_name("user-management").is_ok());
        assert!(validate_domain_name("inventory").is_ok());
        assert!(validate_domain_name("api2").is_ok());
        assert!(validate_domain_name("order-processing-v2").is_ok());
    }

    #[test]
    fn test_validate_domain_name_empty() {
        let result = validate_domain_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_domain_name_not_kebab_case() {
        // CamelCase
        assert!(validate_domain_name("UserManagement").is_err());
        // snake_case
        assert!(validate_domain_name("user_management").is_err());
        // 先頭ハイフン
        assert!(validate_domain_name("-user").is_err());
        // 末尾ハイフン
        assert!(validate_domain_name("user-").is_err());
        // 連続ハイフン
        assert!(validate_domain_name("user--management").is_err());
        // 先頭数字
        assert!(validate_domain_name("2user").is_err());
        // 大文字を含む
        assert!(validate_domain_name("User").is_err());
    }

    #[test]
    fn test_validate_domain_name_reserved() {
        assert!(validate_domain_name("framework").is_err());
        assert!(validate_domain_name("feature").is_err());
        assert!(validate_domain_name("domain").is_err());
        assert!(validate_domain_name("k1s0").is_err());
        assert!(validate_domain_name("common").is_err());
        assert!(validate_domain_name("shared").is_err());
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user-management"), "UserManagement");
        assert_eq!(to_pascal_case("production"), "Production");
        assert_eq!(to_pascal_case("order-processing-v2"), "OrderProcessingV2");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("user-management"), "User Management");
        assert_eq!(to_title_case("production"), "Production");
        assert_eq!(to_title_case("order-processing-v2"), "Order Processing V2");
    }

    #[test]
    fn test_domain_type_output_base() {
        assert_eq!(DomainType::BackendRust.output_base(), "domain/backend/rust");
        assert_eq!(DomainType::BackendGo.output_base(), "domain/backend/go");
        assert_eq!(DomainType::FrontendReact.output_base(), "domain/frontend/react");
        assert_eq!(DomainType::FrontendFlutter.output_base(), "domain/frontend/flutter");
    }

    #[test]
    fn test_domain_type_template_path() {
        assert_eq!(DomainType::BackendRust.template_path(), "CLI/templates/backend-rust/domain");
        assert_eq!(DomainType::BackendGo.template_path(), "CLI/templates/backend-go/domain");
        assert_eq!(DomainType::FrontendReact.template_path(), "CLI/templates/frontend-react/domain");
        assert_eq!(DomainType::FrontendFlutter.template_path(), "CLI/templates/frontend-flutter/domain");
    }

    #[test]
    fn test_has_required_args() {
        let args_complete = NewDomainArgs {
            domain_type: Some(DomainType::BackendRust),
            name: Some("test".to_string()),
            output: None,
            force: false,
            interactive: false,
            with_events: false,
            with_repository: true,
            version: "0.1.0".to_string(),
        };
        assert!(args_complete.has_required_args());

        let args_missing_type = NewDomainArgs {
            domain_type: None,
            name: Some("test".to_string()),
            output: None,
            force: false,
            interactive: false,
            with_events: false,
            with_repository: true,
            version: "0.1.0".to_string(),
        };
        assert!(!args_missing_type.has_required_args());

        let args_missing_name = NewDomainArgs {
            domain_type: Some(DomainType::BackendRust),
            name: None,
            output: None,
            force: false,
            interactive: false,
            with_events: false,
            with_repository: true,
            version: "0.1.0".to_string(),
        };
        assert!(!args_missing_name.has_required_args());
    }
}
