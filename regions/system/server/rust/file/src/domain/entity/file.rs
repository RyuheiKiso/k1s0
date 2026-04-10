use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::error::FileError;

/// テナント分離のための `tenant_id` を含むファイルメタデータエンティティ
/// DB カラム: id, `tenant_id`, filename, `size_bytes`, `content_type`, `uploaded_by`, tags, `storage_path`, checksum, status, `created_at`, `updated_at`
/// migration 003 で `tenant_id` カラムを追加し、RLS によるテナント分離を実現する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    /// テナントID（RLS ポリシーで参照されるカラム）
    pub tenant_id: String,
    #[serde(alias = "name")]
    pub filename: String,
    pub size_bytes: u64,
    #[serde(alias = "mime_type")]
    pub content_type: String,
    #[serde(alias = "owner_id")]
    pub uploaded_by: String,
    pub tags: HashMap<String, String>,
    /// DB カラム名: `storage_path（旧` `storage_key` から改名）
    #[serde(alias = "storage_key")]
    pub storage_path: String,
    /// DB カラム名: checksum（旧 `checksum_sha256` から改名）
    #[serde(alias = "checksum_sha256")]
    pub checksum: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FileMetadata {
    /// テナント分離対応: `tenant_id` を引数に追加。RLS `set_config` と合わせてテナント境界を強制する。
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        id: String,
        tenant_id: String,
        filename: String,
        size_bytes: u64,
        content_type: String,
        uploaded_by: String,
        tags: HashMap<String, String>,
        storage_path: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            tenant_id,
            filename,
            size_bytes,
            content_type,
            uploaded_by,
            tags,
            storage_path,
            checksum: None,
            status: "pending".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_available(&mut self, checksum: Option<String>) {
        self.status = "available".to_string();
        self.checksum = checksum;
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
    /// MED-03: `tenant_id` と filename の両方に対してパストラバーサル攻撃を防ぐバリデーションを実施する。
    /// `tenant_id` に '/' や '..' が含まれていると {`tenant_id}/{filename`} のテナント境界が破れる。
    pub fn generate_storage_path(tenant_id: &str, filename: &str) -> Result<String, FileError> {
        // tenant_id がスラッシュ・バックスラッシュ・ドット連続を含む場合は拒否する
        // （例: "tenant-a/../tenant-b" でテナント境界を突破するパス操作を防止）
        if tenant_id.is_empty()
            || tenant_id.contains('/')
            || tenant_id.contains('\\')
            || tenant_id.contains("..")
        {
            return Err(FileError::InvalidFileName(
                "テナントIDに不正な文字が含まれています".to_string(),
            ));
        }

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
        Ok(format!("{tenant_id}/{filename}"))
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
        // テナント分離対応: tenant_id 引数を追加
        let file = FileMetadata::new(
            "file_001".to_string(),
            "tenant-abc".to_string(),
            "report.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
            "user-001".to_string(),
            sample_tags(),
            "tenant-abc/report.pdf".to_string(),
        );

        assert_eq!(file.id, "file_001");
        assert_eq!(file.filename, "report.pdf");
        assert_eq!(file.size_bytes, 2048);
        assert_eq!(file.content_type, "application/pdf");
        assert_eq!(file.uploaded_by, "user-001");
        assert_eq!(file.status, "pending");
        assert!(file.checksum.is_none());
    }

    #[test]
    fn mark_available_updates_status_and_checksum() {
        // テナント分離対応: tenant_id 引数を追加
        let mut file = FileMetadata::new(
            "file_002".to_string(),
            "tenant-xyz".to_string(),
            "image.png".to_string(),
            1024,
            "image/png".to_string(),
            "user-002".to_string(),
            HashMap::new(),
            "tenant-xyz/image.png".to_string(),
        );

        let checksum = "abc123".to_string();
        file.mark_available(Some(checksum.clone()));

        assert_eq!(file.status, "available");
        assert_eq!(file.checksum, Some(checksum));
    }

    #[test]
    fn mark_deleted_updates_status() {
        // テナント分離対応: tenant_id 引数を追加
        let mut file = FileMetadata::new(
            "file_003".to_string(),
            "tenant-abc".to_string(),
            "data.csv".to_string(),
            512,
            "text/csv".to_string(),
            "user-003".to_string(),
            HashMap::new(),
            "tenant-abc/data.csv".to_string(),
        );

        file.mark_deleted();
        assert_eq!(file.status, "deleted");
    }

    #[test]
    fn update_tags_replaces_all_tags() {
        // テナント分離対応: tenant_id 引数を追加
        let mut file = FileMetadata::new(
            "file_004".to_string(),
            "tenant-abc".to_string(),
            "doc.txt".to_string(),
            256,
            "text/plain".to_string(),
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
    fn generate_storage_path_formats_correctly() {
        let key = FileMetadata::generate_storage_path("tenant-abc", "reports/file.pdf")
            .expect("有効なファイル名のストレージキー生成は成功する");
        assert_eq!(key, "tenant-abc/reports/file.pdf");
    }

    #[test]
    fn generate_storage_path_rejects_parent_dir() {
        // 親ディレクトリ参照を含むファイル名はエラーを返す
        let result = FileMetadata::generate_storage_path("tenant-abc", "../../../etc/passwd");
        assert!(result.is_err());
        match result.unwrap_err() {
            FileError::InvalidFileName(_) => {}
            e => panic!("expected InvalidFileName, got {:?}", e),
        }
    }

    #[test]
    fn generate_storage_path_rejects_tenant_id_with_slash() {
        // MED-03: tenant_id にスラッシュが含まれる場合はエラーを返す（テナント境界突破防止）
        let result = FileMetadata::generate_storage_path("tenant-a/tenant-b", "file.txt");
        assert!(result.is_err());
        match result.unwrap_err() {
            FileError::InvalidFileName(_) => {}
            e => panic!("expected InvalidFileName, got {:?}", e),
        }
    }

    #[test]
    fn generate_storage_path_rejects_tenant_id_with_traversal() {
        // MED-03: tenant_id に ".." が含まれる場合はエラーを返す（パストラバーサル防止）
        let result = FileMetadata::generate_storage_path("tenant-a/../tenant-b", "file.txt");
        assert!(result.is_err());
        match result.unwrap_err() {
            FileError::InvalidFileName(_) => {}
            e => panic!("expected InvalidFileName, got {:?}", e),
        }
    }

    #[test]
    fn generate_storage_path_rejects_empty_tenant_id() {
        // MED-03: テナントIDが空の場合はエラーを返す
        let result = FileMetadata::generate_storage_path("", "file.txt");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn generate_storage_path_rejects_absolute_path_unix() {
        // Unix 形式の絶対パスはエラーを返すことを検証する（LOW-TEST-04 監査対応: Windows 環境との互換性）
        let result = FileMetadata::generate_storage_path("tenant-abc", "/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(windows)]
    fn generate_storage_path_rejects_absolute_path_windows() {
        // Windows 形式の絶対パスはエラーを返すことを検証する（LOW-TEST-04 監査対応: Unix 環境との互換性）
        let result =
            FileMetadata::generate_storage_path("tenant-abc", r"C:\Windows\System32\config");
        assert!(result.is_err());
    }
}
