use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::FileClientConfig;
use crate::error::FileClientError;
use crate::model::{FileMetadata, PresignedUrl};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait FileClient: Send + Sync {
    async fn generate_upload_url(
        &self,
        path: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError>;

    async fn generate_download_url(
        &self,
        path: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError>;

    async fn delete(&self, path: &str) -> Result<(), FileClientError>;

    async fn get_metadata(&self, path: &str) -> Result<FileMetadata, FileClientError>;

    async fn list(&self, prefix: &str) -> Result<Vec<FileMetadata>, FileClientError>;

    async fn copy(&self, src: &str, dst: &str) -> Result<(), FileClientError>;
}

pub struct InMemoryFileClient {
    files: tokio::sync::Mutex<std::collections::HashMap<String, FileMetadata>>,
    _config: FileClientConfig,
}

impl InMemoryFileClient {
    pub fn new(config: FileClientConfig) -> Self {
        Self {
            files: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            _config: config,
        }
    }

    pub async fn stored_files(&self) -> Vec<FileMetadata> {
        let files = self.files.lock().await;
        files.values().cloned().collect()
    }
}

#[async_trait]
impl FileClient for InMemoryFileClient {
    async fn generate_upload_url(
        &self,
        path: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError> {
        let expires_at = chrono::Utc::now() + chrono::Duration::from_std(expires_in).unwrap();
        let meta = FileMetadata {
            path: path.to_string(),
            size_bytes: 0,
            content_type: content_type.to_string(),
            etag: String::new(),
            last_modified: chrono::Utc::now(),
            tags: std::collections::HashMap::new(),
        };
        self.files.lock().await.insert(path.to_string(), meta);
        Ok(PresignedUrl {
            url: format!("https://storage.example.com/upload/{}", path),
            method: "PUT".to_string(),
            expires_at,
            headers: std::collections::HashMap::new(),
        })
    }

    async fn generate_download_url(
        &self,
        path: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError> {
        let files = self.files.lock().await;
        if !files.contains_key(path) {
            return Err(FileClientError::NotFound(path.to_string()));
        }
        let expires_at = chrono::Utc::now() + chrono::Duration::from_std(expires_in).unwrap();
        Ok(PresignedUrl {
            url: format!("https://storage.example.com/download/{}", path),
            method: "GET".to_string(),
            expires_at,
            headers: std::collections::HashMap::new(),
        })
    }

    async fn delete(&self, path: &str) -> Result<(), FileClientError> {
        let mut files = self.files.lock().await;
        files
            .remove(path)
            .ok_or_else(|| FileClientError::NotFound(path.to_string()))?;
        Ok(())
    }

    async fn get_metadata(&self, path: &str) -> Result<FileMetadata, FileClientError> {
        let files = self.files.lock().await;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| FileClientError::NotFound(path.to_string()))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<FileMetadata>, FileClientError> {
        let files = self.files.lock().await;
        Ok(files
            .values()
            .filter(|f| f.path.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn copy(&self, src: &str, dst: &str) -> Result<(), FileClientError> {
        let mut files = self.files.lock().await;
        let source = files
            .get(src)
            .cloned()
            .ok_or_else(|| FileClientError::NotFound(src.to_string()))?;
        let copied = FileMetadata {
            path: dst.to_string(),
            ..source
        };
        files.insert(dst.to_string(), copied);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ServerFileClient — file-server 経由の HTTP 実装
// ---------------------------------------------------------------------------

/// file-server への HTTP リクエスト用内部 DTO
#[derive(Debug, Serialize, Deserialize)]
struct GenerateUrlRequest {
    path: String,
    content_type: Option<String>,
    expires_in_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerateUrlResponse {
    url: String,
    method: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CopyRequest {
    src: String,
    dst: String,
}

/// file-server へ HTTP で委譲する `FileClient` 実装。
pub struct ServerFileClient {
    http: reqwest::Client,
    base_url: String,
}

impl ServerFileClient {
    /// 新しい `ServerFileClient` を生成する。
    ///
    /// `config.server_url` が未設定の場合は `FileClientError::InvalidConfig` を返す。
    pub fn new(config: FileClientConfig) -> Result<Self, FileClientError> {
        let base_url = config
            .server_url
            .ok_or_else(|| FileClientError::InvalidConfig("server_url が設定されていません".into()))?;

        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| FileClientError::Internal(e.to_string()))?;

        Ok(Self { http, base_url })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response, FileClientError> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let body = resp.text().await.unwrap_or_default();
        match status.as_u16() {
            401 | 403 => Err(FileClientError::Unauthorized(body)),
            404 => Err(FileClientError::NotFound(body)),
            _ => Err(FileClientError::Internal(format!("HTTP {}: {}", status, body))),
        }
    }
}

#[async_trait]
impl FileClient for ServerFileClient {
    async fn generate_upload_url(
        &self,
        path: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError> {
        let body = GenerateUrlRequest {
            path: path.to_string(),
            content_type: Some(content_type.to_string()),
            expires_in_secs: expires_in.as_secs(),
        };
        let resp = self
            .http
            .post(self.url("/api/v1/files/upload-url"))
            .json(&body)
            .send()
            .await
            .map_err(|e| FileClientError::ConnectionError(e.to_string()))?;
        let resp = Self::check_status(resp).await?;
        let data: GenerateUrlResponse = resp
            .json()
            .await
            .map_err(|e| FileClientError::Internal(e.to_string()))?;
        Ok(PresignedUrl {
            url: data.url,
            method: data.method,
            expires_at: data.expires_at,
            headers: data.headers,
        })
    }

    async fn generate_download_url(
        &self,
        path: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError> {
        let body = GenerateUrlRequest {
            path: path.to_string(),
            content_type: None,
            expires_in_secs: expires_in.as_secs(),
        };
        let resp = self
            .http
            .post(self.url("/api/v1/files/download-url"))
            .json(&body)
            .send()
            .await
            .map_err(|e| FileClientError::ConnectionError(e.to_string()))?;
        let resp = Self::check_status(resp).await?;
        let data: GenerateUrlResponse = resp
            .json()
            .await
            .map_err(|e| FileClientError::Internal(e.to_string()))?;
        Ok(PresignedUrl {
            url: data.url,
            method: data.method,
            expires_at: data.expires_at,
            headers: data.headers,
        })
    }

    async fn delete(&self, path: &str) -> Result<(), FileClientError> {
        let encoded = urlencoding_simple(path);
        let resp = self
            .http
            .delete(self.url(&format!("/api/v1/files/{}", encoded)))
            .send()
            .await
            .map_err(|e| FileClientError::ConnectionError(e.to_string()))?;
        Self::check_status(resp).await?;
        Ok(())
    }

    async fn get_metadata(&self, path: &str) -> Result<FileMetadata, FileClientError> {
        let encoded = urlencoding_simple(path);
        let resp = self
            .http
            .get(self.url(&format!("/api/v1/files/{}/metadata", encoded)))
            .send()
            .await
            .map_err(|e| FileClientError::ConnectionError(e.to_string()))?;
        let resp = Self::check_status(resp).await?;
        resp.json::<FileMetadata>()
            .await
            .map_err(|e| FileClientError::Internal(e.to_string()))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<FileMetadata>, FileClientError> {
        let resp = self
            .http
            .get(self.url("/api/v1/files"))
            .query(&[("prefix", prefix)])
            .send()
            .await
            .map_err(|e| FileClientError::ConnectionError(e.to_string()))?;
        let resp = Self::check_status(resp).await?;
        resp.json::<Vec<FileMetadata>>()
            .await
            .map_err(|e| FileClientError::Internal(e.to_string()))
    }

    async fn copy(&self, src: &str, dst: &str) -> Result<(), FileClientError> {
        let body = CopyRequest {
            src: src.to_string(),
            dst: dst.to_string(),
        };
        let resp = self
            .http
            .post(self.url("/api/v1/files/copy"))
            .json(&body)
            .send()
            .await
            .map_err(|e| FileClientError::ConnectionError(e.to_string()))?;
        Self::check_status(resp).await?;
        Ok(())
    }
}

/// パス区切り `/` を保持しつつ各セグメントをパーセントエンコードする簡易実装。
/// 外部クレートを追加せずに `reqwest` の URL ビルダーに渡すための補助関数。
fn urlencoding_simple(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            seg.bytes()
                .flat_map(|b| {
                    if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                        vec![b as char]
                    } else {
                        format!("%{:02X}", b).chars().collect::<Vec<_>>()
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> FileClientConfig {
        FileClientConfig::server_mode("http://file-server:8080")
    }

    #[tokio::test]
    async fn test_generate_upload_url() {
        let client = InMemoryFileClient::new(test_config());
        let result = client
            .generate_upload_url("uploads/test.png", "image/png", Duration::from_secs(3600))
            .await
            .unwrap();
        assert!(result.url.contains("uploads/test.png"));
        assert_eq!(result.method, "PUT");
    }

    #[tokio::test]
    async fn test_generate_download_url() {
        let client = InMemoryFileClient::new(test_config());
        client
            .generate_upload_url("uploads/test.png", "image/png", Duration::from_secs(3600))
            .await
            .unwrap();
        let result = client
            .generate_download_url("uploads/test.png", Duration::from_secs(300))
            .await
            .unwrap();
        assert!(result.url.contains("uploads/test.png"));
        assert_eq!(result.method, "GET");
    }

    #[tokio::test]
    async fn test_download_url_not_found() {
        let client = InMemoryFileClient::new(test_config());
        let result = client
            .generate_download_url("nonexistent.txt", Duration::from_secs(300))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let client = InMemoryFileClient::new(test_config());
        client
            .generate_upload_url("uploads/test.png", "image/png", Duration::from_secs(3600))
            .await
            .unwrap();
        client.delete("uploads/test.png").await.unwrap();
        let result = client.get_metadata("uploads/test.png").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let client = InMemoryFileClient::new(test_config());
        client
            .generate_upload_url("uploads/test.png", "image/png", Duration::from_secs(3600))
            .await
            .unwrap();
        let meta = client.get_metadata("uploads/test.png").await.unwrap();
        assert_eq!(meta.path, "uploads/test.png");
        assert_eq!(meta.content_type, "image/png");
    }

    #[tokio::test]
    async fn test_list() {
        let client = InMemoryFileClient::new(test_config());
        client
            .generate_upload_url("uploads/a.png", "image/png", Duration::from_secs(3600))
            .await
            .unwrap();
        client
            .generate_upload_url("uploads/b.jpg", "image/jpeg", Duration::from_secs(3600))
            .await
            .unwrap();
        client
            .generate_upload_url("other/c.txt", "text/plain", Duration::from_secs(3600))
            .await
            .unwrap();
        let files = client.list("uploads/").await.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_copy() {
        let client = InMemoryFileClient::new(test_config());
        client
            .generate_upload_url("uploads/test.png", "image/png", Duration::from_secs(3600))
            .await
            .unwrap();
        client
            .copy("uploads/test.png", "archive/test.png")
            .await
            .unwrap();
        let meta = client.get_metadata("archive/test.png").await.unwrap();
        assert_eq!(meta.content_type, "image/png");
    }

    #[tokio::test]
    async fn test_copy_not_found() {
        let client = InMemoryFileClient::new(test_config());
        let result = client.copy("nonexistent.txt", "dest.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_with_timeout() {
        let config = FileClientConfig::server_mode("http://localhost:8080")
            .with_timeout(Duration::from_secs(60));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.server_url, Some("http://localhost:8080".to_string()));
    }
}
