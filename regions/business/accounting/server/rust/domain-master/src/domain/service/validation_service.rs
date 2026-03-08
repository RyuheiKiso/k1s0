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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::master_category::MasterCategory;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_category(schema: Option<serde_json::Value>) -> MasterCategory {
        MasterCategory {
            id: Uuid::new_v4(),
            code: "TEST".to_string(),
            display_name: "Test".to_string(),
            description: None,
            validation_schema: schema,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_validate_with_valid_attributes() {
        let category = make_category(Some(serde_json::json!({
            "required": ["symbol"],
            "properties": {
                "symbol": { "type": "string", "maxLength": 3 }
            }
        })));
        let attrs = Some(serde_json::json!({"symbol": "JPY"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_missing_required_field() {
        let category = make_category(Some(serde_json::json!({
            "required": ["symbol"],
            "properties": {
                "symbol": { "type": "string" }
            }
        })));
        let attrs = Some(serde_json::json!({"name": "Yen"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("required field 'symbol' is missing"));
    }

    #[test]
    fn test_validate_with_wrong_type() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "decimals": { "type": "number" }
            }
        })));
        let attrs = Some(serde_json::json!({"decimals": "not_a_number"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected type 'number'"));
        assert!(err_msg.contains("decimals"));
    }

    #[test]
    fn test_validate_with_max_length_violation() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "symbol": { "type": "string", "maxLength": 3 }
            }
        })));
        let attrs = Some(serde_json::json!({"symbol": "TOOLONG"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exceeds maxLength 3"));
    }

    #[test]
    fn test_validate_with_pattern_violation() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "code": { "type": "string", "pattern": "^[A-Z]{3}$" }
            }
        })));
        let attrs = Some(serde_json::json!({"code": "abcd"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("does not match pattern"));
    }

    #[test]
    fn test_validate_with_pattern_success() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "code": { "type": "string", "pattern": "^[A-Z]{3}$" }
            }
        })));
        let attrs = Some(serde_json::json!({"code": "USD"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_empty_schema_passes() {
        let category = make_category(Some(serde_json::json!({})));
        let attrs = Some(serde_json::json!({"anything": "goes"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_no_schema_passes() {
        let category = make_category(None);
        let attrs = Some(serde_json::json!({"anything": "goes"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_no_attributes_passes() {
        let category = make_category(Some(serde_json::json!({
            "required": ["symbol"]
        })));
        let result = ValidationService::validate_item_attributes(&category, &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_boolean_type() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "active": { "type": "boolean" }
            }
        })));
        let attrs_ok = Some(serde_json::json!({"active": true}));
        assert!(ValidationService::validate_item_attributes(&category, &attrs_ok).is_ok());

        let attrs_bad = Some(serde_json::json!({"active": "yes"}));
        assert!(ValidationService::validate_item_attributes(&category, &attrs_bad).is_err());
    }

    #[test]
    fn test_validate_object_type() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "metadata": { "type": "object" }
            }
        })));
        let attrs_ok = Some(serde_json::json!({"metadata": {"key": "val"}}));
        assert!(ValidationService::validate_item_attributes(&category, &attrs_ok).is_ok());

        let attrs_bad = Some(serde_json::json!({"metadata": "not_object"}));
        assert!(ValidationService::validate_item_attributes(&category, &attrs_bad).is_err());
    }

    #[test]
    fn test_validate_array_type() {
        let category = make_category(Some(serde_json::json!({
            "properties": {
                "tags": { "type": "array" }
            }
        })));
        let attrs_ok = Some(serde_json::json!({"tags": ["a", "b"]}));
        assert!(ValidationService::validate_item_attributes(&category, &attrs_ok).is_ok());

        let attrs_bad = Some(serde_json::json!({"tags": "not_array"}));
        assert!(ValidationService::validate_item_attributes(&category, &attrs_bad).is_err());
    }

    #[test]
    fn test_validate_multiple_required_fields() {
        let category = make_category(Some(serde_json::json!({
            "required": ["symbol", "name"],
            "properties": {
                "symbol": { "type": "string" },
                "name": { "type": "string" }
            }
        })));
        // Missing "name"
        let attrs = Some(serde_json::json!({"symbol": "USD"}));
        let result = ValidationService::validate_item_attributes(&category, &attrs);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }
}
