use std::sync::Arc;

use crate::domain::entity::config_schema::ConfigSchema;
use crate::domain::repository::config_schema_repository::ConfigSchemaRepository;

/// GetConfigSchemaError は設定スキーマ取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetConfigSchemaError {
    #[error("config schema not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetConfigSchemaUseCase は設定スキーマ取得ユースケース。
pub struct GetConfigSchemaUseCase {
    schema_repo: Arc<dyn ConfigSchemaRepository>,
}

impl GetConfigSchemaUseCase {
    pub fn new(schema_repo: Arc<dyn ConfigSchemaRepository>) -> Self {
        Self { schema_repo }
    }

    /// service_name とテナントIDで設定スキーマを取得する。
    // CRITICAL-RUST-001 監査対応: テナント分離のために tenant_id 引数を追加する。
    pub async fn execute(
        &self,
        service_name: &str,
        tenant_id: &str,
    ) -> Result<ConfigSchema, GetConfigSchemaError> {
        self.schema_repo
            .find_by_service_name(service_name, tenant_id)
            .await
            .map_err(|e| GetConfigSchemaError::Internal(e.to_string()))?
            .ok_or_else(|| GetConfigSchemaError::NotFound(service_name.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::config_schema_repository::MockConfigSchemaRepository;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_config_schema_success() {
        let mut mock = MockConfigSchemaRepository::new();
        let schema = ConfigSchema {
            id: Uuid::new_v4(),
            tenant_id: "test-tenant".to_string(),
            service_name: "auth-server".to_string(),
            namespace_prefix: "system.auth".to_string(),
            schema_json: serde_json::json!({"categories": []}),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let expected = schema.clone();

        mock.expect_find_by_service_name()
            .withf(|name, _tenant| name == "auth-server")
            .returning(move |_, _| Ok(Some(schema.clone())));

        let uc = GetConfigSchemaUseCase::new(Arc::new(mock));
        let result = uc.execute("auth-server", "test-tenant").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, expected.id);
    }

    #[tokio::test]
    async fn test_get_config_schema_not_found() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_find_by_service_name().returning(|_, _| Ok(None));

        let uc = GetConfigSchemaUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent", "test-tenant").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetConfigSchemaError::NotFound(name) => assert_eq!(name, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_config_schema_internal_error() {
        let mut mock = MockConfigSchemaRepository::new();
        mock.expect_find_by_service_name()
            .returning(|_, _| Err(anyhow::anyhow!("connection refused")));

        let uc = GetConfigSchemaUseCase::new(Arc::new(mock));
        let result = uc.execute("auth-server", "test-tenant").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetConfigSchemaError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
