use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ConfigEntry は設定値を表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ConfigEntry {
    pub id: Uuid,
    pub namespace: String,
    pub key: String,
    pub value_json: serde_json::Value,
    pub version: i32,
    pub description: Option<String>,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pagination はページネーションパラメータを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Pagination {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

/// ConfigListResult は設定値一覧とページネーション結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ConfigListResult {
    pub entries: Vec<ConfigEntry>,
    pub pagination: Pagination,
}

/// ServiceConfigEntry はサービス向け設定の簡易表現。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ServiceConfigEntry {
    pub namespace: String,
    pub key: String,
    pub value: serde_json::Value,
}

/// ServiceConfigResult はサービス向け設定一括取得の結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ServiceConfigResult {
    pub service_name: String,
    pub entries: Vec<ServiceConfigEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_entry_creation() {
        let entry = ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: Some("認証サーバーの DB 最大接続数".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(entry.namespace, "system.auth.database");
        assert_eq!(entry.key, "max_connections");
        assert_eq!(entry.value_json, serde_json::json!(25));
        assert_eq!(entry.version, 3);
    }

    #[test]
    fn test_config_entry_serialization_roundtrip() {
        let entry = ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "ssl_mode".to_string(),
            value_json: serde_json::json!("require"),
            version: 1,
            description: Some("SSL 接続モード".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: ConfigEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_config_entry_with_object_value() {
        let entry = ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.jwt".to_string(),
            key: "settings".to_string(),
            value_json: serde_json::json!({
                "issuer": "https://auth.k1s0.internal.example.com/realms/k1s0",
                "audience": "k1s0-api",
                "ttl_secs": 3600
            }),
            version: 2,
            description: Some("JWT 設定".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "operator@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(entry.value_json.is_object());
        assert_eq!(entry.value_json["ttl_secs"], 3600);
    }

    #[test]
    fn test_config_entry_with_none_description() {
        let entry = ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.config.internal".to_string(),
            key: "cache_ttl".to_string(),
            value_json: serde_json::json!(300),
            version: 1,
            description: None,
            created_by: "system".to_string(),
            updated_by: "system".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(entry.description.is_none());
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination {
            total_count: 150,
            page: 1,
            page_size: 20,
            has_next: true,
        };

        assert_eq!(pagination.total_count, 150);
        assert!(pagination.has_next);
    }

    #[test]
    fn test_config_list_result() {
        let result = ConfigListResult {
            entries: vec![],
            pagination: Pagination {
                total_count: 0,
                page: 1,
                page_size: 20,
                has_next: false,
            },
        };

        assert!(result.entries.is_empty());
        assert_eq!(result.pagination.total_count, 0);
        assert!(!result.pagination.has_next);
    }

    #[test]
    fn test_service_config_result() {
        let result = ServiceConfigResult {
            service_name: "auth-server".to_string(),
            entries: vec![
                ServiceConfigEntry {
                    namespace: "system.auth.database".to_string(),
                    key: "max_connections".to_string(),
                    value: serde_json::json!(25),
                },
                ServiceConfigEntry {
                    namespace: "system.auth.database".to_string(),
                    key: "ssl_mode".to_string(),
                    value: serde_json::json!("require"),
                },
            ],
        };

        assert_eq!(result.service_name, "auth-server");
        assert_eq!(result.entries.len(), 2);
    }
}
