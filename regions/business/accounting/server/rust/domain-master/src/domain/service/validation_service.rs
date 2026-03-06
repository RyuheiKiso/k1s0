use crate::domain::entity::master_category::MasterCategory;
use serde_json::Value;

pub struct ValidationService;

impl ValidationService {
    pub fn validate_item_attributes(
        category: &MasterCategory,
        attributes: &Option<Value>,
    ) -> anyhow::Result<()> {
        let Some(schema) = &category.validation_schema else {
            return Ok(());
        };

        let Some(attrs) = attributes else {
            return Ok(());
        };

        if let Some(required_fields) = schema.get("required").and_then(Value::as_array) {
            for field in required_fields {
                if let Some(field_name) = field.as_str() {
                    if attrs.get(field_name).is_none() {
                        anyhow::bail!(
                            "Validation error: required field '{}' is missing in attributes",
                            field_name
                        );
                    }
                }
            }
        }

        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, prop_schema) in properties {
                if let Some(value) = attrs.get(key) {
                    Self::validate_field(key, value, prop_schema)?;
                }
            }
        }

        Ok(())
    }

    fn validate_field(field_name: &str, value: &Value, schema: &Value) -> anyhow::Result<()> {
        if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
            let type_ok = match expected_type {
                "string" => value.is_string(),
                "number" | "integer" => value.is_number(),
                "boolean" => value.is_boolean(),
                "array" => value.is_array(),
                "object" => value.is_object(),
                _ => true,
            };
            if !type_ok {
                anyhow::bail!(
                    "Validation error: field '{}' expected type '{}', got '{}'",
                    field_name,
                    expected_type,
                    value_type_name(value)
                );
            }
        }

        if let Some(max_len) = schema.get("maxLength").and_then(Value::as_u64) {
            if let Some(s) = value.as_str() {
                if s.len() as u64 > max_len {
                    anyhow::bail!(
                        "Validation error: field '{}' exceeds maxLength {}",
                        field_name,
                        max_len
                    );
                }
            }
        }

        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
            if let Some(s) = value.as_str() {
                let re = regex::Regex::new(pattern).map_err(|e| {
                    anyhow::anyhow!("Invalid regex pattern for field '{}': {}", field_name, e)
                })?;
                if !re.is_match(s) {
                    anyhow::bail!(
                        "Validation error: field '{}' does not match pattern '{}'",
                        field_name,
                        pattern
                    );
                }
            }
        }

        Ok(())
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
