use std::collections::HashSet;

use crate::error::CodegenError;

use super::config::EventConfig;

/// Valid proto3 scalar types.
const VALID_PROTO3_TYPES: &[&str] = &[
    "double", "float", "int32", "int64", "uint32", "uint64", "sint32", "sint64", "fixed32",
    "fixed64", "sfixed32", "sfixed64", "bool", "string", "bytes",
];

/// Valid tier values.
const VALID_TIERS: &[&str] = &["system", "business", "service"];

/// Valid language values.
const VALID_LANGUAGES: &[&str] = &["rust", "go"];

/// Validate an `EventConfig` against all rules defined in the spec.
pub fn validate_event_config(config: &EventConfig) -> Result<(), CodegenError> {
    let mut errors: Vec<String> = Vec::new();

    // domain: non-empty, kebab-case
    if config.domain.is_empty() {
        errors.push("domain must not be empty".into());
    } else if !is_kebab_case(&config.domain) {
        errors.push(format!("domain '{}' must be kebab-case", config.domain));
    }

    // tier: system / business / service
    if !VALID_TIERS.contains(&config.tier.as_str()) {
        errors.push(format!(
            "tier '{}' must be one of: {}",
            config.tier,
            VALID_TIERS.join(", ")
        ));
    }

    // service_name: non-empty, kebab-case
    if config.service_name.is_empty() {
        errors.push("service_name must not be empty".into());
    } else if !is_kebab_case(&config.service_name) {
        errors.push(format!(
            "service_name '{}' must be kebab-case",
            config.service_name
        ));
    }

    // language: rust / go
    if !VALID_LANGUAGES.contains(&config.language.as_str()) {
        errors.push(format!(
            "language '{}' must be one of: {}",
            config.language,
            VALID_LANGUAGES.join(", ")
        ));
    }

    // events: at least one
    if config.events.is_empty() {
        errors.push("events must contain at least one event".into());
    }

    // event name uniqueness
    let mut event_names: HashSet<&str> = HashSet::new();
    for event in &config.events {
        if !event_names.insert(&event.name) {
            errors.push(format!("duplicate event name '{}'", event.name));
        }
    }

    // Per-event validation
    for (i, event) in config.events.iter().enumerate() {
        let prefix = format!("events[{}] ('{}')", i, event.name);

        // event.name: kebab-case + dot-separated
        if !is_event_name(&event.name) {
            errors.push(format!(
                "{prefix}: name must be kebab-case segments separated by dots"
            ));
        }

        // event.version: >= 1
        if event.version < 1 {
            errors.push(format!("{prefix}: version must be >= 1"));
        }

        // schema.fields: at least one
        if event.schema.fields.is_empty() {
            errors.push(format!("{prefix}: schema.fields must contain at least one field"));
        }

        // field number uniqueness and validity
        let mut field_numbers: HashSet<u32> = HashSet::new();
        for field in &event.schema.fields {
            if field.number < 1 {
                errors.push(format!(
                    "{prefix}: field '{}' number must be >= 1",
                    field.name
                ));
            }
            if !field_numbers.insert(field.number) {
                errors.push(format!(
                    "{prefix}: duplicate field number {} for field '{}'",
                    field.number, field.name
                ));
            }

            // field.type: proto3 valid type
            if !VALID_PROTO3_TYPES.contains(&field.field_type.as_str()) {
                errors.push(format!(
                    "{prefix}: field '{}' type '{}' is not a valid proto3 type",
                    field.name, field.field_type
                ));
            }
        }

        // partition_key: must reference a field in schema.fields
        let field_names: HashSet<&str> =
            event.schema.fields.iter().map(|f| f.name.as_str()).collect();
        if !field_names.contains(event.partition_key.as_str()) {
            errors.push(format!(
                "{prefix}: partition_key '{}' must reference a field in schema.fields",
                event.partition_key
            ));
        }

        // consumer.handler: snake_case
        for (j, consumer) in event.consumers.iter().enumerate() {
            if !is_snake_case(&consumer.handler) {
                errors.push(format!(
                    "{prefix}: consumers[{j}].handler '{}' must be snake_case",
                    consumer.handler
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(CodegenError::Validation(errors.join("; ")))
    }
}

/// Check if a string is kebab-case: lowercase ascii, digits, hyphens only,
/// no leading/trailing/consecutive hyphens.
fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() || s.starts_with('-') || s.ends_with('-') || s.contains("--") {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check if an event name is valid: dot-separated segments where each segment is kebab-case.
fn is_event_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.split('.').all(|seg| is_kebab_case(seg))
}

/// Check if a string is snake_case.
fn is_snake_case(s: &str) -> bool {
    if s.is_empty() || s.starts_with('_') || s.ends_with('_') || s.contains("__") {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_codegen::parser::parse_event_config_str;

    fn valid_yaml() -> &'static str {
        r#"
domain: accounting
tier: business
service_name: domain-master
language: rust
events:
  - name: master-item.created
    version: 1
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
"#
    }

    // 有効なイベント設定でバリデーションが成功することを確認する。
    #[test]
    fn valid_config_passes() {
        let config = parse_event_config_str(valid_yaml()).unwrap();
        assert!(validate_event_config(&config).is_ok());
    }

    // domain が空の場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn empty_domain_fails() {
        let yaml = valid_yaml().replace("domain: accounting", "domain: \"\"");
        let config = parse_event_config_str(&yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("domain"));
    }

    // 無効な tier 値を指定した場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn invalid_tier_fails() {
        let yaml = valid_yaml().replace("tier: business", "tier: invalid");
        let config = parse_event_config_str(&yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("tier"));
    }

    // 無効な language 値を指定した場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn invalid_language_fails() {
        let yaml = valid_yaml().replace("language: rust", "language: python");
        let config = parse_event_config_str(&yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("language"));
    }

    // スキーマフィールドに存在しない partition_key を指定した場合にエラーになることを確認する。
    #[test]
    fn invalid_partition_key_fails() {
        let yaml = valid_yaml().replace("partition_key: item_id", "partition_key: nonexistent");
        let config = parse_event_config_str(&yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("partition_key"));
    }

    // 無効な proto3 フィールド型を指定した場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn invalid_field_type_fails() {
        let yaml = valid_yaml().replace("type: string", "type: varchar");
        let config = parse_event_config_str(&yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("proto3 type"));
    }

    // 同じイベント名が重複した場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn duplicate_event_name_fails() {
        let yaml = r#"
domain: accounting
tier: business
service_name: domain-master
language: rust
events:
  - name: item.created
    version: 1
    partition_key: id
    schema:
      fields:
        - name: id
          type: string
          number: 1
  - name: item.created
    version: 2
    partition_key: id
    schema:
      fields:
        - name: id
          type: string
          number: 1
"#;
        let config = parse_event_config_str(yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    // フィールド番号が重複した場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn duplicate_field_number_fails() {
        let yaml = r#"
domain: accounting
tier: business
service_name: domain-master
language: rust
events:
  - name: item.created
    version: 1
    partition_key: id
    schema:
      fields:
        - name: id
          type: string
          number: 1
        - name: name
          type: string
          number: 1
"#;
        let config = parse_event_config_str(yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("duplicate field number"));
    }

    // ハンドラー名がスネークケースでない場合にバリデーションエラーが返されることを確認する。
    #[test]
    fn non_snake_case_handler_fails() {
        let yaml = valid_yaml().replace(
            "handler: on_accounting_master_item_created",
            "handler: onAccountingMasterItemCreated",
        );
        let config = parse_event_config_str(&yaml).unwrap();
        let err = validate_event_config(&config).unwrap_err();
        assert!(err.to_string().contains("snake_case"));
    }

    // ケバブケース判定関数が各種入力に対して正しく動作することを確認する。
    #[test]
    fn kebab_case_checks() {
        assert!(is_kebab_case("hello"));
        assert!(is_kebab_case("hello-world"));
        assert!(!is_kebab_case("Hello"));
        assert!(!is_kebab_case("-hello"));
        assert!(!is_kebab_case("hello-"));
        assert!(!is_kebab_case("hello--world"));
        assert!(!is_kebab_case(""));
    }

    // イベント名のバリデーション関数が各種入力に対して正しく動作することを確認する。
    #[test]
    fn event_name_checks() {
        assert!(is_event_name("item.created"));
        assert!(is_event_name("master-item.created"));
        assert!(is_event_name("simple"));
        assert!(!is_event_name(""));
        assert!(!is_event_name(".created"));
        assert!(!is_event_name("item."));
        assert!(!is_event_name("Item.Created"));
    }

    // スネークケース判定関数が各種入力に対して正しく動作することを確認する。
    #[test]
    fn snake_case_checks() {
        assert!(is_snake_case("hello"));
        assert!(is_snake_case("hello_world"));
        assert!(!is_snake_case("Hello"));
        assert!(!is_snake_case("_hello"));
        assert!(!is_snake_case("hello_"));
        assert!(!is_snake_case("hello__world"));
        assert!(!is_snake_case(""));
    }
}
