//! テンプレートタイプ選択プロンプト
//!
//! サービスタイプ、ドメインタイプ、フロントエンドタイプの選択を提供します。

use inquire::Select;

use crate::commands::new_domain::DomainType;
use crate::commands::new_feature::ServiceType;
use crate::commands::new_screen::FrontendType;
use crate::error::Result;
use crate::prompts::{cancelled_error, get_render_config};

/// サービスタイプの選択肢
struct ServiceTypeOption {
    service_type: ServiceType,
    label: &'static str,
    description: &'static str,
}

impl std::fmt::Display for ServiceTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.label, self.description)
    }
}

/// サービスタイプを選択するプロンプト
///
/// 4 つのサービスタイプから 1 つを選択できます。
///
/// # Returns
///
/// 選択された `ServiceType`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_service_type() -> Result<ServiceType> {
    let options = vec![
        ServiceTypeOption {
            service_type: ServiceType::BackendRust,
            label: "backend-rust",
            description: "Rust バックエンドサービス (axum, tokio)",
        },
        ServiceTypeOption {
            service_type: ServiceType::BackendGo,
            label: "backend-go",
            description: "Go バックエンドサービス",
        },
        ServiceTypeOption {
            service_type: ServiceType::BackendCsharp,
            label: "backend-csharp",
            description: "C# バックエンドサービス (ASP.NET Core)",
        },
        ServiceTypeOption {
            service_type: ServiceType::BackendPython,
            label: "backend-python",
            description: "Python バックエンドサービス (FastAPI)",
        },
        ServiceTypeOption {
            service_type: ServiceType::FrontendReact,
            label: "frontend-react",
            description: "React フロントエンドアプリ (TypeScript, Material-UI)",
        },
        ServiceTypeOption {
            service_type: ServiceType::FrontendFlutter,
            label: "frontend-flutter",
            description: "Flutter フロントエンドアプリ (Dart)",
        },
    ];

    let answer = Select::new("サービスタイプを選択してください:", options)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.service_type)
}

/// ドメインタイプの選択肢
struct DomainTypeOption {
    domain_type: DomainType,
    label: &'static str,
    description: &'static str,
}

impl std::fmt::Display for DomainTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.label, self.description)
    }
}

/// ドメインタイプを選択するプロンプト
///
/// # Returns
///
/// 選択された `DomainType`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_domain_type() -> Result<DomainType> {
    let options = vec![
        DomainTypeOption {
            domain_type: DomainType::BackendRust,
            label: "backend-rust",
            description: "Rust ドメインクレート",
        },
        DomainTypeOption {
            domain_type: DomainType::BackendGo,
            label: "backend-go",
            description: "Go ドメインモジュール",
        },
        DomainTypeOption {
            domain_type: DomainType::BackendCsharp,
            label: "backend-csharp",
            description: "C# ドメインプロジェクト",
        },
        DomainTypeOption {
            domain_type: DomainType::BackendPython,
            label: "backend-python",
            description: "Python ドメインパッケージ (FastAPI)",
        },
        DomainTypeOption {
            domain_type: DomainType::FrontendReact,
            label: "frontend-react",
            description: "React ドメインパッケージ (TypeScript)",
        },
        DomainTypeOption {
            domain_type: DomainType::FrontendFlutter,
            label: "frontend-flutter",
            description: "Flutter ドメインパッケージ (Dart)",
        },
    ];

    let answer = Select::new("ドメインタイプを選択してください:", options)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.domain_type)
}

/// フロントエンドタイプの選択肢
struct FrontendTypeOption {
    frontend_type: FrontendType,
    label: &'static str,
    description: &'static str,
}

impl std::fmt::Display for FrontendTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.label, self.description)
    }
}

/// フロントエンドタイプを選択するプロンプト
///
/// # Returns
///
/// 選択された `FrontendType`
///
/// # Errors
///
/// ユーザーがキャンセルした場合
pub fn select_frontend_type() -> Result<FrontendType> {
    let options = vec![
        FrontendTypeOption {
            frontend_type: FrontendType::React,
            label: "react",
            description: "React フロントエンド (TypeScript, Material-UI)",
        },
        FrontendTypeOption {
            frontend_type: FrontendType::Flutter,
            label: "flutter",
            description: "Flutter フロントエンド (Dart)",
        },
    ];

    let answer = Select::new("フロントエンドタイプを選択してください:", options)
        .with_render_config(get_render_config())
        .with_help_message("矢印キーで選択、Enter で確定")
        .prompt()
        .map_err(|_| cancelled_error())?;

    Ok(answer.frontend_type)
}
