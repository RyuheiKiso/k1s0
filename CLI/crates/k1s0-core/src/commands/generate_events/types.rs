use serde::Deserialize;

fn default_true() -> bool {
    true
}

/// events.yaml のルート構造
#[derive(Debug, Clone, Deserialize)]
pub struct EventsConfig {
    pub domain: String,
    pub tier: String,
    pub service_name: String,
    pub language: String,
    pub events: Vec<EventDefinition>,
}

/// 個別イベント定義
#[derive(Debug, Clone, Deserialize)]
pub struct EventDefinition {
    pub name: String,
    pub version: u32,
    pub description: String,
    pub partition_key: String,
    #[serde(default = "default_true")]
    pub outbox: bool,
    pub schema: EventSchema,
    #[serde(default)]
    pub consumers: Vec<ConsumerDefinition>,
}

/// イベントスキーマ（Proto フィールド定義）
#[derive(Debug, Clone, Deserialize)]
pub struct EventSchema {
    pub fields: Vec<SchemaField>,
}

/// スキーマフィールド
#[derive(Debug, Clone, Deserialize)]
pub struct SchemaField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub number: u32,
    #[serde(default)]
    pub description: String,
}

/// Consumer 定義
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerDefinition {
    pub domain: String,
    pub service_name: String,
    pub handler: String,
}

impl EventDefinition {
    /// トピック名を生成する。
    /// 形式: `k1s0.{tier}.{domain}.{name_hyphenated}.v{version}`
    #[must_use]
    pub fn topic_name(&self, tier: &str, domain: &str) -> String {
        let hyphenated = self.name.replace('.', "-");
        format!("k1s0.{tier}.{domain}.{hyphenated}.v{}", self.version)
    }

    /// Proto メッセージ名 (`PascalCase`)
    #[must_use]
    pub fn proto_message_name(&self) -> String {
        self.name
            .split(&['-', '.'][..])
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect()
    }

    /// `snake_case` 名
    #[must_use]
    pub fn name_snake(&self) -> String {
        self.name.replace(&['-', '.'][..], "_")
    }
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_event() -> EventDefinition {
        EventDefinition {
            name: "master-item.created".to_string(),
            version: 1,
            description: "test".to_string(),
            partition_key: "item_id".to_string(),
            outbox: true,
            schema: EventSchema {
                fields: vec![SchemaField {
                    name: "item_id".to_string(),
                    field_type: "string".to_string(),
                    number: 1,
                    description: String::new(),
                }],
            },
            consumers: vec![],
        }
    }

    #[test]
    fn test_topic_name() {
        let event = sample_event();
        assert_eq!(
            event.topic_name("business", "accounting"),
            "k1s0.business.accounting.master-item-created.v1"
        );
    }

    #[test]
    fn test_proto_message_name() {
        let event = sample_event();
        assert_eq!(event.proto_message_name(), "MasterItemCreated");
    }

    #[test]
    fn test_name_snake() {
        let event = sample_event();
        assert_eq!(event.name_snake(), "master_item_created");
    }

    #[test]
    fn test_default_outbox_true() {
        let yaml = r"
name: test.event
version: 1
description: test
partition_key: id
schema:
  fields:
    - name: id
      type: string
      number: 1
";
        let event: EventDefinition = serde_yaml::from_str(yaml).unwrap();
        assert!(event.outbox);
    }

    #[test]
    fn test_deserialize_events_config() {
        let yaml = r#"
domain: accounting
tier: business
service_name: domain-master
language: rust
events:
  - name: master-item.created
    version: 1
    description: "マスタアイテム作成"
    partition_key: item_id
    schema:
      fields:
        - name: item_id
          type: string
          number: 1
    consumers:
      - domain: fa
        service_name: asset-manager
        handler: on_accounting_master_item_created
"#;
        let config: EventsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.domain, "accounting");
        assert_eq!(config.events.len(), 1);
        assert_eq!(config.events[0].consumers.len(), 1);
    }
}
