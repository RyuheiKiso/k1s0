//! `k1s0 new-feature` コマンド
//!
//! 新規サービスの雛形を生成する。

use std::path::PathBuf;

use chrono::Utc;
use clap::{Args, ValueEnum};

use k1s0_generator::fingerprint::calculate_fingerprint;
use k1s0_generator::manifest::{
    Manifest, ServiceInfo, TemplateInfo, UpdatePolicy, SCHEMA_VERSION,
};
use k1s0_generator::template::TemplateRenderer;
use k1s0_generator::Context;

use crate::error::{CliError, Result};
use crate::output::output;
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
    /// Flutter フロントエンド
    #[value(name = "frontend-flutter")]
    FrontendFlutter,
}

impl ServiceType {
    /// テンプレートディレクトリの相対パスを取得
    fn template_path(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "CLI/templates/backend-rust/feature",
            ServiceType::BackendGo => "CLI/templates/backend-go/feature",
            ServiceType::FrontendReact => "CLI/templates/frontend-react/feature",
            ServiceType::FrontendFlutter => "CLI/templates/frontend-flutter/feature",
        }
    }

    /// 出力ディレクトリのベースパスを取得
    fn output_base(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "feature/backend/rust",
            ServiceType::BackendGo => "feature/backend/go",
            ServiceType::FrontendReact => "feature/frontend/react",
            ServiceType::FrontendFlutter => "feature/frontend/flutter",
        }
    }

    /// 言語名を取得
    fn language(&self) -> &'static str {
        match self {
            ServiceType::BackendRust => "rust",
            ServiceType::BackendGo => "go",
            ServiceType::FrontendReact => "typescript",
            ServiceType::FrontendFlutter => "dart",
        }
    }

    /// サービスタイプ名を取得
    fn service_type_name(&self) -> &'static str {
        match self {
            ServiceType::BackendRust | ServiceType::BackendGo => "backend",
            ServiceType::FrontendReact | ServiceType::FrontendFlutter => "frontend",
        }
    }
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::BackendRust => write!(f, "backend-rust"),
            ServiceType::BackendGo => write!(f, "backend-go"),
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
    pub service_type: ServiceType,

    /// サービス名（kebab-case）
    #[arg(short, long)]
    pub name: String,

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
}

/// `k1s0 new-feature` を実行する
pub fn execute(args: NewFeatureArgs) -> Result<()> {
    let out = output();

    // サービス名のバリデーション（kebab-case）
    if !is_valid_kebab_case(&args.name) {
        return Err(CliError::invalid_service_name(&args.name));
    }

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
    let manifest = create_manifest(&args, &template_dir, &fingerprint)?;
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
fn create_template_context(args: &NewFeatureArgs) -> Context {
    let mut context = Context::new();

    // 基本情報
    context.insert("feature_name", &args.name);
    context.insert("service_name", &args.name);
    context.insert("language", args.service_type.language());
    context.insert("service_type", args.service_type.service_type_name());
    context.insert("k1s0_version", version());

    // 命名規則の変換
    context.insert("feature_name_snake", &args.name.replace('-', "_"));
    context.insert("feature_name_pascal", &to_pascal_case(&args.name));

    // オプション
    context.insert("with_grpc", &args.with_grpc);
    context.insert("with_rest", &args.with_rest);
    context.insert("with_db", &args.with_db);

    context
}

/// manifest.json を作成する
fn create_manifest(
    args: &NewFeatureArgs,
    _template_dir: &PathBuf,
    fingerprint: &str,
) -> Result<Manifest> {
    let managed_paths = get_managed_paths(args.service_type);
    let protected_paths = get_protected_paths(args.service_type);
    let update_policy = get_update_policy(args.service_type);

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
        generated_at: Utc::now().to_rfc3339(),
        managed_paths,
        protected_paths,
        update_policy,
        checksums: std::collections::HashMap::new(),
        dependencies: None,
    })
}

/// CLI が管理するパスを取得
fn get_managed_paths(service_type: ServiceType) -> Vec<String> {
    match service_type {
        ServiceType::BackendRust => vec![
            "deploy/".to_string(),
            "buf.yaml".to_string(),
            "buf.gen.yaml".to_string(),
        ],
        ServiceType::BackendGo => vec![
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
        ServiceType::BackendRust | ServiceType::BackendGo => vec![
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
        ServiceType::BackendRust | ServiceType::BackendGo => {
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
}
