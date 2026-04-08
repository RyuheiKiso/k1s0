use std::collections::HashMap;

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::{FileMetadataRepository, FileStorageRepository};

pub struct InMemoryFileMetadataRepository {
    files: tokio::sync::RwLock<HashMap<String, FileMetadata>>,
}

impl Default for InMemoryFileMetadataRepository {
    fn default() -> Self {
        Self {
            files: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl InMemoryFileMetadataRepository {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl FileMetadataRepository for InMemoryFileMetadataRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<FileMetadata>> {
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
        let files = self.files.read().await;
        // migration 003 対応: tenant_id フィールドによるフィルタを有効化する
        let mut filtered: Vec<FileMetadata> = files
            .values()
            .filter(|f| {
                // テナント境界を強制する（tenant_id が Some の場合のみフィルタ）
                if let Some(ref tid) = tenant_id {
                    if f.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(ref uploaded_by) = uploaded_by {
                    if f.uploaded_by != *uploaded_by {
                        return false;
                    }
                }
                if let Some(ref content_type) = content_type {
                    if !f.content_type.starts_with(content_type) {
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
        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(())
    }

    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()> {
        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut files = self.files.write().await;
        Ok(files.remove(id).is_some())
    }

    /// テナント分離対応: `tenant_id` フィールドと `storage_path` の両方でテナント境界を確認する（migration 003 対応）
    /// `expected_uploader` が Some の場合は `uploaded_by` も確認してアトミックな認可チェックを実現する。
    async fn delete_with_tenant_check(
        &self,
        id: String,
        tenant_id_prefix: String,
        expected_uploader: Option<String>,
    ) -> anyhow::Result<bool> {
        let mut files = self.files.write().await;
        // tenant_id フィールドと storage_path プレフィックスの両方でテナント境界を確認する（二重防衛）
        let matches = files.get(&id).is_some_and(|f| {
            let tenant_ok = f.tenant_id == tenant_id_prefix
                && f.storage_path.starts_with(&tenant_id_prefix);
            // expected_uploader が None の場合は所有者チェックをスキップし、Some の場合のみ一致検証する
            let uploader_ok = expected_uploader
                .as_deref()
                .is_none_or(|uploader| f.uploaded_by == uploader);
            tenant_ok && uploader_ok
        });
        if matches {
            files.remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct InMemoryFileStorageRepository;

/// `InMemoryFileStorageRepository` の Default `実装（clippy::new_without_default` 対応）
impl Default for InMemoryFileStorageRepository {
    fn default() -> Self {
        Self
    }
}

impl InMemoryFileStorageRepository {
    #[must_use] 
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl FileStorageRepository for InMemoryFileStorageRepository {
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        _content_type: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "https://storage.example.com/upload/{storage_key}?sig=mock"
        ))
    }

    async fn generate_download_url(
        &self,
        storage_key: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "https://storage.example.com/download/{storage_key}?sig=mock"
        ))
    }

    async fn delete_object(&self, _storage_key: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_object_metadata(
        &self,
        _storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }
}
