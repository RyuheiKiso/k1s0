use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use k1s0_file_server::domain::entity::file::FileMetadata;
use k1s0_file_server::domain::repository::{FileMetadataRepository, FileStorageRepository};
use k1s0_file_server::infrastructure::kafka_producer::FileEventPublisher;
use k1s0_file_server::usecase::complete_upload::{
    CompleteUploadError, CompleteUploadInput, CompleteUploadUseCase,
};
use k1s0_file_server::usecase::delete_file::{DeleteFileError, DeleteFileInput, DeleteFileUseCase};
use k1s0_file_server::usecase::generate_download_url::{
    GenerateDownloadUrlError, GenerateDownloadUrlInput, GenerateDownloadUrlUseCase,
};
use k1s0_file_server::usecase::generate_upload_url::{
    GenerateUploadUrlError, GenerateUploadUrlInput, GenerateUploadUrlUseCase,
};
use k1s0_file_server::usecase::get_file_metadata::{
    GetFileMetadataError, GetFileMetadataInput, GetFileMetadataUseCase,
};
use k1s0_file_server::usecase::list_files::{ListFilesError, ListFilesInput, ListFilesUseCase};
use k1s0_file_server::usecase::update_file_tags::{
    UpdateFileTagsError, UpdateFileTagsInput, UpdateFileTagsUseCase,
};

// ---------------------------------------------------------------------------
// Stub: In-memory FileMetadataRepository
// ---------------------------------------------------------------------------

struct StubMetadataRepository {
    files: RwLock<HashMap<String, FileMetadata>>,
    should_fail: bool,
}

impl StubMetadataRepository {
    fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            should_fail: true,
        }
    }

    async fn seed(&self, file: FileMetadata) {
        self.files.write().await.insert(file.id.clone(), file);
    }

    async fn get(&self, id: &str) -> Option<FileMetadata> {
        self.files.read().await.get(id).cloned()
    }

    async fn count(&self) -> usize {
        self.files.read().await.len()
    }
}

