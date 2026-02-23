use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// ConfigSchema は設定エディタスキーマを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ConfigSchema {
    pub id: Uuid,
    pub service_name: String,
    pub namespace_prefix: String,
    pub schema_json: Value,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_schema_creation() {
        let schema = ConfigSchema {
            id: Uuid::new_v4(),
            service_name: "auth-server".to_string(),
            namespace_prefix: "system.auth".to_string(),
            schema_json: serde_json::json!({
                "categories": []
            }),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(schema.service_name, "auth-server");
        assert_eq!(schema.namespace_prefix, "system.auth");
    }

    #[test]
    fn test_config_schema_serialization_roundtrip() {
        let schema = ConfigSchema {
            id: Uuid::new_v4(),
            service_name: "config-server".to_string(),
            namespace_prefix: "system.config".to_string(),
            schema_json: serde_json::json!({
                "categories": [{
                    "id": "database",
                    "label": "Database",
                    "fields": []
                }]
            }),
            updated_by: "system".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&schema).unwrap();
        let deserialized: ConfigSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(schema, deserialized);
    }
}
