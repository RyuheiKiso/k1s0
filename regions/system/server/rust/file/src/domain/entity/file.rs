use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::error::FileError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    #[serde(alias = "name")]
    pub filename: String,
    pub size_bytes: u64,
    #[serde(alias = "mime_type")]
    pub content_type: String,
    pub tenant_id: String,
    #[serde(alias = "owner_id")]
    pub uploaded_by: String,
    pub tags: HashMap<String, String>,
    pub storage_key: String,
    pub checksum_sha256: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FileMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        filename: String,
        size_bytes: u64,
        content_type: String,
        tenant_id: String,
        uploaded_by: String,
        tags: HashMap<String, String>,
        storage_key: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            filename,
            size_bytes,
            content_type,
            tenant_id,
            uploaded_by,
            tags,
            storage_key,
            checksum_sha256: None,
            status: "pending".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_available(&mut self, checksum_sha256: Option<String>) {
        self.status = "available".to_string();
        self.checksum_sha256 = checksum_sha256;
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn mark_deleted(&mut self) {
        self.status = "deleted".to_string();
        self.updated_at = Utc::now();
    }

    pub fn update_tags(&mut self, tags: HashMap<String, String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }

    /// ストレージキーを生成する
    /// パストラバーサル攻撃を防ぐため、ファイル名に親ディレクトリ参照や絶対パスが含まれていないか検証する
    pub fn generate_storage_key(tenant_id: &str, filename: &str) -> Result<String, FileError> {
        let path = Path::new(filename);
        // 絶対パスや親ディレクトリ参照を拒否する（防御的バリデーション）
        if path.is_absolute()
            || path.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::Prefix(_)
                )
            })
        {
            return Err(FileError::InvalidFileName(
                "ファイル名にパストラバーサル文字列が含まれています".to_string(),
            ));
        }
        Ok(format!("{}/{}", tenant_id, filename))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tags() -> HashMap<String, String> {
        let mut tags = HashMap::new();
        tags.insert("category".to_string(), "report".to_string());
        tags
    }

    #[test]
    fn new_creates_pending_file() {
        let file = FileMetadata::new(
            "file_001".to_string(),
            "report.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            sample_tags(),
            "tenant-abc/report.pdf".to_string(),
        );

        assert_eq!(file.id, "file_001");
        assert_eq!(file.filename, "report.pdf");
        assert_eq!(file.size_bytes, 2048);
        assert_eq!(file.content_type, "application/pdf");
        assert_eq!(file.tenant_id, "tenant-abc");
        assert_eq!(file.uploaded_by, "user-001");
        assert_eq!(file.status, "pending");
        assert!(file.checksum_sha256.is_none());
    }

    #[test]
    fn mark_available_updates_status_and_checksum() {
        let mut file = FileMetadata::new(
            "file_002".to_string(),
            "image.png".to_string(),
            1024,
            "image/png".to_string(),
            "tenant-xyz".to_string(),
            "user-002".to_string(),
            HashMap::new(),
            "tenant-xyz/image.png".to_string(),
        );

        let checksum = "abc123".to_string();
        file.mark_available(Some(checksum.clone()));

        assert_eq!(file.status, "available");
        assert_eq!(file.checksum_sha256, Some(checksum));
    }

    #[test]
    fn mark_deleted_updates_status() {
        let mut file = FileMetadata::new(
            "file_003".to_string(),
            "data.csv".to_string(),
            512,
            "text/csv".to_string(),
            "tenant-abc".to_string(),
            "user-003".to_string(),
            HashMap::new(),
            "tenant-abc/data.csv".to_string(),
        );

        file.mark_deleted();
        assert_eq!(file.status, "deleted");
    }

    #[test]
    fn update_tags_replaces_all_tags() {
        let mut file = FileMetadata::new(
            "file_004".to_string(),
            "doc.txt".to_string(),
            256,
            "text/plain".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            sample_tags(),
            "tenant-abc/doc.txt".to_string(),
        );

        let mut new_tags = HashMap::new();
        new_tags.insert("reviewed".to_string(), "true".to_string());
        file.update_tags(new_tags.clone());

        assert_eq!(file.tags, new_tags);
        assert!(!file.tags.contains_key("category"));
    }

    #[test]
    fn generate_storage_key_formats_correctly() {
        let key = FileMetadata::generate_storage_key("tenant-abc", "reports/file.pdf")
            .expect("有効なファイル名のストレージキー生成は成功する");
        assert_eq!(key, "tenant-abc/reports/file.pdf");
    }

    #[test]
    fn generate_storage_key_rejects_parent_dir() {
        // 親ディレクトリ参照を含むファイル名はエラーを返す
        let result = FileMetadata::generate_storage_key("tenant-abc", "../../../etc/passwd");
        assert!(result.is_err());
        match result.unwrap_err() {
            FileError::InvalidFileName(_) => {}
            e => panic!("expected InvalidFileName, got {:?}", e),
        }
    }

    #[test]
    fn generate_storage_key_rejects_absolute_path() {
        // 絶対パスはエラーを返す（Unix 形式）
        let result = FileMetadata::generate_storage_key("tenant-abc", "/etc/passwd");
        assert!(result.is_err());
    }
}
