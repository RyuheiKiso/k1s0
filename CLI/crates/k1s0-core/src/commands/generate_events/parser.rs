use anyhow::{bail, Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::sync::OnceLock;

use super::types::EventsConfig;

/// kebab-case パターンの正規表現キャッシュ
static KEBAB_RE: OnceLock<Regex> = OnceLock::new();
/// イベント名パターンの正規表現キャッシュ
static EVENT_NAME_RE: OnceLock<Regex> = OnceLock::new();
/// snake_case パターンの正規表現キャッシュ
static SNAKE_RE: OnceLock<Regex> = OnceLock::new();

/// 許可されるフィールド型 (proto3)
const VALID_FIELD_TYPES: &[&str] = &[
    "string",
    "int32",
    "int64",
    "uint32",
    "uint64",
    "bool",
    "float",
    "double",
    "bytes",
    "google.protobuf.Timestamp",
];

/// `events.yaml` を読み込み、バリデーション済みの `EventsConfig` を返す。
///
/// # Errors
///
/// ファイルの読み込み・パース・バリデーションに失敗した場合にエラーを返す。
pub fn parse_events_yaml(path: &str) -> Result<EventsConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("events.yaml を読み込めません: {path}"))?;
    let config: EventsConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("events.yaml のパースに失敗しました: {path}"))?;
    validate(&config)?;
    Ok(config)
}

