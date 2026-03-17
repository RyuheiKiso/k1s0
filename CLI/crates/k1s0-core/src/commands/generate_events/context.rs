use tera::Context;

use super::types::{ConsumerDefinition, EventDefinition, EventsConfig};

/// `EventsConfig` から Tera テンプレートコンテキストを構築する。
pub fn build_template_context(config: &EventsConfig) -> Context {
    let mut ctx = Context::new();

    ctx.insert("domain", &config.domain);
    ctx.insert("domain_snake", &config.domain.replace('-', "_"));
    ctx.insert("tier", &config.tier);
    ctx.insert("service_name", &config.service_name);
    ctx.insert("service_name_snake", &config.service_name.replace('-', "_"));
    ctx.insert("language", &config.language);
    ctx.insert("go_module_base", "github.com/org/k1s0");

    let has_outbox = config.events.iter().any(|e| e.outbox);
    ctx.insert("has_outbox", &has_outbox);

    let has_consumers = config.events.iter().any(|e| !e.consumers.is_empty());
    ctx.insert("has_consumers", &has_consumers);

    let events: Vec<serde_json::Value> = config
        .events
        .iter()
        .map(|event| build_event_json(config, event))
        .collect();

    ctx.insert("events", &events);
    ctx
}

/// 個別イベント + 個別 consumer 用のテンプレートコンテキストを構築する。
pub fn build_consumer_context(
    config: &EventsConfig,
    event: &EventDefinition,
    consumer: &ConsumerDefinition,
) -> Context {
    let mut ctx = build_base_context(config);

    let event_json = build_event_json(config, event);
    ctx.insert("event", &event_json);

    let has_timestamp = event
        .schema
        .fields
        .iter()
        .any(|f| f.field_type == "google.protobuf.Timestamp");
    ctx.insert("has_timestamp", &has_timestamp);

    // 個別 consumer データ
    let consumer_group = format!(
        "{}.{}.{}",
        consumer.domain,
        consumer.service_name,
        event.name_snake()
    );
    ctx.insert(
        "consumer",
        &serde_json::json!({
            "domain": consumer.domain,
            "service_name": consumer.service_name,
            "service_name_snake": consumer.service_name.replace('-', "_"),
            "handler": consumer.handler,
            "handler_pascal": snake_to_pascal(&consumer.handler),
            "consumer_group": consumer_group,
        }),
    );

    // events 配列もそのまま渡す（mod.rs テンプレート等で使用）
    let events: Vec<serde_json::Value> = config
        .events
        .iter()
        .map(|e| build_event_json(config, e))
        .collect();
    ctx.insert("events", &events);

    ctx
}

/// 個別イベント用のテンプレートコンテキストを構築する（Proto テンプレート等）。
pub fn build_single_event_context(config: &EventsConfig, event: &EventDefinition) -> Context {
    let mut ctx = build_base_context(config);

    let event_json = build_event_json(config, event);
    ctx.insert("event", &event_json);

    let has_timestamp = event
        .schema
        .fields
        .iter()
        .any(|f| f.field_type == "google.protobuf.Timestamp");
    ctx.insert("has_timestamp", &has_timestamp);

    let events: Vec<serde_json::Value> = config
        .events
        .iter()
        .map(|e| build_event_json(config, e))
        .collect();
    ctx.insert("events", &events);

    let has_consumers = config.events.iter().any(|e| !e.consumers.is_empty());
    ctx.insert("has_consumers", &has_consumers);

    ctx
}

/// 共通のベースコンテキストを構築する。
fn build_base_context(config: &EventsConfig) -> Context {
    let mut ctx = Context::new();
    ctx.insert("domain", &config.domain);
    ctx.insert("domain_snake", &config.domain.replace('-', "_"));
    ctx.insert("tier", &config.tier);
    ctx.insert("service_name", &config.service_name);
    ctx.insert("service_name_snake", &config.service_name.replace('-', "_"));
    ctx.insert("language", &config.language);
    ctx.insert("go_module_base", "github.com/org/k1s0");

    let has_outbox = config.events.iter().any(|e| e.outbox);
    ctx.insert("has_outbox", &has_outbox);

    ctx
}

