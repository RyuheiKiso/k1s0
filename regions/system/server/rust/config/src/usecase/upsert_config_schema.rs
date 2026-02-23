use std::sync::Arc;

use crate::domain::entity::config_schema::ConfigSchema;
use crate::domain::repository::config_schema_repository::ConfigSchemaRepository;

/// UpsertConfigSchemaError は設定スキーマ作成・更新に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum UpsertConfigSchemaError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// UpsertConfigSchemaInput は設定スキーマ作成・更新の入力を表す。
#[derive(Debug, Clone)]
pub struct UpsertConfigSchemaInput {
    pub service_name: String,
    pub namespace_prefix: String,
    pub schema_json: serde_json::Value,
    pub updated_by: String,
}

/// UpsertConfigSchemaUseCase は設定スキーマ作成・更新ユースケース。
pub struct UpsertConfigSchemaUseCase {
    schema_repo: Arc<dyn ConfigSchemaRepository>,
}

impl UpsertConfigSchemaUseCase {
    pub fn new(schema_repo: Arc<dyn ConfigSchemaRepository>) -> Self {
        Self { schema_repo }
    }

    /// 設定スキーマを作成または更新する。
    pub async fn execute(
        &self,
        input: &UpsertConfigSchemaInput,
    ) -> Result<ConfigSchema, UpsertConfigSchemaError> {
        let schema = ConfigSchema {
            id: uuid::Uuid::new_v4(),
            service_name: input.service_name.clone(),
            namespace_prefix: input.namespace_prefix.clone(),
            schema_json: input.schema_json.clone(),
            updated_by: input.updated_by.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.schema_repo
            .upsert(&schema)
            .await
            .map_err(|e| UpsertConfigSchemaError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::config_schema_repository::MockConfigSchemaRepository;

    #[tokio::test]
    async fn test_upsert_config_schema_success() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_upsert().returning(|schema| Ok(schema.clone()));

        let uc = UpsertConfigSchemaUseCase::new(Arc::new(mock));
        let input = UpsertConfigSchemaInput {
            service_name: "auth-server".to_string(),
            namespace_prefix: "system.auth".to_string(),
            schema_json: serde_json::json!({"categories": []}),
            updated_by: "admin@example.com".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().service_name, "auth-server");
    }

    #[tokio::test]
    async fn test_upsert_config_schema_internal_error() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_upsert()
            .returning(|_| Err(anyhow::anyhow!("database error")));

        let uc = UpsertConfigSchemaUseCase::new(Arc::new(mock));
        let input = UpsertConfigSchemaInput {
            service_name: "auth-server".to_string(),
            namespace_prefix: "system.auth".to_string(),
            schema_json: serde_json::json!({"categories": []}),
            updated_by: "admin@example.com".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpsertConfigSchemaError::Internal(msg) => assert!(msg.contains("database error")),
        }
    }
}