/// `EventsConfig` のバリデーションを行う。
///
/// # Errors
///
/// バリデーションルールに違反するフィールドが見つかった場合にエラーを返す。
///
/// # Panics
///
/// 正規表現のコンパイルに失敗した場合にパニックする（定数パターンのため発生しない）。
pub fn validate(config: &EventsConfig) -> Result<()> {
    // 静的正規表現のコンパイル失敗はプログラミングエラーのため expect で即時パニックする
    let kebab_re = KEBAB_RE.get_or_init(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("static regex"));
    let event_name_re = EVENT_NAME_RE.get_or_init(|| {
        Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*(\.[a-z0-9]+(-[a-z0-9]+)*)+$").expect("static regex")
    });
    let snake_re = SNAKE_RE.get_or_init(|| Regex::new(r"^[a-z_][a-z0-9_]*$").expect("static regex"));

    // domain
    if !kebab_re.is_match(&config.domain) {
        bail!(
            "domain '{}' は kebab-case でなければなりません",
            config.domain
        );
    }

    // tier
    if !matches!(config.tier.as_str(), "system" | "business" | "service") {
        bail!(
            "tier '{}' は system, business, service のいずれかでなければなりません",
            config.tier
        );
    }

    // service_name
    if !kebab_re.is_match(&config.service_name) {
        bail!(
            "service_name '{}' は kebab-case でなければなりません",
            config.service_name
        );
    }

    // language
    if !matches!(config.language.as_str(), "rust" | "go") {
        bail!(
            "language '{}' は rust, go のいずれかでなければなりません",
            config.language
        );
    }

    // events: 1つ以上
    if config.events.is_empty() {
        bail!("events は1つ以上定義する必要があります");
    }

    // イベント名の重複チェック
    let mut event_names = HashSet::new();
    for event in &config.events {
        if !event_names.insert(&event.name) {
            bail!("イベント名 '{}' が重複しています", event.name);
        }

        // event.name
        if !event_name_re.is_match(&event.name) {
            bail!(
                "event.name '{}' は kebab-case + ドット区切り (例: master-item.created) でなければなりません",
                event.name
            );
        }

        // event.version
        if event.version < 1 {
            bail!(
                "event '{}' の version は 1 以上でなければなりません",
                event.name
            );
        }

        // schema.fields: 1つ以上
        if event.schema.fields.is_empty() {
            bail!(
                "event '{}' の schema.fields は1つ以上定義する必要があります",
                event.name
            );
        }

        // field.number の重複チェック + 型チェック
        let mut field_numbers = HashSet::new();
        let mut field_names = HashSet::new();
        for field in &event.schema.fields {
            if field.number < 1 {
                bail!(
                    "event '{}' の field '{}' の number は 1 以上でなければなりません",
                    event.name,
                    field.name
                );
            }
            if !field_numbers.insert(field.number) {
                bail!(
                    "event '{}' の field number {} が重複しています",
                    event.name,
                    field.number
                );
            }
            field_names.insert(field.name.as_str());

            if !VALID_FIELD_TYPES.contains(&field.field_type.as_str()) {
                bail!(
                    "event '{}' の field '{}' の type '{}' は無効です。有効な型: {}",
                    event.name,
                    field.name,
                    field.field_type,
                    VALID_FIELD_TYPES.join(", ")
                );
            }
        }

        // partition_key がフィールドに存在するか
        if !field_names.contains(event.partition_key.as_str()) {
            bail!(
                "event '{}' の partition_key '{}' は schema.fields に定義されていません",
                event.name,
                event.partition_key
            );
        }

        // consumer.handler のバリデーション
        for consumer in &event.consumers {
            if !snake_re.is_match(&consumer.handler) {
                bail!(
                    "event '{}' の consumer handler '{}' は snake_case でなければなりません",
                    event.name,
                    consumer.handler
                );
            }
        }
    }

    Ok(())
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::commands::generate_events::types::*;

    fn valid_config() -> EventsConfig {
        EventsConfig {
            domain: "accounting".to_string(),
            tier: "business".to_string(),
            service_name: "domain-master".to_string(),
            language: "rust".to_string(),
            events: vec![EventDefinition {
                name: "master-item.created".to_string(),
                version: 1,
                description: "test".to_string(),
                partition_key: "item_id".to_string(),
                outbox: true,
                schema: EventSchema {
                    fields: vec![
                        SchemaField {
                            name: "item_id".to_string(),
                            field_type: "string".to_string(),
                            number: 1,
                            description: String::new(),
                        },
                        SchemaField {
                            name: "category_id".to_string(),
                            field_type: "string".to_string(),
                            number: 2,
                            description: String::new(),
                        },
                    ],
                },
                consumers: vec![ConsumerDefinition {
                    domain: "fa".to_string(),
                    service_name: "asset-manager".to_string(),
                    handler: "on_accounting_master_item_created".to_string(),
                }],
            }],
        }
    }

    #[test]
    fn test_valid_config_passes() {
        assert!(validate(&valid_config()).is_ok());
    }

    #[test]
    fn test_invalid_domain() {
        let mut config = valid_config();
        config.domain = "UPPER".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_invalid_tier() {
        let mut config = valid_config();
        config.tier = "invalid".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_invalid_language() {
        let mut config = valid_config();
        config.language = "python".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_empty_events() {
        let mut config = valid_config();
        config.events = vec![];
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_invalid_event_name() {
        let mut config = valid_config();
        config.events[0].name = "simple".to_string(); // ドットなし
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_duplicate_event_names() {
        let mut config = valid_config();
        let event_copy = config.events[0].clone();
        config.events.push(event_copy);
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_duplicate_field_numbers() {
        let mut config = valid_config();
        config.events[0].schema.fields[1].number = 1; // number 1 の重複
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_invalid_field_type() {
        let mut config = valid_config();
        config.events[0].schema.fields[0].field_type = "invalid".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_partition_key_not_in_fields() {
        let mut config = valid_config();
        config.events[0].partition_key = "nonexistent".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_invalid_handler_name() {
        let mut config = valid_config();
        config.events[0].consumers[0].handler = "CamelCase".to_string();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_empty_schema_fields() {
        let mut config = valid_config();
        config.events[0].schema.fields = vec![];
        assert!(validate(&config).is_err());
    }

    #[test]
    fn test_timestamp_field_type_valid() {
        let mut config = valid_config();
        config.events[0].schema.fields[0].field_type = "google.protobuf.Timestamp".to_string();
        assert!(validate(&config).is_ok());
    }
}
