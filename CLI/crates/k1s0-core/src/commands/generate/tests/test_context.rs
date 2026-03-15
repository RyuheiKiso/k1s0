// TemplateContext 構築のテスト。
// GenerateConfig から正しい TemplateContext が生成されることを検証する。

use crate::commands::generate::context::build_template_context;
use crate::commands::generate::types::{
    ApiStyle, DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Tier,
};
use crate::config::CliConfig;

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
