use std::sync::Arc;

use crate::domain::repository::{FileMetadataRepository, FileStorageRepository};

#[derive(Debug, Clone)]
pub struct GenerateDownloadUrlInput {
    pub file_id: String,
    pub expires_in_seconds: u32,
}

#[derive(Debug, Clone)]
pub struct GenerateDownloadUrlOutput {
    pub file_id: String,
    pub download_url: String,
    pub expires_in_seconds: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateDownloadUrlError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("file not available: {0}")]
    NotAvailable(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GenerateDownloadUrlUseCase {
    metadata_repo: Arc<dyn FileMetadataRepository>,
    storage_repo: Arc<dyn FileStorageRepository>,
}

impl GenerateDownloadUrlUseCase {
    pub fn new(
        metadata_repo: Arc<dyn FileMetadataRepository>,
        storage_repo: Arc<dyn FileStorageRepository>,
    ) -> Self {
        Self {
            metadata_repo,
            storage_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &GenerateDownloadUrlInput,
    ) -> Result<GenerateDownloadUrlOutput, GenerateDownloadUrlError> {
        let file = self
            .metadata_repo
            .find_by_id(&input.file_id)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?
            .ok_or_else(|| GenerateDownloadUrlError::NotFound(input.file_id.clone()))?;

        if file.status != "available" {
            return Err(GenerateDownloadUrlError::NotAvailable(
                input.file_id.clone(),
            ));
        }

        let download_url = self
            .storage_repo
            .generate_download_url(&file.storage_key, input.expires_in_seconds)
            .await
            .map_err(|e| GenerateDownloadUrlError::Internal(e.to_string()))?;

        Ok(GenerateDownloadUrlOutput {
            file_id: file.id,
            download_url,
            expires_in_seconds: input.expires_in_seconds,
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
    use std::collections::HashMap;

    fn available_file() -> FileMetadata {
        let mut file = FileMetadata::new(
            "file_001".to_string(),
            "report.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            HashMap::new(),
            "tenant-abc/report.pdf".to_string(),
        );
        file.mark_available(Some("sha256hash".to_string()));
        file
    }

    #[tokio::test]
    async fn success() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let mut storage_mock = MockFileStorageRepository::new();

        let file = available_file();
        let return_file = file.clone();

        metadata_mock
            .expect_find_by_id()
            .withf(|id| id == "file_001")
            .returning(move |_| Ok(Some(return_file.clone())));

        storage_mock
            .expect_generate_download_url()
            .returning(|_, _| Ok("https://storage.example.com/download?sig=xyz".to_string()));

        let uc = GenerateDownloadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let input = GenerateDownloadUrlInput {
            file_id: "file_001".to_string(),
            expires_in_seconds: 3600,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.file_id, "file_001");
        assert!(output.download_url.contains("storage.example.com"));
    }

    #[tokio::test]
    async fn not_found() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        metadata_mock
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = GenerateDownloadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let input = GenerateDownloadUrlInput {
            file_id: "missing".to_string(),
            expires_in_seconds: 3600,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GenerateDownloadUrlError::NotFound(id) => assert_eq!(id, "missing"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn not_available_pending_file() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        let file = FileMetadata::new(
            "file_002".to_string(),
            "pending.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            HashMap::new(),
            "tenant-abc/pending.pdf".to_string(),
        );
        let return_file = file.clone();

        metadata_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(return_file.clone())));

        let uc = GenerateDownloadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let input = GenerateDownloadUrlInput {
            file_id: "file_002".to_string(),
            expires_in_seconds: 3600,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GenerateDownloadUrlError::NotAvailable(id) => assert_eq!(id, "file_002"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        metadata_mock
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GenerateDownloadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let input = GenerateDownloadUrlInput {
            file_id: "file_001".to_string(),
            expires_in_seconds: 3600,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GenerateDownloadUrlError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
