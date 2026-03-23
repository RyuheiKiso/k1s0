use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 設定変更の監査ログを表すドメインエンティティ。
/// 設定エントリに対する CREATED / UPDATED / DELETED 操作の前後の値と操作者を記録し、
/// 変更履歴の追跡と監査を可能にする。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigChangeLog {
    /// ログの一意識別子
    pub id: Uuid,
    /// 変更対象の設定エントリ ID
    pub config_entry_id: Uuid,
    /// 変更対象の名前空間
    pub namespace: String,
    /// 変更対象の設定キー
    pub key: String,
    /// 変更前の設定値（新規作成時は `None`）
    pub old_value: Option<serde_json::Value>,
    /// 変更後の設定値（削除時は `None`）
    pub new_value: Option<serde_json::Value>,
    /// 変更前のバージョン番号
    pub old_version: i32,
    /// 変更後のバージョン番号
    pub new_version: i32,
    /// 変更種別（`CREATED` / `UPDATED` / `DELETED`）
    pub change_type: String,
    /// 変更を実施したユーザー識別子
    pub changed_by: String,
    /// 変更操作に紐付くトレース ID（オプション）
    pub trace_id: Option<String>,
    /// 変更が記録された日時（UTC）
    pub changed_at: DateTime<Utc>,
}

/// 変更ログ作成リクエストを表す値オブジェクト。
/// `ConfigChangeLog::new` に渡すことでログエントリを生成する。
#[derive(Debug, Clone)]
pub struct CreateChangeLogRequest {
    /// 変更対象の設定エントリ ID
    pub config_entry_id: Uuid,
    /// 変更対象の名前空間
    pub namespace: String,
    /// 変更対象の設定キー
    pub key: String,
    /// 変更前の設定値（新規作成時は `None`）
    pub old_value: Option<serde_json::Value>,
    /// 変更後の設定値（削除時は `None`）
    pub new_value: Option<serde_json::Value>,
    /// 変更前のバージョン番号
    pub old_version: i32,
    /// 変更後のバージョン番号
    pub new_version: i32,
    /// 変更種別（`CREATED` / `UPDATED` / `DELETED`）
    pub change_type: String,
    /// 変更を実施したユーザー識別子
    pub changed_by: String,
    /// 変更操作に紐付くトレース ID（オプション）
    pub trace_id: Option<String>,
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
            trace_id: req.trace_id,
            changed_at: Utc::now(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            trace_id: Some("trace-001".to_string()),
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
        assert_eq!(log.trace_id.as_deref(), Some("trace-001"));
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
            trace_id: None,
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
            trace_id: None,
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
            trace_id: None,
        });

        let json = serde_json::to_string(&log).unwrap();
        let deserialized: ConfigChangeLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log, deserialized);
    }
}
