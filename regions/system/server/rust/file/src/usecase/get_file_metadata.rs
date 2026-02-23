use std::sync::Arc;

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::FileMetadataRepository;

#[derive(Debug, Clone)]
pub struct GetFileMetadataInput {
    pub file_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GetFileMetadataError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFileMetadataUseCase {
    repo: Arc<dyn FileMetadataRepository>,
}

impl GetFileMetadataUseCase {
    pub fn new(repo: Arc<dyn FileMetadataRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &GetFileMetadataInput,
    ) -> Result<FileMetadata, GetFileMetadataError> {
        self.repo
            .find_by_id(&input.file_id)
            .await
            .map_err(|e| GetFileMetadataError::Internal(e.to_string()))?
            .ok_or_else(|| GetFileMetadataError::NotFound(input.file_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::file_repository::MockFileMetadataRepository;
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
        let mut mock = MockFileMetadataRepository::new();
        let file = sample_file();
        let return_file = file.clone();

        mock.expect_find_by_id()
            .withf(|id| id == "file_001")
            .returning(move |_| Ok(Some(return_file.clone())));

        let uc = GetFileMetadataUseCase::new(Arc::new(mock));
        let input = GetFileMetadataInput {
            file_id: "file_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.id, "file_001");
        assert_eq!(file.name, "report.pdf");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetFileMetadataUseCase::new(Arc::new(mock));
        let input = GetFileMetadataInput {
            file_id: "missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetFileMetadataError::NotFound(id) => assert_eq!(id, "missing"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetFileMetadataUseCase::new(Arc::new(mock));
        let input = GetFileMetadataInput {
            file_id: "file_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetFileMetadataError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
