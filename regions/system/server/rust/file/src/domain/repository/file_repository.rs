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
        uploaded_by: Option<String>,
        content_type: Option<String>,
        tag: Option<(String, String)>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)>;
    async fn create(&self, file: &FileMetadata) -> anyhow::Result<()>;
    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
    /// CRIT-01 監査対応: テナントIDと所有者IDの条件を追加することで認可と削除をアトミックに実行する。
    /// storage_path が tenant_id_prefix で始まり、かつ uploaded_by が expected_uploader に一致する場合のみ削除する。
    /// expected_uploader が None の場合は所有者チェックをスキップする（sys_admin 用）。
    /// mockall の automock は &str の lifetime を扱えないため、引数型に String を使用する。
    async fn delete_with_tenant_check(
        &self,
        id: String,
        tenant_id_prefix: String,
        expected_uploader: Option<String>,
    ) -> anyhow::Result<bool>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileStorageRepository: Send + Sync {
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        content_type: &str,
        expires_in_seconds: u32,
    ) -> anyhow::Result<String>;
    async fn generate_download_url(
        &self,
        storage_key: &str,
        expires_in_seconds: u32,
    ) -> anyhow::Result<String>;
    async fn delete_object(&self, storage_key: &str) -> anyhow::Result<()>;
    #[allow(dead_code)]
    async fn get_object_metadata(
        &self,
        storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>>;
}
