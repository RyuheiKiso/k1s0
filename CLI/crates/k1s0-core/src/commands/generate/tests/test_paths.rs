// 出力先パス構築のテスト。
// build_output_path が各 Kind/Tier/LangFw の組み合わせで
// 正しいパスを生成することを検証する。

use std::path::{Path, PathBuf};

use crate::commands::generate::paths::{build_ci_workflow_path, build_output_path};
use crate::commands::generate::types::{
    DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Rdbms, Tier,
};

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
