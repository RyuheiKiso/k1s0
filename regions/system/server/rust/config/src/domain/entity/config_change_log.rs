use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ConfigChangeLog は設定変更の監査ログを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigChangeLog {
    pub id: Uuid,
    pub config_entry_id: Uuid,
    pub namespace: String,
    pub key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub old_version: i32,
    pub new_version: i32,
    pub change_type: String,
    pub changed_by: String,
    pub changed_at: DateTime<Utc>,
}

/// CreateChangeLogRequest は変更ログ作成リクエストを表す。
#[derive(Debug, Clone)]
pub struct CreateChangeLogRequest {
    pub config_entry_id: Uuid,
    pub namespace: String,
    pub key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub old_version: i32,
    pub new_version: i32,
    pub change_type: String,
    pub changed_by: String,
}

impl ConfigChangeLog {
    /// 新しい ConfigChangeLog を作成する。
    pub fn new(req: CreateChangeLogRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            config_entry_id: req.config_entry_id,
            namespace: req.namespace,
            key: req.key,
            old_value: req.old_value,
            new_value: req.new_value,
            old_version: req.old_version,
            new_version: req.new_version,
            change_type: req.change_type,
            changed_by: req.changed_by,
            changed_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_change_log_creation() {
        let entry_id = Uuid::new_v4();
        let log = ConfigChangeLog::new(CreateChangeLogRequest {
            config_entry_id: entry_id,
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            old_value: Some(serde_json::json!(25)),
            new_value: Some(serde_json::json!(50)),
            old_version: 3,
            new_version: 4,
            change_type: "UPDATED".to_string(),
            changed_by: "operator@example.com".to_string(),
        });

        assert_eq!(log.config_entry_id, entry_id);
        assert_eq!(log.namespace, "system.auth.database");
        assert_eq!(log.key, "max_connections");
        assert_eq!(log.old_value, Some(serde_json::json!(25)));
        assert_eq!(log.new_value, Some(serde_json::json!(50)));
        assert_eq!(log.old_version, 3);
        assert_eq!(log.new_version, 4);
        assert_eq!(log.change_type, "UPDATED");
        assert_eq!(log.changed_by, "operator@example.com");
    }

    #[test]
    fn test_config_change_log_created() {
        let entry_id = Uuid::new_v4();
        let log = ConfigChangeLog::new(CreateChangeLogRequest {
            config_entry_id: entry_id,
            namespace: "system.auth.jwt".to_string(),
            key: "issuer".to_string(),
            old_value: None,
            new_value: Some(serde_json::json!(
                "https://auth.k1s0.internal.example.com/realms/k1s0"
            )),
            old_version: 0,
            new_version: 1,
            change_type: "CREATED".to_string(),
            changed_by: "admin@example.com".to_string(),
        });

        assert!(log.old_value.is_none());
        assert!(log.new_value.is_some());
        assert_eq!(log.change_type, "CREATED");
    }

    #[test]
    fn test_config_change_log_deleted() {
        let entry_id = Uuid::new_v4();
        let log = ConfigChangeLog::new(CreateChangeLogRequest {
            config_entry_id: entry_id,
            namespace: "system.auth.database".to_string(),
            key: "deprecated_setting".to_string(),
            old_value: Some(serde_json::json!("old_value")),
            new_value: None,
            old_version: 5,
            new_version: 6,
            change_type: "DELETED".to_string(),
            changed_by: "admin@example.com".to_string(),
        });

        assert!(log.old_value.is_some());
        assert!(log.new_value.is_none());
        assert_eq!(log.change_type, "DELETED");
    }

    #[test]
    fn test_config_change_log_serialization_roundtrip() {
        let entry_id = Uuid::new_v4();
        let log = ConfigChangeLog::new(CreateChangeLogRequest {
            config_entry_id: entry_id,
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            old_value: Some(serde_json::json!(25)),
            new_value: Some(serde_json::json!(50)),
            old_version: 3,
            new_version: 4,
            change_type: "UPDATED".to_string(),
            changed_by: "operator@example.com".to_string(),
        });

        let json = serde_json::to_string(&log).unwrap();
        let deserialized: ConfigChangeLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log, deserialized);
    }
}
