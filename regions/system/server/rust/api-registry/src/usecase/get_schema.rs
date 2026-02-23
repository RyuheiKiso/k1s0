use std::sync::Arc;

use crate::domain::entity::api_registration::{ApiSchema, ApiSchemaVersion};
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};

#[derive(Debug, Clone)]
pub struct GetSchemaOutput {
    pub schema: ApiSchema,
    pub latest_content: Option<ApiSchemaVersion>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetSchemaError {
    #[error("schema not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetSchemaUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
}

impl GetSchemaUseCase {
    pub fn new(
        schema_repo: Arc<dyn ApiSchemaRepository>,
        version_repo: Arc<dyn ApiSchemaVersionRepository>,
    ) -> Self {
        Self {
            schema_repo,
            version_repo,
        }
    }

    pub async fn execute(&self, name: &str) -> Result<GetSchemaOutput, GetSchemaError> {
        let schema = self
            .schema_repo
            .find_by_name(name)
            .await
            .map_err(|e| GetSchemaError::Internal(e.to_string()))?
            .ok_or_else(|| GetSchemaError::NotFound(name.to_string()))?;

        let latest_version = self
            .version_repo
            .find_latest_by_name(name)
            .await
            .map_err(|e| GetSchemaError::Internal(e.to_string()))?;

        Ok(GetSchemaOutput {
            schema,
            latest_content: latest_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::api_registration::{ApiSchema, ApiSchemaVersion, SchemaType};
    use crate::domain::repository::api_repository::{
        MockApiSchemaRepository, MockApiSchemaVersionRepository,
    };

    #[tokio::test]
    async fn success() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock.expect_find_latest_by_name().returning(|_| {
            Ok(Some(ApiSchemaVersion::new(
                "test-api".to_string(),
                1,
                SchemaType::OpenApi,
                "openapi: 3.0.3".to_string(),
                "user-001".to_string(),
            )))
        });

        let uc = GetSchemaUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("test-api").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.schema.name, "test-api");
        assert!(output.latest_content.is_some());
    }

    #[tokio::test]
    async fn not_found() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Ok(None));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = GetSchemaUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetSchemaError::NotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = GetSchemaUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("test-api").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetSchemaError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
