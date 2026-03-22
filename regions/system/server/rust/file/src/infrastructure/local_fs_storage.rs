use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;

use crate::domain::repository::FileStorageRepository;

/// ローカルファイルシステム（PVベース）を使用したストレージリポジトリ実装。
/// S3/AWS SDK依存を排除し、シンプルなファイル I/O でファイルの保存・取得を行う。
pub struct LocalFsStorageRepository {
    /// ファイルを保存するルートディレクトリのパス。
    root_path: PathBuf,
    /// upload/download URL のベース URL（file-server 自身のエンドポイント）。
    base_url: String,
}

impl LocalFsStorageRepository {
    /// 新しい LocalFsStorageRepository を作成する。
    /// root_path にファイルを保存し、base_url を使って URL を生成する。
    pub fn new(root_path: PathBuf, base_url: String) -> Self {
        Self { root_path, base_url }
    }

    /// storage_key をファイルシステムのフルパスに変換する。
    /// パストラバーサル攻撃を防ぐため、root_path 外へのパスは拒否する。
    fn resolve_path(&self, storage_key: &str) -> anyhow::Result<PathBuf> {
        let key_path = PathBuf::from(storage_key);
        // 絶対パスやパストラバーサル（../）を含むキーを拒否する
        if key_path.is_absolute() || key_path.components().any(|c| {
            matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_))
        }) {
            anyhow::bail!("不正なストレージキー: {}", storage_key);
        }
        Ok(self.root_path.join(key_path))
    }
}

#[async_trait]
impl FileStorageRepository for LocalFsStorageRepository {
    /// ファイルのアップロード先 URL を生成する。
    /// 親ディレクトリを事前作成し、file-server のアップロードエンドポイント URL を返す。
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        _content_type: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        let full_path = self.resolve_path(storage_key)?;
        // 親ディレクトリが存在しない場合は作成する
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        // ローカル FS の場合は file-server 自身のエンドポイントへの URL を返す
        Ok(format!("{}/internal/storage/{}", self.base_url, storage_key))
    }

    /// ファイルのダウンロード URL を生成する。
    /// ファイルの存在を確認し、file-server のダウンロードエンドポイント URL を返す。
    async fn generate_download_url(
        &self,
        storage_key: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        let full_path = self.resolve_path(storage_key)?;
        if !full_path.exists() {
            anyhow::bail!("ファイルが存在しません: {}", storage_key);
        }
        Ok(format!("{}/internal/storage/{}", self.base_url, storage_key))
    }

    /// 指定された storage_key に対応するファイルを削除する。
    async fn delete_object(&self, storage_key: &str) -> anyhow::Result<()> {
        let full_path = self.resolve_path(storage_key)?;
        if full_path.exists() {
            tokio::fs::remove_file(&full_path).await?;
        }
        Ok(())
    }

    /// ファイルのメタデータ（サイズ、コンテンツタイプ）を取得する。
    async fn get_object_metadata(
        &self,
        storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        let full_path = self.resolve_path(storage_key)?;
        let metadata = tokio::fs::metadata(&full_path).await?;
        let mut result = HashMap::new();
        result.insert("content_length".to_string(), metadata.len().to_string());
        // 拡張子からコンテンツタイプを推定する
        let content_type = full_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "pdf" => "application/pdf",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "json" => "application/json",
                _ => "application/octet-stream",
            })
            .unwrap_or("application/octet-stream");
        result.insert("content_type".to_string(), content_type.to_string());
        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_repo(dir: &std::path::Path) -> LocalFsStorageRepository {
        LocalFsStorageRepository::new(
            dir.to_path_buf(),
            "http://localhost:8098".to_string(),
        )
    }

    #[tokio::test]
    async fn generate_upload_url_creates_parent_dir() {
        let dir = tempdir().unwrap();
        let repo = make_repo(dir.path());

        let url = repo
            .generate_upload_url("tenant-abc/report.pdf", "application/pdf", 3600)
            .await
            .unwrap();
        // 親ディレクトリが作成されていること
        assert!(dir.path().join("tenant-abc").exists());
        assert!(url.contains("/internal/storage/tenant-abc/report.pdf"));
    }

    #[tokio::test]
    async fn generate_download_url_returns_url_for_existing_file() {
        let dir = tempdir().unwrap();
        let key = "tenant-abc/test.pdf";
        let file_path = dir.path().join("tenant-abc/test.pdf");
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await.unwrap();
        tokio::fs::write(&file_path, b"dummy content").await.unwrap();

        let repo = make_repo(dir.path());
        let url = repo.generate_download_url(key, 3600).await.unwrap();
        assert!(url.contains("/internal/storage/tenant-abc/test.pdf"));
    }

    #[tokio::test]
    async fn generate_download_url_fails_for_missing_file() {
        let dir = tempdir().unwrap();
        let repo = make_repo(dir.path());

        let result = repo.generate_download_url("nonexistent/file.pdf", 3600).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_object_removes_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("to_delete.txt");
        tokio::fs::write(&file_path, b"delete me").await.unwrap();

        let repo = make_repo(dir.path());
        repo.delete_object("to_delete.txt").await.unwrap();
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn delete_object_succeeds_for_missing_file() {
        let dir = tempdir().unwrap();
        let repo = make_repo(dir.path());
        // 存在しないファイルの削除はエラーにならない
        repo.delete_object("nonexistent.txt").await.unwrap();
    }

    #[tokio::test]
    async fn resolve_path_rejects_traversal() {
        let dir = tempdir().unwrap();
        let repo = make_repo(dir.path());

        let result = repo.generate_upload_url("../outside/file.txt", "text/plain", 3600).await;
        assert!(result.is_err());
    }
}
