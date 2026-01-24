//! `k1s0 new-feature` コマンド
//!
//! 新規サービスの雛形を生成する。

use anyhow::Result;
use clap::{Args, ValueEnum};

/// サービスタイプ
#[derive(ValueEnum, Clone, Debug)]
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
    // サービス名のバリデーション（kebab-case）
    if !is_valid_kebab_case(&args.name) {
        anyhow::bail!(
            "サービス名は kebab-case で指定してください: {}",
            args.name
        );
    }

    let output_path = args.output.clone().unwrap_or_else(|| {
        let type_path = match args.service_type {
            ServiceType::BackendRust => "feature/backend/rust",
            ServiceType::BackendGo => "feature/backend/go",
            ServiceType::FrontendReact => "feature/frontend/react",
            ServiceType::FrontendFlutter => "feature/frontend/flutter",
        };
        format!("{}/{}", type_path, args.name)
    });

    println!("k1s0 new-feature");
    println!("  type: {}", args.service_type);
    println!("  name: {}", args.name);
    println!("  output: {}", output_path);
    println!("  force: {}", args.force);
    println!("  with_grpc: {}", args.with_grpc);
    println!("  with_rest: {}", args.with_rest);
    println!("  with_db: {}", args.with_db);
    println!();
    println!("TODO: 実装予定（フェーズ12）");
    println!();
    println!("実行内容:");
    println!("  1. テンプレートから雛形を生成");
    println!("  2. .k1s0/manifest.json を作成");
    println!("  3. 生成後の k1s0 lint を実行");

    Ok(())
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
