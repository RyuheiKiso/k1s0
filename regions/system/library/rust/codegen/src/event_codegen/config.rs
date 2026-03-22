use serde::Deserialize;

/// Top-level configuration parsed from `events.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct EventConfig {
    /// Domain name in kebab-case (e.g. "taskmanagement").
    pub domain: String,
    /// Architecture tier.
    pub tier: String,
    /// Service name in kebab-case (e.g. "project-master").
    pub service_name: String,
    /// Target language: "rust" or "go".
    pub language: String,
    /// List of event definitions.
    pub events: Vec<EventDef>,
}

/// A single event definition.
#[derive(Debug, Clone, Deserialize)]
pub struct EventDef {
    /// Event name in kebab-case + dot-separated (e.g. "master-item.created").
    pub name: String,
    /// Schema version (>= 1).
    pub version: u32,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Field name used as Kafka partition key (must exist in schema.fields).
    pub partition_key: String,
    /// Whether to generate outbox support.
    #[serde(default = "default_true")]
    pub outbox: bool,
    /// Event schema definition.
    pub schema: EventSchema,
    /// Consumer definitions.
    #[serde(default)]
    pub consumers: Vec<ConsumerDef>,
}

/// Schema for an event.
#[derive(Debug, Clone, Deserialize)]
pub struct EventSchema {
    /// Field definitions.
    pub fields: Vec<FieldDef>,
}

/// A single field in an event schema.
#[derive(Debug, Clone, Deserialize)]
pub struct FieldDef {
    /// Field name in snake_case.
    pub name: String,
    /// Proto3 type (e.g. "string", "int32", "bool").
    #[serde(rename = "type")]
    pub field_type: String,
    /// Proto field number (>= 1).
    pub number: u32,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
}

/// A consumer definition.
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerDef {
    /// Consumer domain name.
    pub domain: String,
    /// Consumer service name.
    pub service_name: String,
    /// Handler function name in snake_case.
    pub handler: String,
}

fn default_true() -> bool {
    true
}

impl EventConfig {
    /// Build the Kafka topic name for an event.
    ///
    /// Pattern: `k1s0.{tier}.{domain}.{name-hyphenated}.v{version}`
    pub fn topic_name(&self, event: &EventDef) -> String {
        let name_hyphenated = event.name.replace('.', "-");
        format!(
            "k1s0.{}.{}.{}.v{}",
            self.tier, self.domain, name_hyphenated, event.version
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_yaml() -> &'static str {
        r#"
domain: taskmanagement
tier: business
service_name: project-master
language: rust

events:
  - name: project-type.changed
    version: 1
    description: "プロジェクトタイプが変更された時に発行されるイベント"
    partition_key: project_type_id
    outbox: true
    schema:
      fields:
        - name: project_type_id
          type: string
          number: 1
          description: "プロジェクトタイプID"
    consumers:
      - domain: fa
        service_name: asset-manager
        handler: on_taskmanagement_project_type_changed
"#
    }

    #[test]
    fn deserialize_yaml() {
        let config: EventConfig = serde_yaml::from_str(sample_yaml()).unwrap();
        assert_eq!(config.domain, "taskmanagement");
        assert_eq!(config.tier, "business");
        assert_eq!(config.service_name, "project-master");
        assert_eq!(config.language, "rust");
        assert_eq!(config.events.len(), 1);

        let event = &config.events[0];
        assert_eq!(event.name, "project-type.changed");
        assert_eq!(event.version, 1);
        assert!(event.outbox);
        assert_eq!(event.partition_key, "project_type_id");
        assert_eq!(event.schema.fields.len(), 1);
        assert_eq!(event.consumers.len(), 1);
    }

    #[test]
    fn topic_name_format() {
        let config: EventConfig = serde_yaml::from_str(sample_yaml()).unwrap();
        let topic = config.topic_name(&config.events[0]);
        assert_eq!(topic, "k1s0.business.taskmanagement.project-type-changed.v1");
    }
}
