use std::sync::Arc;

use crate::domain::entity::api_registration::{ApiSchema, ApiSchemaVersion, SchemaType};
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use crate::infrastructure::kafka::{NoopSchemaEventPublisher, SchemaEventPublisher, SchemaUpdatedEvent};

#[derive(Debug, Clone)]
pub struct RegisterSchemaInput {
    pub name: String,
    pub description: String,
    pub schema_type: SchemaType,
    pub content: String,
    pub registered_by: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterSchemaError {
    #[error("schema already exists: {0}")]
    AlreadyExists(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RegisterSchemaUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
    publisher: Arc<dyn SchemaEventPublisher>,
}

impl RegisterSchemaUseCase {
    pub fn new(
        schema_repo: Arc<dyn ApiSchemaRepository>,
        version_repo: Arc<dyn ApiSchemaVersionRepository>,
    ) -> Self {
        Self {
            schema_repo,
            version_repo,
            publisher: Arc::new(NoopSchemaEventPublisher),
        }
    }

    pub fn with_publisher(
        schema_repo: Arc<dyn ApiSchemaRepository>,
        version_repo: Arc<dyn ApiSchemaVersionRepository>,
        publisher: Arc<dyn SchemaEventPublisher>,
    ) -> Self {
        Self {
            schema_repo,
            version_repo,
            publisher,
        }
    }

    pub async fn execute(
        &self,
        input: &RegisterSchemaInput,
    ) -> Result<ApiSchemaVersion, RegisterSchemaError> {
        let existing = self
            .schema_repo
            .find_by_name(&input.name)
            .await
            .map_err(|e| RegisterSchemaError::Internal(e.to_string()))?;

        if existing.is_some() {
            return Err(RegisterSchemaError::AlreadyExists(input.name.clone()));
        }

        let schema = ApiSchema::new(
            input.name.clone(),
            input.description.clone(),
            input.schema_type.clone(),
        );

        self.schema_repo
            .create(&schema)
            .await
            .map_err(|e| RegisterSchemaError::Internal(e.to_string()))?;

        let version = ApiSchemaVersion::new(
            input.name.clone(),
            1,
            input.schema_type.clone(),
            input.content.clone(),
            input.registered_by.clone(),
        );

        self.version_repo
            .create(&version)
            .await
            .map_err(|e| RegisterSchemaError::Internal(e.to_string()))?;

        // Kafka イベント発行
        let event = SchemaUpdatedEvent {
            event_type: "SCHEMA_REGISTERED".to_string(),
            schema_name: version.name.clone(),
            schema_type: version.schema_type.to_string(),
            version: version.version,
            content_hash: Some(version.content_hash.clone()),
            breaking_changes: Some(false),
            registered_by: Some(input.registered_by.clone()),
            deleted_by: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        if let Err(e) = self.publisher.publish_schema_updated(&event).await {
            tracing::warn!("Failed to publish schema registered event: {}", e);
        }

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::api_repository::{
        MockApiSchemaRepository, MockApiSchemaVersionRepository,
    };

    #[tokio::test]
    async fn success() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Ok(None));
        schema_mock.expect_create().returning(|_| Ok(()));

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock.expect_create().returning(|_| Ok(()));

        let uc = RegisterSchemaUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = RegisterSchemaInput {
            name: "test-api".to_string(),
            description: "Test API".to_string(),
            schema_type: SchemaType::OpenApi,
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.name, "test-api");
        assert_eq!(version.version, 1);
        assert!(version.content_hash.starts_with("sha256:"));
    }

    #[tokio::test]
    async fn already_exists() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = RegisterSchemaUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = RegisterSchemaInput {
            name: "test-api".to_string(),
            description: "Test API".to_string(),
            schema_type: SchemaType::OpenApi,
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RegisterSchemaError::AlreadyExists(name) => assert_eq!(name, "test-api"),
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = RegisterSchemaUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = RegisterSchemaInput {
            name: "test-api".to_string(),
            description: "Test API".to_string(),
            schema_type: SchemaType::OpenApi,
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RegisterSchemaError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
