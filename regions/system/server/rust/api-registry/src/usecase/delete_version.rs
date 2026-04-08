use std::sync::Arc;

use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use crate::infrastructure::kafka::{
    NoopSchemaEventPublisher, SchemaEventPublisher, SchemaUpdatedEvent,
};

#[derive(Debug, thiserror::Error)]
pub enum DeleteVersionError {
    #[error("schema not found: {0}")]
    SchemaNotFound(String),
    #[error("schema version not found: {name}@{version}")]
    VersionNotFound { name: String, version: u32 },
    #[error("cannot delete the only remaining version of schema: {0}")]
    CannotDeleteLatest(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteVersionUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
    publisher: Arc<dyn SchemaEventPublisher>,
}

impl DeleteVersionUseCase {
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

    // テナントスコープでバージョンを削除し、スキーマメタデータを更新する
    pub async fn execute(
        &self,
        tenant_id: &str,
        name: &str,
        version: u32,
        deleted_by: Option<String>,
    ) -> Result<(), DeleteVersionError> {
        // テナント分離のため tenant_id を渡してリポジトリを呼び出す
        let schema = self
            .schema_repo
            .find_by_name(tenant_id, name)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?
            .ok_or_else(|| DeleteVersionError::SchemaNotFound(name.to_string()))?;

        let count = self
            .version_repo
            .count_by_name(tenant_id, name)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?;

        if count <= 1 {
            return Err(DeleteVersionError::CannotDeleteLatest(schema.name.clone()));
        }

        let deleted = self
            .version_repo
            .delete(tenant_id, name, version)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteVersionError::VersionNotFound {
                name: name.to_string(),
                version,
            });
        }

        // 削除後メタデータ更新
        let remaining_count = self
            .version_repo
            .count_by_name(tenant_id, name)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?;
        let latest = self
            .version_repo
            .find_latest_by_name(tenant_id, name)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?
            .ok_or_else(|| DeleteVersionError::Internal("latest version not found".to_string()))?;

        let mut updated_schema = schema.clone();
        updated_schema.version_count = remaining_count as u32;
        updated_schema.latest_version = latest.version;
        updated_schema.updated_at = chrono::Utc::now();

        self.schema_repo
            .update(tenant_id, &updated_schema)
            .await
            .map_err(|e| DeleteVersionError::Internal(e.to_string()))?;

        // Kafka イベント発行
        let event = SchemaUpdatedEvent {
            event_type: "SCHEMA_VERSION_DELETED".to_string(),
            schema_name: name.to_string(),
            schema_type: schema.schema_type.to_string(),
            version,
            content_hash: None,
            breaking_changes: None,
            registered_by: None,
            deleted_by,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        if let Err(e) = self.publisher.publish_schema_updated(&event).await {
            tracing::warn!("Failed to publish schema version deleted event: {}", e);
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::api_registration::{ApiSchema, SchemaType};
    use crate::domain::repository::api_repository::{
        MockApiSchemaRepository, MockApiSchemaVersionRepository,
    };

    #[tokio::test]
    async fn success() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_, _| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });
        schema_mock.expect_update().returning(|_, _| Ok(()));

        let mut version_mock = MockApiSchemaVersionRepository::new();
        let count_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count_calls_clone = count_calls.clone();
        version_mock.expect_count_by_name().returning(move |_, _| {
            let call = count_calls_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if call == 0 {
                Ok(3)
            } else {
                Ok(2)
            }
        });
        version_mock.expect_delete().returning(|_, _, _| Ok(true));
        version_mock.expect_find_latest_by_name().returning(|_, name| {
            Ok(Some(
                crate::domain::entity::api_registration::ApiSchemaVersion::new(
                    name.to_string(),
                    2,
                    SchemaType::OpenApi,
                    "{}".to_string(),
                    "tester".to_string(),
                ),
            ))
        });

        let uc = DeleteVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("tenant-a", "test-api", 1, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found_schema() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_, _| Ok(None));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = DeleteVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("tenant-a", "nonexistent", 1, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteVersionError::SchemaNotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected SchemaNotFound error"),
        }
    }

    #[tokio::test]
    async fn cannot_delete_latest() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_, _| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock.expect_count_by_name().returning(|_, _| Ok(1));

        let uc = DeleteVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("tenant-a", "test-api", 1, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteVersionError::CannotDeleteLatest(name) => assert_eq!(name, "test-api"),
            _ => panic!("Expected CannotDeleteLatest error"),
        }
    }

    #[tokio::test]
    async fn version_not_found() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_, _| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock.expect_count_by_name().returning(|_, _| Ok(3));
        version_mock.expect_delete().returning(|_, _, _| Ok(false));

        let uc = DeleteVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("tenant-a", "test-api", 99, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteVersionError::VersionNotFound { name, version } => {
                assert_eq!(name, "test-api");
                assert_eq!(version, 99);
            }
            _ => panic!("Expected VersionNotFound error"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = DeleteVersionUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let result = uc.execute("tenant-a", "test-api", 1, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteVersionError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
