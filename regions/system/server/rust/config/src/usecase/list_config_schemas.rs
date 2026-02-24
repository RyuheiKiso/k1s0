use std::sync::Arc;

use crate::domain::entity::config_schema::ConfigSchema;
use crate::domain::repository::config_schema_repository::ConfigSchemaRepository;

/// ListConfigSchemasError は設定スキーマ一覧取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum ListConfigSchemasError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// ListConfigSchemasUseCase は全設定スキーマ一覧取得ユースケース。
pub struct ListConfigSchemasUseCase {
    schema_repo: Arc<dyn ConfigSchemaRepository>,
}

impl ListConfigSchemasUseCase {
    pub fn new(schema_repo: Arc<dyn ConfigSchemaRepository>) -> Self {
        Self { schema_repo }
    }

    /// 全ての設定スキーマを一覧取得する。
    pub async fn execute(&self) -> Result<Vec<ConfigSchema>, ListConfigSchemasError> {
        self.schema_repo
            .list_all()
            .await
            .map_err(|e| ListConfigSchemasError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::config_schema_repository::MockConfigSchemaRepository;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_config_schemas_success() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_list_all().returning(|| {
            Ok(vec![
                ConfigSchema {
                    id: Uuid::new_v4(),
                    service_name: "auth-server".to_string(),
                    namespace_prefix: "system.auth".to_string(),
                    schema_json: serde_json::json!({"categories": []}),
                    updated_by: "admin@example.com".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                ConfigSchema {
                    id: Uuid::new_v4(),
                    service_name: "config-server".to_string(),
                    namespace_prefix: "system.config".to_string(),
                    schema_json: serde_json::json!({"categories": []}),
                    updated_by: "admin@example.com".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ])
        });

        let uc = ListConfigSchemasUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_config_schemas_empty() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_list_all().returning(|| Ok(vec![]));

        let uc = ListConfigSchemasUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_config_schemas_internal_error() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_list_all()
            .returning(|| Err(anyhow::anyhow!("connection refused")));

        let uc = ListConfigSchemasUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListConfigSchemasError::Internal(msg) => assert!(msg.contains("connection refused")),
        }
    }
}
