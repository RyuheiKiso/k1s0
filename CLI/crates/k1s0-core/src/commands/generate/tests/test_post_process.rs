// 後処理コマンド決定のテスト。
// determine_post_commands が各設定に対して正しいコマンドリストを返すことを検証する。

use crate::commands::generate::post_process::determine_post_commands;
use crate::commands::generate::types::{
    ApiStyle, DbInfo, DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Rdbms, Tier,
};

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
