use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::entity::file::FileMetadata;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileMetadataRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<FileMetadata>>;
    async fn find_all(
        &self,
        tenant_id: Option<String>,
        owner_id: Option<String>,
        mime_type: Option<String>,
        tag: Option<(String, String)>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)>;
    async fn create(&self, file: &FileMetadata) -> anyhow::Result<()>;
    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileStorageRepository: Send + Sync {
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        mime_type: &str,
        expires_in_seconds: u32,
    ) -> anyhow::Result<String>;
    async fn generate_download_url(
        &self,
        storage_key: &str,
        expires_in_seconds: u32,
    ) -> anyhow::Result<String>;
    async fn delete_object(&self, storage_key: &str) -> anyhow::Result<()>;
    async fn get_object_metadata(
        &self,
        storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>>;
}
