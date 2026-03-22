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
        placement: Some("task".to_string()),
        lang_fw: LangFw::Language(Language::Rust),
        detail: DetailConfig {
            name: Some("task".to_string()),
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
        placement: Some("task".to_string()),
        lang_fw: LangFw::Language(Language::Rust),
        detail: DetailConfig {
            name: Some("task".to_string()),
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
    assert_eq!(ctx.rust_crate, "task");
    assert_eq!(ctx.module_path, "regions/service/task/server/rust");
}

#[test]
fn test_build_template_context_react_client_language_framework() {
    let config = GenerateConfig {
        kind: Kind::Client,
        tier: Tier::Business,
        placement: Some("taskmanagement".to_string()),
        lang_fw: LangFw::Framework(Framework::React),
        detail: DetailConfig {
            name: Some("taskmanagement-web".to_string()),
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
        placement: Some("task".to_string()),
        lang_fw: LangFw::Framework(Framework::Flutter),
        detail: DetailConfig {
            name: Some("task-app".to_string()),
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

#[test]
fn test_build_template_context_business_tier_sets_domain() {
    // business tier では placement から domain が正しく設定されることを検証する
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Business,
        placement: Some("taskmanagement".to_string()),
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("project-master".to_string()),
            api_styles: vec![ApiStyle::Grpc],
            db: None,
            kafka: false,
            redis: false,
            bff_language: None,
        },
    };
    let cli_config = CliConfig::default();
    let ctx = build_template_context(&config, &cli_config).unwrap();
    assert_eq!(
        ctx.domain, "taskmanagement",
        "business tier では domain が placement から設定される必要がある"
    );
    assert_eq!(ctx.tier, "business");
    // module_path に domain が含まれることを検証する
    assert!(
        ctx.module_path.contains("taskmanagement"),
        "module_path に domain が含まれる必要がある: {}",
        ctx.module_path
    );
}

#[test]
fn test_build_template_context_business_tier_empty_domain_fails_validation() {
    // business tier で domain が空の場合に validate() がエラーを返すことを検証する
    use crate::template::context::TemplateContextBuilder;

    let builder = TemplateContextBuilder::new("project-master", "business", "go", "server");
    // domain を設定しない（空のまま）ので validate() はエラーになる
    let result = builder.try_build();
    assert!(
        result.is_err(),
        "business tier で domain が未設定の場合、try_build() はエラーを返す必要がある"
    );
    let error = result.unwrap_err();
    assert!(
        error.contains("domain"),
        "エラーメッセージに 'domain' が含まれる必要がある: {error}"
    );
}
