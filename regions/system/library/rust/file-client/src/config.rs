use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FileClientConfig {
    pub server_url: Option<String>,
    pub s3_endpoint: Option<String>,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub timeout: Duration,
}

impl FileClientConfig {
    pub fn server_mode(server_url: impl Into<String>) -> Self {
        Self {
            server_url: Some(server_url.into()),
            s3_endpoint: None,
            bucket: None,
            region: None,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn direct_mode(
        s3_endpoint: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            server_url: None,
            s3_endpoint: Some(s3_endpoint.into()),
            bucket: Some(bucket.into()),
            region: Some(region.into()),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