#[async_trait]
impl FileMetadataRepository for StubMetadataRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<FileMetadata>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let files = self.files.read().await;
        Ok(files.get(id).cloned())
    }

    async fn find_all(
        &self,
        tenant_id: Option<String>,
        uploaded_by: Option<String>,
        content_type: Option<String>,
        tag: Option<(String, String)>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let files = self.files.read().await;
        let mut filtered: Vec<FileMetadata> = files
            .values()
            .filter(|f| {
                if let Some(ref tid) = tenant_id {
                    if f.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(ref ub) = uploaded_by {
                    if f.uploaded_by != *ub {
                        return false;
                    }
                }
                if let Some(ref ct) = content_type {
                    if !f.content_type.starts_with(ct) {
                        return false;
                    }
                }
                if let Some((ref key, ref value)) = tag {
                    match f.tags.get(key) {
                        Some(v) if v == value => {}
                        _ => return false,
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = page.saturating_sub(1) as usize * page_size as usize;
        filtered = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((filtered, total))
    }

    async fn create(&self, file: &FileMetadata) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(())
    }

    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut files = self.files.write().await;
        Ok(files.remove(id).is_some())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory FileStorageRepository
// ---------------------------------------------------------------------------

struct StubStorageRepository {
    deleted_keys: RwLock<Vec<String>>,
    should_fail: bool,
}

impl StubStorageRepository {
    fn new() -> Self {
        Self {
            deleted_keys: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            deleted_keys: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    async fn deleted_keys(&self) -> Vec<String> {
        self.deleted_keys.read().await.clone()
    }
}

#[async_trait]
impl FileStorageRepository for StubStorageRepository {
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        _content_type: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub storage error"));
        }
        Ok(format!(
            "https://storage.example.com/upload/{}?sig=test",
            storage_key
        ))
    }

    async fn generate_download_url(
        &self,
        storage_key: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub storage error"));
        }
        Ok(format!(
            "https://storage.example.com/download/{}?sig=test",
            storage_key
        ))
    }

    async fn delete_object(&self, storage_key: &str) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub storage error"));
        }
        self.deleted_keys
            .write()
            .await
            .push(storage_key.to_string());
        Ok(())
    }

    async fn get_object_metadata(
        &self,
        _storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub storage error"));
        }
        Ok(HashMap::new())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory FileEventPublisher
// ---------------------------------------------------------------------------

struct StubEventPublisher {
    events: RwLock<Vec<(String, serde_json::Value)>>,
    should_fail: bool,
}

impl StubEventPublisher {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    async fn published_events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.read().await.clone()
    }
}

#[async_trait]
impl FileEventPublisher for StubEventPublisher {
    async fn publish(&self, event_type: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("stub kafka error"));
        }
        self.events
            .write()
            .await
            .push((event_type.to_string(), payload.clone()));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_tags() -> HashMap<String, String> {
    let mut tags = HashMap::new();
    tags.insert("category".to_string(), "report".to_string());
    tags
}

fn pending_file(id: &str) -> FileMetadata {
    FileMetadata::new(
        id.to_string(),
        "report.pdf".to_string(),
        2048,
        "application/pdf".to_string(),
        "tenant-abc".to_string(),
        "user-001".to_string(),
        sample_tags(),
        format!("tenant-abc/{}.pdf", id),
    )
}

fn available_file(id: &str) -> FileMetadata {
    let mut file = pending_file(id);
    file.mark_available(Some("sha256_abc123".to_string()));
    file
}

fn valid_upload_input() -> GenerateUploadUrlInput {
    GenerateUploadUrlInput {
        filename: "report.pdf".to_string(),
        size_bytes: 2048,
        content_type: "application/pdf".to_string(),
        tenant_id: "tenant-abc".to_string(),
        uploaded_by: "user-001".to_string(),
        tags: sample_tags(),
        expires_in_seconds: 3600,
    }
}

// ===========================================================================
// GenerateUploadUrl
// ===========================================================================

mod generate_upload_url {
    use super::*;

    #[tokio::test]
    async fn success_creates_pending_file_and_returns_url() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata.clone(), storage);

        let result = uc.execute(&valid_upload_input()).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.file_id.is_empty());
        assert!(output.upload_url.contains("storage.example.com/upload"));
        assert_eq!(output.expires_in_seconds, 3600);

        // Verify file was persisted in pending state
        let stored = metadata.get(&output.file_id).await;
        assert!(stored.is_some());
        let stored = stored.unwrap();
        assert_eq!(stored.status, "pending");
        assert_eq!(stored.filename, "report.pdf");
        assert_eq!(stored.tenant_id, "tenant-abc");
    }

    #[tokio::test]
    async fn validation_error_empty_filename() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata, storage);

        let mut input = valid_upload_input();
        input.filename = "".to_string();

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateUploadUrlError::Validation(msg) => assert!(msg.contains("name")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validation_error_zero_size() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata, storage);

        let mut input = valid_upload_input();
        input.size_bytes = 0;

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateUploadUrlError::Validation(msg) => assert!(msg.contains("size_bytes")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validation_error_empty_content_type() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata, storage);

        let mut input = valid_upload_input();
        input.content_type = "".to_string();

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateUploadUrlError::Validation(msg) => assert!(msg.contains("content_type")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn size_exceeded_error() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata, storage);

        let mut input = valid_upload_input();
        input.size_bytes = 200 * 1024 * 1024; // 200 MB exceeds 100 MB limit

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateUploadUrlError::SizeExceeded { actual, max } => {
                assert_eq!(actual, 200 * 1024 * 1024);
                assert_eq!(max, 100 * 1024 * 1024);
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_metadata_create_failure() {
        let metadata = Arc::new(StubMetadataRepository::failing());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata, storage);

        let result = uc.execute(&valid_upload_input()).await;
        match result.unwrap_err() {
            GenerateUploadUrlError::Internal(msg) => assert!(msg.contains("stub db error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_storage_url_failure() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::failing());
        let uc = GenerateUploadUrlUseCase::new(metadata, storage);

        let result = uc.execute(&valid_upload_input()).await;
        match result.unwrap_err() {
            GenerateUploadUrlError::Internal(msg) => assert!(msg.contains("error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn tags_are_preserved_in_created_file() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let uc = GenerateUploadUrlUseCase::new(metadata.clone(), storage);

        let input = valid_upload_input();
        let output = uc.execute(&input).await.unwrap();

        let stored = metadata.get(&output.file_id).await.unwrap();
        assert_eq!(stored.tags.get("category"), Some(&"report".to_string()));
    }
}

// ===========================================================================
// CompleteUpload
// ===========================================================================

mod complete_upload {
    use super::*;

    #[tokio::test]
    async fn success_marks_file_available_with_checksum() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let events = Arc::new(StubEventPublisher::new());
        metadata.seed(pending_file("file_001")).await;

        let uc = CompleteUploadUseCase::new(metadata.clone(), events.clone());
        let input = CompleteUploadInput {
            file_id: "file_001".to_string(),
            checksum_sha256: Some("sha256_abc".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.status, "available");
        assert_eq!(file.checksum_sha256, Some("sha256_abc".to_string()));

        // Verify persisted state
        let stored = metadata.get("file_001").await.unwrap();
        assert_eq!(stored.status, "available");

        // Verify event published
        let published = events.published_events().await;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, "file.upload.completed");
    }

    #[tokio::test]
    async fn success_without_checksum() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let events = Arc::new(StubEventPublisher::new());
        metadata.seed(pending_file("file_002")).await;

        let uc = CompleteUploadUseCase::new(metadata.clone(), events);
        let input = CompleteUploadInput {
            file_id: "file_002".to_string(),
            checksum_sha256: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.status, "available");
        assert!(file.checksum_sha256.is_none());
    }

    #[tokio::test]
    async fn not_found_error() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let events = Arc::new(StubEventPublisher::new());

        let uc = CompleteUploadUseCase::new(metadata, events);
        let input = CompleteUploadInput {
            file_id: "nonexistent".to_string(),
            checksum_sha256: None,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            CompleteUploadError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn already_completed_error() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let events = Arc::new(StubEventPublisher::new());
        metadata.seed(available_file("file_003")).await;

        let uc = CompleteUploadUseCase::new(metadata, events);
        let input = CompleteUploadInput {
            file_id: "file_003".to_string(),
            checksum_sha256: None,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            CompleteUploadError::AlreadyCompleted(id) => assert_eq!(id, "file_003"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_find_failure() {
        let metadata = Arc::new(StubMetadataRepository::failing());
        let events = Arc::new(StubEventPublisher::new());

        let uc = CompleteUploadUseCase::new(metadata, events);
        let input = CompleteUploadInput {
            file_id: "file_001".to_string(),
            checksum_sha256: None,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            CompleteUploadError::Internal(msg) => assert!(msg.contains("stub db error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn event_publish_failure_does_not_fail_usecase() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let events = Arc::new(StubEventPublisher::failing());
        metadata.seed(pending_file("file_004")).await;

        let uc = CompleteUploadUseCase::new(metadata.clone(), events);
        let input = CompleteUploadInput {
            file_id: "file_004".to_string(),
            checksum_sha256: Some("sha256_xyz".to_string()),
        };

        // Event publish failure is logged but does not fail the usecase
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.status, "available");
    }
}

// ===========================================================================
// GenerateDownloadUrl
// ===========================================================================

mod generate_download_url {
    use super::*;

    #[tokio::test]
    async fn success_returns_download_url() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        metadata.seed(available_file("file_001")).await;

        let uc = GenerateDownloadUrlUseCase::new(metadata, storage);
        let input = GenerateDownloadUrlInput {
            file_id: "file_001".to_string(),
            expires_in_seconds: 3600,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.file_id, "file_001");
        assert!(output.download_url.contains("storage.example.com/download"));
        assert_eq!(output.expires_in_seconds, 3600);
    }

    #[tokio::test]
    async fn not_found_error() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());

        let uc = GenerateDownloadUrlUseCase::new(metadata, storage);
        let input = GenerateDownloadUrlInput {
            file_id: "missing".to_string(),
            expires_in_seconds: 3600,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateDownloadUrlError::NotFound(id) => assert_eq!(id, "missing"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn not_available_for_pending_file() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        metadata.seed(pending_file("file_pending")).await;

        let uc = GenerateDownloadUrlUseCase::new(metadata, storage);
        let input = GenerateDownloadUrlInput {
            file_id: "file_pending".to_string(),
            expires_in_seconds: 3600,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateDownloadUrlError::NotAvailable(id) => assert_eq!(id, "file_pending"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn not_available_for_deleted_file() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let mut file = pending_file("file_del");
        file.mark_deleted();
        metadata.seed(file).await;

        let uc = GenerateDownloadUrlUseCase::new(metadata, storage);
        let input = GenerateDownloadUrlInput {
            file_id: "file_del".to_string(),
            expires_in_seconds: 3600,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateDownloadUrlError::NotAvailable(id) => assert_eq!(id, "file_del"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_metadata_failure() {
        let metadata = Arc::new(StubMetadataRepository::failing());
        let storage = Arc::new(StubStorageRepository::new());

        let uc = GenerateDownloadUrlUseCase::new(metadata, storage);
        let input = GenerateDownloadUrlInput {
            file_id: "file_001".to_string(),
            expires_in_seconds: 3600,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateDownloadUrlError::Internal(msg) => assert!(msg.contains("stub db error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_storage_failure() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::failing());
        metadata.seed(available_file("file_001")).await;

        let uc = GenerateDownloadUrlUseCase::new(metadata, storage);
        let input = GenerateDownloadUrlInput {
            file_id: "file_001".to_string(),
            expires_in_seconds: 3600,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GenerateDownloadUrlError::Internal(msg) => {
                assert!(msg.contains("stub storage error"))
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }
}

// ===========================================================================
// GetFileMetadata
// ===========================================================================

mod get_file_metadata {
    use super::*;

    #[tokio::test]
    async fn success_returns_file() {
        let metadata = Arc::new(StubMetadataRepository::new());
        metadata.seed(pending_file("file_001")).await;

        let uc = GetFileMetadataUseCase::new(metadata);
        let input = GetFileMetadataInput {
            file_id: "file_001".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.id, "file_001");
        assert_eq!(file.filename, "report.pdf");
        assert_eq!(file.tenant_id, "tenant-abc");
    }

    #[tokio::test]
    async fn not_found_error() {
        let metadata = Arc::new(StubMetadataRepository::new());

        let uc = GetFileMetadataUseCase::new(metadata);
        let input = GetFileMetadataInput {
            file_id: "nonexistent".to_string(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GetFileMetadataError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let metadata = Arc::new(StubMetadataRepository::failing());

        let uc = GetFileMetadataUseCase::new(metadata);
        let input = GetFileMetadataInput {
            file_id: "file_001".to_string(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            GetFileMetadataError::Internal(msg) => assert!(msg.contains("stub db error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }
}

// ===========================================================================
// ListFiles
// ===========================================================================

mod list_files {
    use super::*;

    #[tokio::test]
    async fn success_returns_all_files() {
        let metadata = Arc::new(StubMetadataRepository::new());
        metadata.seed(pending_file("file_001")).await;
        metadata.seed(pending_file("file_002")).await;

        let uc = ListFilesUseCase::new(metadata);
        let input = ListFilesInput {
            tenant_id: Some("tenant-abc".to_string()),
            uploaded_by: None,
            content_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.files.len(), 2);
        assert_eq!(output.total_count, 2);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn empty_result() {
        let metadata = Arc::new(StubMetadataRepository::new());

        let uc = ListFilesUseCase::new(metadata);
        let input = ListFilesInput {
            tenant_id: Some("no-tenant".to_string()),
            uploaded_by: None,
            content_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.files.is_empty());
        assert_eq!(output.total_count, 0);
    }

    #[tokio::test]
    async fn filter_by_tenant_id() {
        let metadata = Arc::new(StubMetadataRepository::new());
        metadata.seed(pending_file("file_001")).await;

        let other_file = FileMetadata::new(
            "file_other".to_string(),
            "other.txt".to_string(),
            512,
            "text/plain".to_string(),
            "tenant-xyz".to_string(),
            "user-002".to_string(),
            HashMap::new(),
            "tenant-xyz/other.txt".to_string(),
        );
        metadata.seed(other_file).await;

        let uc = ListFilesUseCase::new(metadata);
        let input = ListFilesInput {
            tenant_id: Some("tenant-abc".to_string()),
            uploaded_by: None,
            content_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        let output = result.unwrap();
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].tenant_id, "tenant-abc");
    }

    #[tokio::test]
    async fn filter_by_tag() {
        let metadata = Arc::new(StubMetadataRepository::new());
        metadata.seed(pending_file("file_tagged")).await;

        let untagged = FileMetadata::new(
            "file_untagged".to_string(),
            "untagged.txt".to_string(),
            512,
            "text/plain".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            HashMap::new(),
            "tenant-abc/untagged.txt".to_string(),
        );
        metadata.seed(untagged).await;

        let uc = ListFilesUseCase::new(metadata);
        let input = ListFilesInput {
            tenant_id: None,
            uploaded_by: None,
            content_type: None,
            tag: Some(("category".to_string(), "report".to_string())),
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        let output = result.unwrap();
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].id, "file_tagged");
    }

    #[tokio::test]
    async fn pagination_has_next() {
        let metadata = Arc::new(StubMetadataRepository::new());
        for i in 0..5 {
            metadata.seed(pending_file(&format!("file_{:03}", i))).await;
        }

        let uc = ListFilesUseCase::new(metadata);
        let input = ListFilesInput {
            tenant_id: None,
            uploaded_by: None,
            content_type: None,
            tag: None,
            page: 1,
            page_size: 2,
        };

        let result = uc.execute(&input).await;
        let output = result.unwrap();
        assert_eq!(output.files.len(), 2);
        assert_eq!(output.total_count, 5);
        assert!(output.has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let metadata = Arc::new(StubMetadataRepository::failing());

        let uc = ListFilesUseCase::new(metadata);
        let input = ListFilesInput {
            tenant_id: None,
            uploaded_by: None,
            content_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            ListFilesError::Internal(msg) => assert!(msg.contains("stub db error")),
        }
    }
}

// ===========================================================================
// UpdateFileTags
// ===========================================================================

mod update_file_tags {
    use super::*;

    #[tokio::test]
    async fn success_replaces_tags() {
        let metadata = Arc::new(StubMetadataRepository::new());
        metadata.seed(pending_file("file_001")).await;

        let uc = UpdateFileTagsUseCase::new(metadata.clone());
        let mut new_tags = HashMap::new();
        new_tags.insert("reviewed".to_string(), "true".to_string());
        new_tags.insert("priority".to_string(), "high".to_string());

        let input = UpdateFileTagsInput {
            file_id: "file_001".to_string(),
            tags: new_tags.clone(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.tags, new_tags);
        assert!(!file.tags.contains_key("category")); // old tag removed

        // Verify persisted
        let stored = metadata.get("file_001").await.unwrap();
        assert_eq!(stored.tags, new_tags);
    }

    #[tokio::test]
    async fn success_clears_all_tags() {
        let metadata = Arc::new(StubMetadataRepository::new());
        metadata.seed(pending_file("file_001")).await;

        let uc = UpdateFileTagsUseCase::new(metadata.clone());
        let input = UpdateFileTagsInput {
            file_id: "file_001".to_string(),
            tags: HashMap::new(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let file = result.unwrap();
        assert!(file.tags.is_empty());
    }

    #[tokio::test]
    async fn not_found_error() {
        let metadata = Arc::new(StubMetadataRepository::new());

        let uc = UpdateFileTagsUseCase::new(metadata);
        let input = UpdateFileTagsInput {
            file_id: "missing".to_string(),
            tags: HashMap::new(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            UpdateFileTagsError::NotFound(id) => assert_eq!(id, "missing"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let metadata = Arc::new(StubMetadataRepository::failing());

        let uc = UpdateFileTagsUseCase::new(metadata);
        let input = UpdateFileTagsInput {
            file_id: "file_001".to_string(),
            tags: HashMap::new(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            UpdateFileTagsError::Internal(msg) => assert!(msg.contains("stub db error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }
}

// ===========================================================================
// DeleteFile
// ===========================================================================

mod delete_file {
    use super::*;

    #[tokio::test]
    async fn success_deletes_storage_and_metadata() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let events = Arc::new(StubEventPublisher::new());
        metadata.seed(available_file("file_001")).await;

        let uc = DeleteFileUseCase::new(metadata.clone(), storage.clone(), events.clone());
        let input = DeleteFileInput {
            file_id: "file_001".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.message.contains("file_001"));

        // Verify metadata deleted
        assert!(metadata.get("file_001").await.is_none());
        assert_eq!(metadata.count().await, 0);

        // Verify storage object deleted
        let deleted = storage.deleted_keys().await;
        assert_eq!(deleted.len(), 1);

        // Verify event published
        let published = events.published_events().await;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, "file.deleted");
    }

    #[tokio::test]
    async fn not_found_error() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let events = Arc::new(StubEventPublisher::new());

        let uc = DeleteFileUseCase::new(metadata, storage, events);
        let input = DeleteFileInput {
            file_id: "missing".to_string(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            DeleteFileError::NotFound(id) => assert_eq!(id, "missing"),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_storage_delete_failure() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::failing());
        let events = Arc::new(StubEventPublisher::new());
        metadata.seed(available_file("file_001")).await;

        let uc = DeleteFileUseCase::new(metadata, storage, events);
        let input = DeleteFileInput {
            file_id: "file_001".to_string(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            DeleteFileError::Internal(msg) => assert!(msg.contains("stub storage error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error_on_metadata_find_failure() {
        let metadata = Arc::new(StubMetadataRepository::failing());
        let storage = Arc::new(StubStorageRepository::new());
        let events = Arc::new(StubEventPublisher::new());

        let uc = DeleteFileUseCase::new(metadata, storage, events);
        let input = DeleteFileInput {
            file_id: "file_001".to_string(),
        };

        let result = uc.execute(&input).await;
        match result.unwrap_err() {
            DeleteFileError::Internal(msg) => assert!(msg.contains("stub db error")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn event_publish_failure_does_not_fail_delete() {
        let metadata = Arc::new(StubMetadataRepository::new());
        let storage = Arc::new(StubStorageRepository::new());
        let events = Arc::new(StubEventPublisher::failing());
        metadata.seed(available_file("file_001")).await;

        let uc = DeleteFileUseCase::new(metadata.clone(), storage, events);
        let input = DeleteFileInput {
            file_id: "file_001".to_string(),
        };

        // Event publish failure is logged but does not fail the usecase
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        // Verify file was still deleted
        assert!(metadata.get("file_001").await.is_none());
    }
}
