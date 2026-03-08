use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::domain::entity::config_schema::ConfigSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFieldType {
    String,
    Integer,
    Float,
    Boolean,
    Enum,
    Object,
    Array,
}

impl ConfigFieldType {
    pub fn from_legacy_number(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::String),
            2 => Some(Self::Integer),
            3 => Some(Self::Float),
            4 => Some(Self::Boolean),
            5 => Some(Self::Enum),
            6 => Some(Self::Object),
            7 => Some(Self::Array),
            _ => None,
        }
    }

    pub fn to_legacy_number(self) -> i32 {
        match self {
            Self::String => 1,
            Self::Integer => 2,
            Self::Float => 3,
            Self::Boolean => 4,
            Self::Enum => 5,
            Self::Object => 6,
            Self::Array => 7,
        }
    }

    pub fn from_schema_value(value: &Value) -> Option<Self> {
        value
            .as_str()
            .and_then(|v| match v {
                "string" => Some(Self::String),
                "integer" => Some(Self::Integer),
                "float" | "number" => Some(Self::Float),
                "boolean" => Some(Self::Boolean),
                "enum" => Some(Self::Enum),
                "object" => Some(Self::Object),
                "array" => Some(Self::Array),
                _ => None,
            })
            .or_else(|| value.as_i64().and_then(Self::from_legacy_number))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConfigFieldSchemaDto {
    pub key: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(rename = "type", deserialize_with = "deserialize_field_type")]
    pub field_type: ConfigFieldType,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub min: i64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub max: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub pattern: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    #[serde(default, alias = "default_value", skip_serializing_if = "Value::is_null")]
    pub default: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConfigCategorySchemaDto {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub icon: String,
    #[serde(default)]
    pub namespaces: Vec<String>,
    #[serde(default)]
    pub fields: Vec<ConfigFieldSchemaDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConfigEditorSchemaDto {
    pub service: String,
    pub namespace_prefix: String,
    pub categories: Vec<ConfigCategorySchemaDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpsertConfigSchemaRequestDto {
    pub namespace_prefix: String,
    pub categories: Vec<ConfigCategorySchemaDto>,
}

impl UpsertConfigSchemaRequestDto {
    pub fn into_schema_json(self) -> Value {
        let categories = self
            .categories
            .into_iter()
            .map(|category| {
                let fields = category
                    .fields
                    .into_iter()
                    .map(|field| {
                        serde_json::json!({
                            "key": field.key,
                            "label": field.label,
                            "description": field.description,
                            "type": field.field_type,
                            "min": field.min,
                            "max": field.max,
                            "options": field.options,
                            "pattern": field.pattern,
                            "unit": field.unit,
                            "default": field.default,
                        })
                    })
                    .collect::<Vec<_>>();

                serde_json::json!({
                    "id": category.id,
                    "label": category.label,
                    "icon": category.icon,
                    "namespaces": category.namespaces,
                    "fields": fields,
                })
            })
            .collect::<Vec<_>>();

        serde_json::json!({ "categories": categories })
    }
}

impl TryFrom<&ConfigSchema> for ConfigEditorSchemaDto {
    type Error = anyhow::Error;

    fn try_from(schema: &ConfigSchema) -> Result<Self, Self::Error> {
        let categories = schema
            .schema_json
            .get("categories")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("schema_json.categories must be an array"))?
            .iter()
            .map(ConfigCategorySchemaDto::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            service: schema.service_name.clone(),
            namespace_prefix: schema.namespace_prefix.clone(),
            categories,
            updated_at: Some(schema.updated_at),
        })
    }
}

impl TryFrom<&Value> for ConfigCategorySchemaDto {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let fields = value
            .get("fields")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(ConfigFieldSchemaDto::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            id: required_str(value, "id")?.to_string(),
            label: required_str(value, "label")?.to_string(),
            icon: optional_str(value, "icon").unwrap_or_default().to_string(),
            namespaces: value
                .get("namespaces")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            fields,
        })
    }
}

impl TryFrom<&Value> for ConfigFieldSchemaDto {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let field_type = value
            .get("type")
            .and_then(ConfigFieldType::from_schema_value)
            .ok_or_else(|| anyhow::anyhow!("field type is invalid"))?;

        Ok(Self {
            key: required_str(value, "key")?.to_string(),
            label: required_str(value, "label")?.to_string(),
            description: optional_str(value, "description").unwrap_or_default().to_string(),
            field_type,
            min: value.get("min").and_then(Value::as_i64).unwrap_or_default(),
            max: value.get("max").and_then(Value::as_i64).unwrap_or_default(),
            options: value
                .get("options")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            pattern: optional_str(value, "pattern").unwrap_or_default().to_string(),
            unit: optional_str(value, "unit").unwrap_or_default().to_string(),
            default: value
                .get("default")
                .cloned()
                .or_else(|| value.get("default_value").cloned())
                .unwrap_or(Value::Null),
        })
    }
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, anyhow::Error> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{} is required", key))
}

fn optional_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}

fn deserialize_field_type<'de, D>(deserializer: D) -> Result<ConfigFieldType, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    ConfigFieldType::from_schema_value(&value)
        .ok_or_else(|| serde::de::Error::custom("invalid config field type"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn domain_schema_maps_to_rest_dto_with_legacy_shape() {
        let schema = ConfigSchema {
            id: Uuid::new_v4(),
            service_name: "order-service".to_string(),
            namespace_prefix: "service.order".to_string(),
            schema_json: serde_json::json!({
                "categories": [{
                    "id": "database",
                    "label": "Database",
                    "icon": "storage",
                    "namespaces": ["service.order.database"],
                    "fields": [{
                        "key": "timeout",
                        "label": "Timeout",
                        "type": 2,
                        "default_value": 30
                    }]
                }]
            }),
            updated_by: "tester".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = ConfigEditorSchemaDto::try_from(&schema).unwrap();
        assert_eq!(dto.service, "order-service");
        assert_eq!(dto.categories[0].fields[0].field_type, ConfigFieldType::Integer);
        assert_eq!(dto.categories[0].fields[0].default, serde_json::json!(30));
    }

    #[test]
    fn upsert_request_normalizes_schema_json() {
        let req = UpsertConfigSchemaRequestDto {
            namespace_prefix: "service.order".to_string(),
            categories: vec![ConfigCategorySchemaDto {
                id: "database".to_string(),
                label: "Database".to_string(),
                icon: String::new(),
                namespaces: vec!["service.order.database".to_string()],
                fields: vec![ConfigFieldSchemaDto {
                    key: "enabled".to_string(),
                    label: "Enabled".to_string(),
                    description: String::new(),
                    field_type: ConfigFieldType::Boolean,
                    min: 0,
                    max: 0,
                    options: vec![],
                    pattern: String::new(),
                    unit: String::new(),
                    default: serde_json::json!(true),
                }],
            }],
        };

        let json = req.into_schema_json();
        assert_eq!(json["categories"][0]["fields"][0]["type"], "boolean");
        assert_eq!(json["categories"][0]["fields"][0]["default"], true);
        assert!(json["categories"][0]["fields"][0].get("default_value").is_none());
    }
}
