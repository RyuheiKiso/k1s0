use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_s3::presigning::PresigningConfig;

use crate::domain::repository::FileStorageRepository;

pub struct S3StorageRepository {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3StorageRepository {
    pub async fn new(
        bucket: String,
        region: Option<String>,
        endpoint: Option<String>,
    ) -> anyhow::Result<Self> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(ref r) = region {
            config_loader = config_loader.region(aws_config::Region::new(r.clone()));
        }
        if let Some(ref ep) = endpoint {
            config_loader =
                config_loader.endpoint_url(ep);
        }
        let sdk_config = config_loader.load().await;
        let client = aws_sdk_s3::Client::new(&sdk_config);
        Ok(Self { client, bucket })
    }
}

#[async_trait]
impl FileStorageRepository for S3StorageRepository {
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        mime_type: &str,
        expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        let presigning = PresigningConfig::expires_in(std::time::Duration::from_secs(
            expires_in_seconds as u64,
        ))?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(storage_key)
            .content_type(mime_type)
            .presigned(presigning)
            .await?;

        Ok(presigned.uri().to_string())
    }

    async fn generate_download_url(
        &self,
        storage_key: &str,
        expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        let presigning = PresigningConfig::expires_in(std::time::Duration::from_secs(
            expires_in_seconds as u64,
        ))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_key)
            .presigned(presigning)
            .await?;

        Ok(presigned.uri().to_string())
    }

    async fn delete_object(&self, storage_key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(storage_key)
            .send()
            .await?;
        Ok(())
    }

    async fn get_object_metadata(
        &self,
        storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        let resp = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(storage_key)
            .send()
            .await?;

        let mut metadata = HashMap::new();
        if let Some(ct) = resp.content_type() {
            metadata.insert("content_type".to_string(), ct.to_string());
        }
        if let Some(cl) = resp.content_length() {
            metadata.insert("content_length".to_string(), cl.to_string());
        }
        if let Some(m) = resp.metadata() {
            for (k, v) in m {
                metadata.insert(k.clone(), v.clone());
            }
        }
        Ok(metadata)
    }
}
