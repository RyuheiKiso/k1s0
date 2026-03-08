use super::config::{EventConfig, EventDef};
use crate::naming;

/// Generate a proto3 schema file for a single event.
pub fn generate_proto(config: &EventConfig, event: &EventDef) -> String {
    let message_name = event_to_pascal(&event.name);
    let package = format!(
        "k1s0.{}.{}.events.v{}",
        config.tier, config.domain, event.version
    );

    let mut lines = Vec::new();
    lines.push("syntax = \"proto3\";".to_string());
    lines.push(String::new());
    lines.push(format!("package {package};"));
    lines.push(String::new());

    if !event.description.is_empty() {
        lines.push(format!("// {}", event.description));
    }
    lines.push(format!("message {message_name} {{"));

    for field in &event.schema.fields {
        if !field.description.is_empty() {
            lines.push(format!("  // {}", field.description));
        }
        lines.push(format!(
            "  {} {} = {};",
            field.field_type, field.name, field.number
        ));
    }

    lines.push("}".to_string());
    lines.push(String::new());

    lines.join("\n")
}

/// Build the relative output path for a proto file.
///
/// Pattern: `proto/{domain}/events/v{ver}/{name_snake}.proto`
pub fn proto_rel_path(config: &EventConfig, event: &EventDef) -> String {
    let name_snake = naming::to_snake(&event.name.replace('.', "-"));
    format!(
        "proto/{}/events/v{}/{}.proto",
        config.domain, event.version, name_snake
    )
}

/// Convert a dot-separated kebab-case event name to PascalCase.
///
/// Example: "master-item.created" -> "MasterItemCreated"
fn event_to_pascal(name: &str) -> String {
    name.split('.')
        .map(|segment| naming::to_pascal(segment))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_codegen::parser::parse_event_config_str;

    fn sample_config() -> EventConfig {
        parse_event_config_str(
            r#"
domain: accounting
tier: business
service_name: domain-master
language: rust
events:
  - name: master-item.created
    version: 1
    description: "マスタアイテムが作成された時に発行されるイベント"
    partition_key: item_id
    schema:
      fields:
        - name: item_id
          type: string
          number: 1
          description: "アイテムID"
        - name: name
          type: string
          number: 2
"#,
        )
        .unwrap()
    }

    #[test]
    fn proto_output() {
        let config = sample_config();
        let proto = generate_proto(&config, &config.events[0]);
        assert!(proto.contains("syntax = \"proto3\";"));
        assert!(proto.contains("package k1s0.business.accounting.events.v1;"));
        assert!(proto.contains("message MasterItemCreated {"));
        assert!(proto.contains("string item_id = 1;"));
        assert!(proto.contains("string name = 2;"));
    }

    #[test]
    fn proto_path() {
        let config = sample_config();
        let path = proto_rel_path(&config, &config.events[0]);
        assert_eq!(path, "proto/accounting/events/v1/master_item_created.proto");
    }

    #[test]
    fn event_pascal_case() {
        assert_eq!(event_to_pascal("master-item.created"), "MasterItemCreated");
        assert_eq!(event_to_pascal("order.placed"), "OrderPlaced");
        assert_eq!(event_to_pascal("simple"), "Simple");
    }
}
