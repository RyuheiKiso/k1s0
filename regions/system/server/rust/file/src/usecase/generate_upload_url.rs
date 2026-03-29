use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::{FileMetadataRepository, FileStorageRepository};
use crate::domain::service::FileDomainService;

#[derive(Debug, Clone)]
pub struct GenerateUploadUrlInput {
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub tenant_id: String,
    pub uploaded_by: String,
    pub tags: HashMap<String, String>,
    pub expires_in_seconds: u32,
}

#[derive(Debug, Clone)]
pub struct GenerateUploadUrlOutput {
    pub file_id: String,
    pub upload_url: String,
    pub expires_in_seconds: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateUploadUrlError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("file size exceeded: actual={actual}, max={max}")]
    SizeExceeded { actual: u64, max: u64 },

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GenerateUploadUrlUseCase {
    metadata_repo: Arc<dyn FileMetadataRepository>,
    storage_repo: Arc<dyn FileStorageRepository>,
}

impl GenerateUploadUrlUseCase {
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
        input: &GenerateUploadUrlInput,
    ) -> Result<GenerateUploadUrlOutput, GenerateUploadUrlError> {
        if input.filename.is_empty() {
            return Err(GenerateUploadUrlError::Validation(
                "filename is required".to_string(),
            ));
        }
        if input.size_bytes == 0 {
            return Err(GenerateUploadUrlError::Validation(
                "size_bytes must be greater than 0".to_string(),
            ));
        }
        let max_size = FileDomainService::DEFAULT_MAX_FILE_SIZE_BYTES;
        if FileDomainService::validate_upload_size(input.size_bytes, max_size).is_err() {
            return Err(GenerateUploadUrlError::SizeExceeded {
                actual: input.size_bytes,
                max: max_size,
            });
        }
        if input.content_type.is_empty() {
            return Err(GenerateUploadUrlError::Validation(
                "content_type is required".to_string(),
            ));
        }

        let file_id = format!("file_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        // パストラバーサル検証を含むストレージパス生成。不正なファイル名の場合は Validation エラーを返す
        // C-01 監査対応: generate_storage_key → generate_storage_path にリネーム
        let storage_path = FileMetadata::generate_storage_path(&input.tenant_id, &input.filename)
            .map_err(|e| GenerateUploadUrlError::Validation(e.to_string()))?;

        let file = FileMetadata::new(
            file_id.clone(),
            input.filename.clone(),
            input.size_bytes,
            input.content_type.clone(),
            input.uploaded_by.clone(),
            input.tags.clone(),
            storage_path.clone(),
        );

        self.metadata_repo
            .create(&file)
            .await
            .map_err(|e| GenerateUploadUrlError::Internal(e.to_string()))?;

        let upload_url = self
            .storage_repo
            .generate_upload_url(&storage_path, &input.content_type, input.expires_in_seconds)
            .await
            .map_err(|e| GenerateUploadUrlError::Internal(e.to_string()))?;

        Ok(GenerateUploadUrlOutput {
            file_id,
            upload_url,
            expires_in_seconds: input.expires_in_seconds,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::file_repository::{
        MockFileMetadataRepository, MockFileStorageRepository,
    };

    fn valid_input() -> GenerateUploadUrlInput {
        GenerateUploadUrlInput {
            filename: "report.pdf".to_string(),
            size_bytes: 2048,
            content_type: "application/pdf".to_string(),
            tenant_id: "tenant-abc".to_string(),
            uploaded_by: "user-001".to_string(),
            tags: HashMap::new(),
            expires_in_seconds: 3600,
        }
    }

    #[tokio::test]
    async fn success() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let mut storage_mock = MockFileStorageRepository::new();

        metadata_mock.expect_create().returning(|_| Ok(()));
        storage_mock
            .expect_generate_upload_url()
            .returning(|_, _, _| Ok("https://storage.example.com/upload?sig=abc".to_string()));

        let uc = GenerateUploadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let result = uc.execute(&valid_input()).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.file_id.is_empty());
        assert!(output.upload_url.contains("storage.example.com"));
        assert_eq!(output.expires_in_seconds, 3600);
    }

    #[tokio::test]
    async fn validation_empty_name() {
        let metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        let uc = GenerateUploadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let mut input = valid_input();
        input.filename = "".to_string();

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GenerateUploadUrlError::Validation(msg) => assert!(msg.contains("name")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validation_zero_size() {
        let metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        let uc = GenerateUploadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let mut input = valid_input();
        input.size_bytes = 0;

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GenerateUploadUrlError::Validation(msg) => assert!(msg.contains("size_bytes")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_create() {
        let mut metadata_mock = MockFileMetadataRepository::new();
        let storage_mock = MockFileStorageRepository::new();

        metadata_mock
            .expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GenerateUploadUrlUseCase::new(Arc::new(metadata_mock), Arc::new(storage_mock));
        let result = uc.execute(&valid_input()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GenerateUploadUrlError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
