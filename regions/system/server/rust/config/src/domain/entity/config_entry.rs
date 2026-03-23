use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 設定エントリを表すドメインエンティティ。
/// サービスの設定値（名前空間とキーのペア）を管理し、バージョン管理と監査情報を保持する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ConfigEntry {
    /// エントリの一意識別子
    pub id: Uuid,
    /// 設定の名前空間（例: `system.auth.database`）
    pub namespace: String,
    /// 設定キー（例: `max_connections`）
    pub key: String,
    /// 設定値（JSON 形式で格納）。シリアライズ時は `value` キーで出力される
    #[serde(rename = "value", alias = "value_json")]
    pub value_json: serde_json::Value,
    /// 楽観的ロックに使用するバージョン番号
    pub version: i32,
    /// 設定の説明文
    pub description: String,
    /// エントリを作成したユーザー識別子
    pub created_by: String,
    /// エントリを最後に更新したユーザー識別子
    pub updated_by: String,
    /// エントリ作成日時（UTC）
    pub created_at: DateTime<Utc>,
    /// エントリ最終更新日時（UTC）
    pub updated_at: DateTime<Utc>,
}

/// ページネーション情報を表す。
/// 一覧取得結果とともに返され、クライアントが次ページの存在を判断するために使用する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Pagination {
    /// フィルタ条件に一致する全件数
    pub total_count: i64,
    /// 現在のページ番号（1 始まり）
    pub page: i32,
    /// 1 ページあたりの件数
    pub page_size: i32,
    /// 次ページが存在するかどうか
    pub has_next: bool,
}

/// 設定エントリ一覧とページネーション情報をまとめた結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ConfigListResult {
    /// 設定エントリの一覧
    pub entries: Vec<ConfigEntry>,
    /// ページネーション情報
    pub pagination: Pagination,
}

/// サービス向けに提供する設定エントリを表す。
/// `ConfigEntry` から内部管理フィールドを除いたサービス参照用の軽量な表現。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ServiceConfigEntry {
    /// 設定の名前空間（例: `system.auth.database`）
    pub namespace: String,
    /// 設定キー（例: `max_connections`）
    pub key: String,
    /// 設定値（JSON 形式）
    pub value: serde_json::Value,
    /// 現在のバージョン番号
    pub version: i32,
}

/// サービスが保持する設定エントリ一覧をまとめた結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ServiceConfigResult {
    /// 設定を所有するサービス名（例: `auth-server`）
    pub service_name: String,
    /// 該当サービスの設定エントリ一覧
    pub entries: Vec<ServiceConfigEntry>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            description: "DB max connections".to_string(),
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
            description: "SSL mode".to_string(),
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
                "issuer": "https://auth.example.com/realms/k1s0",
                "audience": "k1s0-api",
                "ttl_secs": 3600
            }),
            version: 2,
            description: "JWT settings".to_string(),
            created_by: "admin@example.com".to_string(),
            updated_by: "operator@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(entry.value_json.is_object());
        assert_eq!(entry.value_json["ttl_secs"], 3600);
    }

    #[test]
    fn test_config_entry_with_empty_description() {
        let entry = ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.config.internal".to_string(),
            key: "cache_ttl".to_string(),
            value_json: serde_json::json!(300),
            version: 1,
            description: String::new(),
            created_by: "system".to_string(),
            updated_by: "system".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(entry.description.is_empty());
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
                    version: 3,
                },
                ServiceConfigEntry {
                    namespace: "system.auth.database".to_string(),
                    key: "ssl_mode".to_string(),
                    value: serde_json::json!("require"),
                    version: 1,
                },
            ],
        };

        assert_eq!(result.service_name, "auth-server");
        assert_eq!(result.entries.len(), 2);
    }
}
