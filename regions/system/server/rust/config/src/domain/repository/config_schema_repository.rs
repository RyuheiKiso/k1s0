use async_trait::async_trait;

use crate::domain::entity::config_schema::ConfigSchema;

/// ConfigSchemaRepository は設定スキーマの永続化のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigSchemaRepository: Send + Sync {
    /// service_name で設定スキーマを取得する。
    async fn find_by_service_name(&self, service_name: &str) -> anyhow::Result<Option<ConfigSchema>>;

    /// namespace プレフィックスに一致するスキーマを取得する。
    /// 指定された namespace がスキーマの namespace_prefix で始まるものを返す。
    async fn find_by_namespace(&self, namespace: &str) -> anyhow::Result<Option<ConfigSchema>>;

    /// 全ての設定スキーマを一覧取得する。
    async fn list_all(&self) -> anyhow::Result<Vec<ConfigSchema>>;

    /// 設定スキーマを作成または更新する（upsert）。
    async fn upsert(&self, schema: &ConfigSchema) -> anyhow::Result<ConfigSchema>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_mock_find_by_service_name() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_find_by_service_name()
            .withf(|name| name == "auth-server")
            .returning(|_| {
                Ok(Some(ConfigSchema {
                    id: Uuid::new_v4(),
                    service_name: "auth-server".to_string(),
                    namespace_prefix: "system.auth".to_string(),
                    schema_json: serde_json::json!({"categories": []}),
                    updated_by: "admin@example.com".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }))
            });

        let result = mock.find_by_service_name("auth-server").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().service_name, "auth-server");
    }

    #[tokio::test]
    async fn test_mock_find_by_service_name_not_found() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_find_by_service_name()
            .returning(|_| Ok(None));

        let result = mock.find_by_service_name("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mock_upsert() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_upsert().returning(|schema| Ok(schema.clone()));

        let schema = ConfigSchema {
            id: Uuid::new_v4(),
            service_name: "auth-server".to_string(),
            namespace_prefix: "system.auth".to_string(),
            schema_json: serde_json::json!({"categories": []}),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = mock.upsert(&schema).await.unwrap();
        assert_eq!(result.service_name, "auth-server");
    }
}
