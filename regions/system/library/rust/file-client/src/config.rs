use std::time::Duration;

/// ファイルクライアントの設定。
/// S3/AWS SDK 依存を除去し、file-server 経由（server-mode）のみをサポートする。
#[derive(Debug, Clone)]
pub struct FileClientConfig {
    /// file-server の URL。
    pub server_url: Option<String>,
    /// リクエストのタイムアウト。
    pub timeout: Duration,
}

impl FileClientConfig {
    /// file-server 経由でファイル操作を行うモードの設定を作成する。
    pub fn server_mode(server_url: impl Into<String>) -> Self {
        Self {
            server_url: Some(server_url.into()),
            timeout: Duration::from_secs(30),
        }
    }

    /// タイムアウト時間を設定する。
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
