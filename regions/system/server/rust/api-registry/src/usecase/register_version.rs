use std::sync::Arc;

use crate::domain::entity::api_registration::{ApiSchemaVersion, BreakingChange};
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use crate::infrastructure::kafka::{NoopSchemaEventPublisher, SchemaEventPublisher, SchemaUpdatedEvent};

#[derive(Debug, Clone)]
pub struct RegisterVersionInput {
    pub name: String,
    pub content: String,
    pub registered_by: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterVersionError {
    #[error("schema not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RegisterVersionUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
    publisher: Arc<dyn SchemaEventPublisher>,
}

impl RegisterVersionUseCase {
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
        input: &RegisterVersionInput,
    ) -> Result<ApiSchemaVersion, RegisterVersionError> {
        let mut schema = self
            .schema_repo
            .find_by_name(&input.name)
            .await
            .map_err(|e| RegisterVersionError::Internal(e.to_string()))?
            .ok_or_else(|| RegisterVersionError::NotFound(input.name.clone()))?;

        let new_version_num = schema.latest_version + 1;

        let _previous = self
            .version_repo
            .find_latest_by_name(&input.name)
            .await
            .map_err(|e| RegisterVersionError::Internal(e.to_string()))?;

        // In a full implementation, breaking change detection would compare _previous with new content.
        // For now, create a version with no breaking changes detected.
        let breaking_changes: Vec<BreakingChange> = Vec::new();
        let has_breaking = !breaking_changes.is_empty();

        let version = ApiSchemaVersion::new(
            input.name.clone(),
            new_version_num,
            schema.schema_type.clone(),
            input.content.clone(),
            input.registered_by.clone(),
        )
        .with_breaking_changes(has_breaking, breaking_changes);

        self.version_repo
            .create(&version)
            .await
            .map_err(|e| RegisterVersionError::Internal(e.to_string()))?;

        schema.latest_version = new_version_num;
        schema.version_count += 1;
        schema.updated_at = chrono::Utc::now();

        self.schema_repo
            .update(&schema)
            .await
            .map_err(|e| RegisterVersionError::Internal(e.to_string()))?;

        // Kafka イベント発行
        let event = SchemaUpdatedEvent {
            event_type: "SCHEMA_VERSION_REGISTERED".to_string(),
            schema_name: version.name.clone(),
            schema_type: version.schema_type.to_string(),
            version: version.version,
            content_hash: Some(version.content_hash.clone()),
            breaking_changes: Some(has_breaking),
            registered_by: Some(input.registered_by.clone()),
            deleted_by: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        if let Err(e) = self.publisher.publish_schema_updated(&event).await {
            tracing::warn!("Failed to publish schema version registered event: {}", e);
        }

        Ok(version)
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
        schema_mock.expect_update().returning(|_| Ok(()));

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
        version_mock.expect_create().returning(|_| Ok(()));

        let uc = RegisterVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = RegisterVersionInput {
            name: "test-api".to_string(),
            content: "openapi: 3.0.3\nversion: 2.0.0".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.version, 2);
        assert_eq!(version.name, "test-api");
    }

    #[tokio::test]
    async fn not_found() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Ok(None));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = RegisterVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = RegisterVersionInput {
            name: "nonexistent".to_string(),
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RegisterVersionError::NotFound(name) => assert_eq!(name, "nonexistent"),
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

        let uc = RegisterVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = RegisterVersionInput {
            name: "test-api".to_string(),
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RegisterVersionError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
