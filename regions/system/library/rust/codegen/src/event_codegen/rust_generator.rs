use super::config::{EventConfig, EventDef};
use crate::naming;

/// Generate the `src/events/mod.rs` module file.
pub fn generate_events_mod(config: &EventConfig) -> String {
    let mut lines = Vec::new();
    lines.push("pub mod producer;".to_string());
    lines.push("pub mod types;".to_string());

    // Add consumer module declarations
    let has_consumers = config.events.iter().any(|e| !e.consumers.is_empty());
    if has_consumers {
        lines.push("pub mod consumers;".to_string());
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Generate the `src/events/types.rs` file with event type structs.
pub fn generate_types(config: &EventConfig) -> String {
    let mut lines = Vec::new();
    lines.push("use serde::{Deserialize, Serialize};".to_string());
    lines.push(String::new());

    for event in &config.events {
        let struct_name = event_to_pascal(&event.name);

        if !event.description.is_empty() {
            lines.push(format!("/// {}", event.description));
        }
        lines.push("#[derive(Debug, Clone, Serialize, Deserialize)]".to_string());
        lines.push(format!("pub struct {struct_name} {{"));

        for field in &event.schema.fields {
            if !field.description.is_empty() {
                lines.push(format!("    /// {}", field.description));
            }
            let rust_type = proto_type_to_rust(&field.field_type);
            lines.push(format!("    pub {}: {rust_type},", field.name));
        }

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate the `src/events/producer.rs` file with producer functions.
pub fn generate_producer(config: &EventConfig) -> String {
    let mut lines = Vec::new();
    lines.push(
        "use k1s0_messaging::{EventEnvelope, EventMetadata, KafkaEventProducer};".to_string(),
    );

    let has_outbox = config.events.iter().any(|e| e.outbox);
    if has_outbox {
        lines.push("use k1s0_outbox::{OutboxMessage, OutboxStore};".to_string());
    }

    lines.push(String::new());
    lines.push("use super::types::*;".to_string());
    lines.push(String::new());

    for event in &config.events {
        let struct_name = event_to_pascal(&event.name);
        let fn_name = format!("publish_{}", event_to_snake(&event.name));
        let topic = config.topic_name(event);

        lines.push(format!("/// Publish a {struct_name} event to Kafka."));
        lines.push(format!(
            "pub async fn {fn_name}(producer: &KafkaEventProducer, event: {struct_name}) -> Result<(), Box<dyn std::error::Error>> {{"
        ));
        lines.push(format!("    let topic = \"{topic}\";"));
        lines.push(format!(
            "    let partition_key = event.{}.clone();",
            event.partition_key
        ));
        lines.push("    let metadata = EventMetadata::new(topic.to_string());".to_string());
        lines.push("    let envelope = EventEnvelope::new(metadata, event);".to_string());
        lines.push("    producer.send(topic, &partition_key, &envelope).await?;".to_string());
        lines.push("    Ok(())".to_string());
        lines.push("}".to_string());
        lines.push(String::new());

        // Outbox variant
        if event.outbox {
            let outbox_fn = format!("publish_{}_via_outbox", event_to_snake(&event.name));
            lines.push(format!(
                "/// Publish a {struct_name} event via the outbox pattern."
            ));
            lines.push(format!(
                "pub async fn {outbox_fn}(store: &dyn OutboxStore, event: {struct_name}) -> Result<(), Box<dyn std::error::Error>> {{"
            ));
            lines.push(format!("    let topic = \"{topic}\";"));
            lines.push(format!(
                "    let partition_key = event.{}.clone();",
                event.partition_key
            ));
            lines.push("    let payload = serde_json::to_vec(&event)?;".to_string());
            lines.push(
                "    let message = OutboxMessage::new(topic.to_string(), partition_key, payload);"
                    .to_string(),
            );
            lines.push("    store.insert(message).await?;".to_string());
            lines.push("    Ok(())".to_string());
            lines.push("}".to_string());
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

/// Generate a consumer handler stub file.
pub fn generate_consumer_handler(config: &EventConfig, event: &EventDef, handler: &str) -> String {
    let struct_name = event_to_pascal(&event.name);
    let topic = config.topic_name(event);

    let mut lines = Vec::new();
    lines.push("use k1s0_messaging::ConsumedMessage;".to_string());
    lines.push(String::new());
    lines.push("use crate::events::types::*;".to_string());
    lines.push(String::new());
    lines.push(format!(
        "/// Handle a consumed {struct_name} event from topic `{topic}`."
    ));
    lines.push(format!(
        "pub async fn {handler}(message: ConsumedMessage) -> Result<(), Box<dyn std::error::Error>> {{"
    ));
    lines.push(format!(
        "    let _event: {struct_name} = serde_json::from_slice(message.payload())?;"
    ));
    lines.push("    // TODO: implement handler logic".to_string());
    lines.push("    Ok(())".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    lines.join("\n")
}

/// Generate the `src/events/consumers/mod.rs` file.
pub fn generate_consumers_mod(config: &EventConfig) -> String {
    let mut lines = Vec::new();

    for event in &config.events {
        for consumer in &event.consumers {
            lines.push(format!("pub mod {};", consumer.handler));
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Generate the `config/schema-registry.yaml` file.
pub fn generate_schema_registry_config(config: &EventConfig) -> String {
    let mut lines = Vec::new();
    lines.push("# Schema Registry configuration".to_string());
    lines.push("# Auto-generated by k1s0-codegen event-codegen".to_string());
    lines.push(String::new());
    lines.push("schemas:".to_string());

    for event in &config.events {
        let topic = config.topic_name(event);
        let proto_path = super::proto_generator::proto_rel_path(config, event);
        let message_name = event_to_pascal(&event.name);

        lines.push(format!("  - subject: \"{topic}-value\""));
        lines.push("    type: PROTOBUF".to_string());
        lines.push(format!("    file: \"{proto_path}\""));
        lines.push(format!("    message: \"{message_name}\""));
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Convert a proto3 type to a Rust type.
fn proto_type_to_rust(proto_type: &str) -> &str {
    match proto_type {
        "double" => "f64",
        "float" => "f32",
        "int32" | "sint32" | "sfixed32" => "i32",
        "int64" | "sint64" | "sfixed64" => "i64",
        "uint32" | "fixed32" => "u32",
        "uint64" | "fixed64" => "u64",
        "bool" => "bool",
        "string" => "String",
        "bytes" => "Vec<u8>",
        _ => "String", // fallback
    }
}

/// Convert a dot-separated kebab-case event name to PascalCase.
fn event_to_pascal(name: &str) -> String {
    name.split('.')
        .map(|segment| naming::to_pascal(segment))
        .collect::<String>()
}

/// Convert a dot-separated kebab-case event name to snake_case.
fn event_to_snake(name: &str) -> String {
    name.split('.')
        .map(|segment| naming::to_snake(segment))
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::event_codegen::parser::parse_event_config_str;

    fn sample_config() -> EventConfig {
        parse_event_config_str(
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
"#,
        )
        .unwrap()
    }

    #[test]
    fn types_output() {
        let config = sample_config();
        let types = generate_types(&config);
        assert!(types.contains("pub struct ProjectTypeChanged {"));
        assert!(types.contains("pub project_type_id: String,"));
    }

    #[test]
    fn producer_output() {
        let config = sample_config();
        let producer = generate_producer(&config);
        assert!(producer.contains("pub async fn publish_project_type_changed("));
        assert!(producer.contains("pub async fn publish_project_type_changed_via_outbox("));
        assert!(producer.contains("k1s0.business.taskmanagement.project-type-changed.v1"));
    }

    #[test]
    fn consumer_handler_output() {
        let config = sample_config();
        let event = &config.events[0];
        let handler = generate_consumer_handler(&config, event, &event.consumers[0].handler);
        assert!(handler.contains("pub async fn on_taskmanagement_project_type_changed("));
        assert!(handler.contains("ProjectTypeChanged"));
    }

    #[test]
    fn events_mod_output() {
        let config = sample_config();
        let mod_rs = generate_events_mod(&config);
        assert!(mod_rs.contains("pub mod producer;"));
        assert!(mod_rs.contains("pub mod types;"));
        assert!(mod_rs.contains("pub mod consumers;"));
    }

    #[test]
    fn consumers_mod_output() {
        let config = sample_config();
        let mod_rs = generate_consumers_mod(&config);
        assert!(mod_rs.contains("pub mod on_taskmanagement_project_type_changed;"));
    }

    #[test]
    fn schema_registry_output() {
        let config = sample_config();
        let yaml = generate_schema_registry_config(&config);
        assert!(yaml.contains("subject: \"k1s0.business.taskmanagement.project-type-changed.v1-value\""));
        assert!(yaml.contains("type: PROTOBUF"));
        assert!(yaml.contains("message: \"ProjectTypeChanged\""));
    }

    #[test]
    fn proto_type_mapping() {
        assert_eq!(proto_type_to_rust("string"), "String");
        assert_eq!(proto_type_to_rust("int32"), "i32");
        assert_eq!(proto_type_to_rust("int64"), "i64");
        assert_eq!(proto_type_to_rust("bool"), "bool");
        assert_eq!(proto_type_to_rust("bytes"), "Vec<u8>");
        assert_eq!(proto_type_to_rust("double"), "f64");
        assert_eq!(proto_type_to_rust("float"), "f32");
        assert_eq!(proto_type_to_rust("uint32"), "u32");
        assert_eq!(proto_type_to_rust("uint64"), "u64");
    }
}
