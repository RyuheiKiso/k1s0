use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub tags: HashMap<String, String>,
    pub storage_key: String,
    pub checksum_sha256: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FileMetadata {
    pub fn new(
        id: String,
        name: String,
        size_bytes: u64,
        mime_type: String,
        tenant_id: String,
        owner_id: String,
        tags: HashMap<String, String>,
        storage_key: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            size_bytes,
            mime_type,
            tenant_id,
            owner_id,
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

    pub fn mark_deleted(&mut self) {
        self.status = "deleted".to_string();
        self.updated_at = Utc::now();
    }

    pub fn update_tags(&mut self, tags: HashMap<String, String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }

    pub fn generate_storage_key(tenant_id: &str, file_name: &str) -> String {
        format!("{}/{}", tenant_id, file_name)
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
        assert_eq!(file.name, "report.pdf");
        assert_eq!(file.size_bytes, 2048);
        assert_eq!(file.mime_type, "application/pdf");
        assert_eq!(file.tenant_id, "tenant-abc");
        assert_eq!(file.owner_id, "user-001");
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
        let key = FileMetadata::generate_storage_key("tenant-abc", "reports/file.pdf");
        assert_eq!(key, "tenant-abc/reports/file.pdf");
    }
}
