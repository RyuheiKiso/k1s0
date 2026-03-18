use std::sync::Arc;

use crate::domain::repository::{FileMetadataRepository, FileStorageRepository};
use crate::infrastructure::kafka_producer::FileEventPublisher;

#[derive(Debug, Clone)]
pub struct DeleteFileInput {
    pub file_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteFileOutput {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteFileError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteFileUseCase {
    metadata_repo: Arc<dyn FileMetadataRepository>,
    storage_repo: Arc<dyn FileStorageRepository>,
    event_publisher: Arc<dyn FileEventPublisher>,
}

impl DeleteFileUseCase {
    pub fn new(
        metadata_repo: Arc<dyn FileMetadataRepository>,
        storage_repo: Arc<dyn FileStorageRepository>,
        event_publisher: Arc<dyn FileEventPublisher>,
    ) -> Self {
        Self {
            metadata_repo,
            storage_repo,
            event_publisher,
        }
    }

    pub async fn execute(
        &self,
        input: &DeleteFileInput,
    ) -> Result<DeleteFileOutput, DeleteFileError> {
        let file = self
            .metadata_repo
            .find_by_id(&input.file_id)
            .await
            .map_err(|e| DeleteFileError::Internal(e.to_string()))?
            .ok_or_else(|| DeleteFileError::NotFound(input.file_id.clone()))?;

        self.storage_repo
            .delete_object(&file.storage_key)
            .await
            .map_err(|e| DeleteFileError::Internal(e.to_string()))?;

        self.metadata_repo
            .delete(&input.file_id)
            .await
            .map_err(|e| DeleteFileError::Internal(e.to_string()))?;

        let payload = serde_json::json!({
            "file_id": file.id,
            "tenant_id": file.tenant_id,
            "uploaded_by": file.uploaded_by,
            "storage_key": file.storage_key,
            "deleted_at": chrono::Utc::now().to_rfc3339(),
        });
        if let Err(e) = self.event_publisher.publish("file.deleted", &payload).await {
            tracing::warn!(error = %e, "failed to publish file.deleted event");
        }

        Ok(DeleteFileOutput {
            success: true,
            message: format!("file {} deleted", input.file_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::file::FileMetadata;
    use crate::domain::repository::file_repository::{
        MockFileMetadataRepository, MockFileStorageRepository,
    };
    use crate::infrastructure::kafka_producer::MockFileEventPublisher;
    use std::collections::HashMap;

    fn sample_file() -> FileMetadata {
        FileMetadata::new(
            "file_001".to_string(),
            "report.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            HashMap::new(),
            "tenant-abc/report.pdf".to_string(),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let mut storage_mock = MockFileStorageRepository::new();

        let file = sample_file();
        let return_file = file.clone();

        metadata_mock
            .expect_find_by_id()
            .withf(|id| id == "file_001")
            .returning(move |_| Ok(Some(return_file.clone())));
        storage_mock.expect_delete_object().returning(|_| Ok(()));
        metadata_mock
            .expect_delete()
            .withf(|id| id == "file_001")
            .returning(|_| Ok(true));
        let mut event_publisher = MockFileEventPublisher::new();
        event_publisher.expect_publish().returning(|_, _| Ok(()));

        let uc = DeleteFileUseCase::new(
            Arc::new(metadata_mock),
            Arc::new(storage_mock),
            Arc::new(event_publisher),
        );
        let input = DeleteFileInput {
            file_id: "file_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.message.contains("file_001"));
    }

    #[tokio::test]
    async fn not_found() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        metadata_mock.expect_find_by_id().returning(|_| Ok(None));
        let event_publisher = MockFileEventPublisher::new();

        let uc = DeleteFileUseCase::new(
            Arc::new(metadata_mock),
            Arc::new(storage_mock),
            Arc::new(event_publisher),
        );
        let input = DeleteFileInput {
            file_id: "missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteFileError::NotFound(id) => assert_eq!(id, "missing"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_storage_delete() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let mut storage_mock = MockFileStorageRepository::new();

        let file = sample_file();
        let return_file = file.clone();

        metadata_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(return_file.clone())));
        storage_mock
            .expect_delete_object()
            .returning(|_| Err(anyhow::anyhow!("storage error")));
        let event_publisher = MockFileEventPublisher::new();

        let uc = DeleteFileUseCase::new(
            Arc::new(metadata_mock),
            Arc::new(storage_mock),
            Arc::new(event_publisher),
        );
        let input = DeleteFileInput {
            file_id: "file_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteFileError::Internal(msg) => assert!(msg.contains("storage error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