/// イベント定義を JSON 値に変換する。
fn build_event_json(config: &EventsConfig, event: &EventDefinition) -> serde_json::Value {
    let topic = event.topic_name(&config.tier, &config.domain);
    let proto_package = format!(
        "k1s0.event.{}.{}.v{}",
        config.tier, config.domain, event.version
    );

    let fields: Vec<serde_json::Value> = event
        .schema
        .fields
        .iter()
        .map(|f| {
            serde_json::json!({
                "name": f.name,
                "name_pascal": snake_to_pascal(&f.name),
                "field_type": f.field_type,
                "number": f.number,
                "description": f.description,
                "rust_type": proto_to_rust_type(&f.field_type),
                "go_type": proto_to_go_type(&f.field_type),
            })
        })
        .collect();

    let consumers: Vec<serde_json::Value> = event
        .consumers
        .iter()
        .map(|c| {
            let consumer_group = format!("{}.{}.{}", c.domain, c.service_name, event.name_snake());
            serde_json::json!({
                "domain": c.domain,
                "service_name": c.service_name,
                "service_name_snake": c.service_name.replace('-', "_"),
                "handler": c.handler,
                "handler_pascal": snake_to_pascal(&c.handler),
                "consumer_group": consumer_group,
            })
        })
        .collect();

    // partition_key の PascalCase 版
    let partition_key_pascal = snake_to_pascal(&event.partition_key);

    serde_json::json!({
        "name": event.name,
        "name_snake": event.name_snake(),
        "name_pascal": event.proto_message_name(),
        "version": event.version,
        "description": event.description,
        "topic": topic,
        "partition_key": event.partition_key,
        "partition_key_pascal": partition_key_pascal,
        "outbox": event.outbox,
        "proto_package": proto_package,
        "fields": fields,
        "consumers": consumers,
    })
}

/// `snake_case` → `PascalCase` 変換
fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

/// Proto 型 → Rust 型
fn proto_to_rust_type(proto_type: &str) -> &str {
    match proto_type {
        "int32" => "i32",
        "int64" => "i64",
        "uint32" => "u32",
        "uint64" => "u64",
        "bool" => "bool",
        "float" => "f32",
        "double" => "f64",
        "bytes" => "Vec<u8>",
        "google.protobuf.Timestamp" => "chrono::DateTime<chrono::Utc>",
        // string およびその他の型は String にフォールバック
        _ => "String",
    }
}

