use std::sync::Arc;

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::FileMetadataRepository;
use crate::infrastructure::kafka_producer::FileEventPublisher;

#[derive(Debug, Clone)]
/// C-01 監査対応: `checksum_sha256` → checksum にリネーム（DB カラム名に合わせる）
pub struct CompleteUploadInput {
    pub file_id: String,
    pub checksum: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteUploadError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("file already completed: {0}")]
    AlreadyCompleted(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CompleteUploadUseCase {
    repo: Arc<dyn FileMetadataRepository>,
    event_publisher: Arc<dyn FileEventPublisher>,
}

impl CompleteUploadUseCase {
    pub fn new(
        repo: Arc<dyn FileMetadataRepository>,
        event_publisher: Arc<dyn FileEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(
        &self,
        input: &CompleteUploadInput,
    ) -> Result<FileMetadata, CompleteUploadError> {
        let mut file = self
            .repo
            .find_by_id(&input.file_id)
            .await
            .map_err(|e| CompleteUploadError::Internal(e.to_string()))?
            .ok_or_else(|| CompleteUploadError::NotFound(input.file_id.clone()))?;

        if file.status == "available" {
            return Err(CompleteUploadError::AlreadyCompleted(input.file_id.clone()));
        }

        file.mark_available(input.checksum.clone());

        self.repo
            .update(&file)
            .await
            .map_err(|e| CompleteUploadError::Internal(e.to_string()))?;

        // C-01 監査対応: フィールド名を DB カラム名に合わせる
        let payload = serde_json::json!({
            "file_id": file.id,
            "uploaded_by": file.uploaded_by,
            "status": file.status,
            "actor_user_id": file.uploaded_by,
            "before": serde_json::Value::Null,
            "after": {
                "file_id": file.id,
                "status": file.status,
                "checksum": file.checksum,
            },
            "checksum": file.checksum,
            "updated_at": file.updated_at.to_rfc3339(),
        });
        if let Err(e) = self
            .event_publisher
            .publish("file.upload.completed", &payload)
            .await
        {
            tracing::warn!(error = %e, "failed to publish file.upload.completed event");
        }

        Ok(file)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::file_repository::MockFileMetadataRepository;
    use crate::infrastructure::kafka_producer::MockFileEventPublisher;
    use std::collections::HashMap;

    fn pending_file() -> FileMetadata {
        // テナント分離対応: tenant_id 引数を追加
        FileMetadata::new(
            "file_001".to_string(),
            "tenant-abc".to_string(),
            "report.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
            "user-001".to_string(),
            HashMap::new(),
            "tenant-abc/report.pdf".to_string(),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockFileMetadataRepository::new();
        let file = pending_file();
        let return_file = file.clone();

        mock.expect_find_by_id()
            .withf(|id| id == "file_001")
            .returning(move |_| Ok(Some(return_file.clone())));
        mock.expect_update().returning(|_| Ok(()));
        let mut event_publisher = MockFileEventPublisher::new();
        event_publisher.expect_publish().returning(|_, _| Ok(()));

        let uc = CompleteUploadUseCase::new(Arc::new(mock), Arc::new(event_publisher));
        let input = CompleteUploadInput {
            file_id: "file_001".to_string(),
            checksum: Some("sha256hash".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let completed = result.unwrap();
        assert_eq!(completed.status, "available");
        assert_eq!(completed.checksum, Some("sha256hash".to_string()));
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));
        let event_publisher = MockFileEventPublisher::new();

        let uc = CompleteUploadUseCase::new(Arc::new(mock), Arc::new(event_publisher));
        let input = CompleteUploadInput {
            file_id: "missing_file".to_string(),
            checksum: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CompleteUploadError::NotFound(id) => assert_eq!(id, "missing_file"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn already_completed() {
        let mut mock = MockFileMetadataRepository::new();
        let mut file = pending_file();
        file.mark_available(None);
        let return_file = file.clone();

        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(return_file.clone())));
        let event_publisher = MockFileEventPublisher::new();

        let uc = CompleteUploadUseCase::new(Arc::new(mock), Arc::new(event_publisher));
        let input = CompleteUploadInput {
            file_id: "file_001".to_string(),
            checksum: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CompleteUploadError::AlreadyCompleted(id) => assert_eq!(id, "file_001"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));
        let event_publisher = MockFileEventPublisher::new();

        let uc = CompleteUploadUseCase::new(Arc::new(mock), Arc::new(event_publisher));
        let input = CompleteUploadInput {
            file_id: "file_001".to_string(),
            checksum: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CompleteUploadError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
