use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// 設定エディタスキーマを表すドメインエンティティ。
/// サービスごとに設定値の構造（カテゴリ・フィールド定義）を JSON スキーマとして保持し、
/// UI がフォームを動的に生成するために参照する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct ConfigSchema {
    /// スキーマの一意識別子
    pub id: Uuid,
    /// スキーマを所有するサービス名（例: `auth-server`）
    pub service_name: String,
    /// このスキーマが対象とする名前空間のプレフィックス（例: `system.auth`）
    pub namespace_prefix: String,
    /// フィールド定義を含む JSON スキーマ本体
    pub schema_json: Value,
    /// スキーマを最後に更新したユーザー識別子
    pub updated_by: String,
    /// スキーマ作成日時（UTC）
    pub created_at: DateTime<Utc>,
    /// スキーマ最終更新日時（UTC）
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