/// Proto 型 → Go 型
fn proto_to_go_type(proto_type: &str) -> &str {
    match proto_type {
        "int32" => "int32",
        "int64" => "int64",
        "uint32" => "uint32",
        "uint64" => "uint64",
        "bool" => "bool",
        "float" => "float32",
        "double" => "float64",
        "bytes" => "[]byte",
        "google.protobuf.Timestamp" => "time.Time",
        // string およびその他の型は string にフォールバック
        _ => "string",
    }
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::commands::generate_events::types::*;

    fn sample_config() -> EventsConfig {
        EventsConfig {
            domain: "accounting".to_string(),
            tier: "business".to_string(),
            service_name: "domain-master".to_string(),
            language: "rust".to_string(),
            events: vec![EventDefinition {
                name: "master-item.created".to_string(),
                version: 1,
                description: "マスタアイテム作成".to_string(),
                partition_key: "item_id".to_string(),
                outbox: true,
                schema: EventSchema {
                    fields: vec![
                        SchemaField {
                            name: "item_id".to_string(),
                            field_type: "string".to_string(),
                            number: 1,
                            description: "アイテムID".to_string(),
                        },
                        SchemaField {
                            name: "amount".to_string(),
                            field_type: "int64".to_string(),
                            number: 2,
                            description: "金額".to_string(),
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
    fn test_build_template_context() {
        let config = sample_config();
        let ctx = build_template_context(&config);
        let json = ctx.into_json();

        assert_eq!(json["domain"], "accounting");
        assert_eq!(json["domain_snake"], "accounting");
        assert_eq!(json["tier"], "business");
        assert_eq!(json["service_name"], "domain-master");
        assert_eq!(json["service_name_snake"], "domain_master");
        assert_eq!(json["language"], "rust");
        assert_eq!(json["has_outbox"], true);
        assert_eq!(json["has_consumers"], true);

        let events = json["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event["name_snake"], "master_item_created");
        assert_eq!(event["name_pascal"], "MasterItemCreated");
        assert_eq!(
            event["topic"],
            "k1s0.business.accounting.master-item-created.v1"
        );
        assert_eq!(event["proto_package"], "k1s0.event.business.accounting.v1");
        assert_eq!(event["partition_key_pascal"], "ItemId");

        let fields = event["fields"].as_array().unwrap();
        assert_eq!(fields[0]["rust_type"], "String");
        assert_eq!(fields[0]["name_pascal"], "ItemId");
        assert_eq!(fields[1]["rust_type"], "i64");
        assert_eq!(fields[1]["go_type"], "int64");
        assert_eq!(fields[1]["name_pascal"], "Amount");

        let consumers = event["consumers"].as_array().unwrap();
        assert_eq!(consumers[0]["handler"], "on_accounting_master_item_created");
        assert_eq!(
            consumers[0]["handler_pascal"],
            "OnAccountingMasterItemCreated"
        );
        assert_eq!(
            consumers[0]["consumer_group"],
            "fa.asset-manager.master_item_created"
        );
    }

    #[test]
    fn test_has_outbox_false_when_no_outbox() {
        let mut config = sample_config();
        config.events[0].outbox = false;
        let ctx = build_template_context(&config);
        let json = ctx.into_json();
        assert_eq!(json["has_outbox"], false);
    }

    #[test]
    fn test_has_consumers_false() {
        let mut config = sample_config();
        config.events[0].consumers = vec![];
        let ctx = build_template_context(&config);
        let json = ctx.into_json();
        assert_eq!(json["has_consumers"], false);
    }

    #[test]
    fn test_build_consumer_context() {
        let config = sample_config();
        let event = &config.events[0];
        let consumer = &event.consumers[0];
        let ctx = build_consumer_context(&config, event, consumer);
        let json = ctx.into_json();

        assert_eq!(
            json["consumer"]["handler"],
            "on_accounting_master_item_created"
        );
        assert_eq!(
            json["consumer"]["handler_pascal"],
            "OnAccountingMasterItemCreated"
        );
        assert_eq!(json["event"]["name_pascal"], "MasterItemCreated");
    }

    #[test]
    fn test_build_single_event_context_with_timestamp() {
        let mut config = sample_config();
        config.events[0].schema.fields.push(SchemaField {
            name: "created_at".to_string(),
            field_type: "google.protobuf.Timestamp".to_string(),
            number: 3,
            description: "作成日時".to_string(),
        });
        let ctx = build_single_event_context(&config, &config.events[0]);
        let json = ctx.into_json();
        assert_eq!(json["has_timestamp"], true);
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("item_id"), "ItemId");
        assert_eq!(snake_to_pascal("on_item_created"), "OnItemCreated");
        assert_eq!(snake_to_pascal("simple"), "Simple");
        assert_eq!(snake_to_pascal("a_b_c"), "ABC");
    }

    #[test]
    fn test_proto_to_rust_type_coverage() {
        assert_eq!(proto_to_rust_type("string"), "String");
        assert_eq!(proto_to_rust_type("int32"), "i32");
        assert_eq!(proto_to_rust_type("bool"), "bool");
        assert_eq!(proto_to_rust_type("bytes"), "Vec<u8>");
        assert_eq!(
            proto_to_rust_type("google.protobuf.Timestamp"),
            "chrono::DateTime<chrono::Utc>"
        );
        assert_eq!(proto_to_rust_type("unknown"), "String");
    }

    #[test]
    fn test_proto_to_go_type_coverage() {
        assert_eq!(proto_to_go_type("string"), "string");
        assert_eq!(proto_to_go_type("int64"), "int64");
        assert_eq!(proto_to_go_type("bytes"), "[]byte");
        assert_eq!(proto_to_go_type("google.protobuf.Timestamp"), "time.Time");
    }
}
