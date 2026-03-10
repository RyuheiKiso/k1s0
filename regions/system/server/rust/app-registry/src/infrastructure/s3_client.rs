use anyhow::Context;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

/// S3Client は Ceph RGW / S3 互換ストレージとの通信を担当するクライアント。
/// アプリバイナリの保管と署名付きダウンロード URL の生成を行う。
pub struct S3Client {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3Client {
    /// 新しい S3Client を作成する。
    /// endpoint には Ceph RGW の URL を指定する。
    pub async fn new(endpoint: &str, bucket: &str, region: &str) -> Self {
        let credentials = aws_credential_types::provider::SharedCredentialsProvider::new(
            aws_credential_types::Credentials::new(
                std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "minioadmin".to_string()),
                std::env::var("AWS_SECRET_ACCESS_KEY")
                    .unwrap_or_else(|_| "minioadmin".to_string()),
                None,
                None,
                "env",
            ),
        );

        let config = aws_sdk_s3::Config::builder()
            .endpoint_url(endpoint)
            .region(aws_sdk_s3::config::Region::new(region.to_string()))
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        Self {
            client,
            bucket: bucket.to_string(),
        }
    }

    /// 署名付きダウンロード URL を生成する。
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in_secs: u64,
    ) -> anyhow::Result<String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_in_secs))
            .build()
            .context("failed to build presigning config")?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .context("failed to generate presigned URL")?;

        Ok(presigned.uri().to_string())
    }

    /// バケット名を取得する。
    #[allow(dead_code)]
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}
