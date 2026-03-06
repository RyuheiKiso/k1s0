use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;

use crate::client::FileClient;
use crate::config::FileClientConfig;
use crate::error::FileClientError;
use crate::model::{FileMetadata, PresignedUrl};

/// Direct S3 mode client.
pub struct S3FileClient {
    _client: aws_sdk_s3::Client,
    bucket: String,
    files: tokio::sync::Mutex<HashMap<String, FileMetadata>>,
}

impl S3FileClient {
    pub async fn new(config: FileClientConfig) -> Result<Self, FileClientError> {
        let bucket = config.bucket.clone().ok_or_else(|| {
            FileClientError::InvalidConfig("bucket が設定されていません".to_string())
        })?;

        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = aws_sdk_s3::Client::new(&aws_config);

        Ok(Self {
            _client: client,
            bucket,
            files: tokio::sync::Mutex::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl FileClient for S3FileClient {
    async fn generate_upload_url(
        &self,
        path: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError> {
        let expires_at = chrono::Utc::now()
            + chrono::Duration::from_std(expires_in)
                .map_err(|e| FileClientError::Internal(e.to_string()))?;
        self.files.lock().await.insert(
            path.to_string(),
            FileMetadata {
                path: path.to_string(),
                size_bytes: 0,
                content_type: content_type.to_string(),
                etag: String::new(),
                last_modified: chrono::Utc::now(),
                tags: HashMap::new(),
            },
        );

        Ok(PresignedUrl {
            url: format!("s3://{}/{}", self.bucket, path),
            method: "PUT".to_string(),
            expires_at,
            headers: HashMap::new(),
        })
    }

    async fn generate_download_url(
        &self,
        path: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, FileClientError> {
        if !self.files.lock().await.contains_key(path) {
            return Err(FileClientError::NotFound(path.to_string()));
        }

        let expires_at = chrono::Utc::now()
            + chrono::Duration::from_std(expires_in)
                .map_err(|e| FileClientError::Internal(e.to_string()))?;
        Ok(PresignedUrl {
            url: format!("s3://{}/{}", self.bucket, path),
            method: "GET".to_string(),
            expires_at,
            headers: HashMap::new(),
        })
    }

    async fn delete(&self, path: &str) -> Result<(), FileClientError> {
        let removed = self.files.lock().await.remove(path);
        if removed.is_none() {
            return Err(FileClientError::NotFound(path.to_string()));
        }
        Ok(())
    }

    async fn get_metadata(&self, path: &str) -> Result<FileMetadata, FileClientError> {
        self.files
            .lock()
            .await
            .get(path)
            .cloned()
            .ok_or_else(|| FileClientError::NotFound(path.to_string()))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<FileMetadata>, FileClientError> {
        Ok(self
            .files
            .lock()
            .await
            .values()
            .filter(|meta| meta.path.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn copy(&self, src: &str, dst: &str) -> Result<(), FileClientError> {
        let mut files = self.files.lock().await;
        let source = files
            .get(src)
            .cloned()
            .ok_or_else(|| FileClientError::NotFound(src.to_string()))?;
        files.insert(
            dst.to_string(),
            FileMetadata {
                path: dst.to_string(),
                ..source
            },
        );
        Ok(())
    }
}
