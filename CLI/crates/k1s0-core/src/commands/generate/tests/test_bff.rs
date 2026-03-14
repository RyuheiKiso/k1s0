// BFF（Backend For Frontend）生成のテスト。
// service Tier + GraphQL の条件で BFF ディレクトリが正しく生成されること、
// 条件を満たさない場合に生成されないことを検証する。

use tempfile::TempDir;

use crate::commands::generate::execute::execute_generate_at;
use crate::commands::generate::types::{
    ApiStyle, DetailConfig, GenerateConfig, Kind, LangFw, Language, Tier,
};

#[test]
fn test_service_tier_graphql_creates_bff_directory() {
    // service Tier + GraphQL + Go 言語サーバー (BFF 用) で、bff/ ディレクトリが追加生成される
    let tmp = TempDir::new().unwrap();
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Service,
        placement: Some("order".to_string()),
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("order".to_string()),
            api_styles: vec![ApiStyle::GraphQL],
            db: None,
            kafka: false,
            redis: false,
            bff_language: Some(Language::Go),
        },
    };
    execute_generate_at(&config, tmp.path()).unwrap();
    // BFF ディレクトリが存在するか確認
    let bff_path = tmp.path().join("regions/service/order/server/go/bff");
    assert!(
        bff_path.exists(),
        "service Tier + GraphQL should create bff/ directory"
    );
}

#[test]
fn test_bff_not_created_for_system_tier_graphql() {
    // system Tier の GraphQL では BFF ディレクトリは作成されない
    let tmp = TempDir::new().unwrap();
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::System,
        placement: None,
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("gateway".to_string()),
            api_styles: vec![ApiStyle::GraphQL],
            db: None,
            kafka: false,
            redis: false,
            bff_language: Some(Language::Go),
        },
    };
    execute_generate_at(&config, tmp.path()).unwrap();
    let bff_path = tmp.path().join("regions/system/server/go/gateway/bff");
    assert!(
        !bff_path.exists(),
        "system Tier では BFF ディレクトリは作成されない"
    );
}

#[test]
fn test_bff_not_created_for_business_tier_graphql() {
    // business Tier の GraphQL では BFF ディレクトリは作成されない
    let tmp = TempDir::new().unwrap();
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Business,
        placement: Some("accounting".to_string()),
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("ledger".to_string()),
            api_styles: vec![ApiStyle::GraphQL],
            db: None,
            kafka: false,
            redis: false,
            bff_language: Some(Language::Go),
        },
    };
    execute_generate_at(&config, tmp.path()).unwrap();
    let bff_path = tmp
        .path()
        .join("regions/business/accounting/server/go/ledger/bff");
    assert!(
        !bff_path.exists(),
        "business Tier では BFF ディレクトリは作成されない"
    );
}

#[test]
fn test_bff_directory_created_with_language() {
    // service Tier + GraphQL + bff_language=Go の場合に bff/ が作成される
    let tmp = TempDir::new().unwrap();
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Service,
        placement: Some("order".to_string()),
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("order".to_string()),
            api_styles: vec![ApiStyle::GraphQL],
            db: None,
            kafka: false,
            redis: false,
            bff_language: Some(Language::Go),
        },
    };
    execute_generate_at(&config, tmp.path()).unwrap();
    let bff_path = tmp.path().join("regions/service/order/server/go/bff");
    assert!(
        bff_path.exists(),
        "service Tier + GraphQL + bff_language=Go で bff/ が作成される"
    );
}

#[test]
fn test_bff_not_created_when_no_graphql() {
    // GraphQL なしの場合は BFF は作成されない
    let tmp = TempDir::new().unwrap();
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Service,
        placement: Some("order".to_string()),
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("order".to_string()),
            api_styles: vec![ApiStyle::Rest],
            db: None,
            kafka: false,
            redis: false,
            bff_language: None,
        },
    };
    execute_generate_at(&config, tmp.path()).unwrap();
    let bff_path = tmp.path().join("regions/service/order/server/go/bff");
    assert!(
        !bff_path.exists(),
        "GraphQL なしでは BFF ディレクトリは作成されない"
    );
}

#[test]
fn test_bff_not_created_when_bff_language_none() {
    let tmp = TempDir::new().unwrap();
    let config = GenerateConfig {
        kind: Kind::Server,
        tier: Tier::Service,
        placement: Some("order".to_string()),
        lang_fw: LangFw::Language(Language::Go),
        detail: DetailConfig {
            name: Some("order".to_string()),
            api_styles: vec![ApiStyle::GraphQL],
            db: None,
            kafka: false,
            redis: false,
            bff_language: None,
        },
    };
    execute_generate_at(&config, tmp.path()).unwrap();
    let bff_path = tmp.path().join("regions/service/order/server/go/bff");
    assert!(
        !bff_path.exists(),
        "bff_language=None では BFF ディレクトリは生成されない"
    );
}
