// 生成実行の統合テスト。
// execute_generate_at / execute_generate_with_config が
// 各 Kind/言語の組み合わせで正しくファイルを生成することを検証する。

use tempfile::TempDir;

use crate::commands::generate::execute::{execute_generate_at, execute_generate_with_config};
use crate::commands::generate::types::{
    ApiStyle, DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Rdbms, Tier,
};
use crate::config::CliConfig;

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
