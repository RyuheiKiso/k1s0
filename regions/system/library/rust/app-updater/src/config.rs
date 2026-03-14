use std::time::Duration;

/// アプリアップデーターの設定
#[derive(Debug, Clone)]
pub struct AppUpdaterConfig {
    /// App Registry サーバーの URL
    pub server_url: String,
    /// アプリケーション ID
    pub app_id: String,
    /// プラットフォーム（例: "linux", "darwin", "windows"）
    pub platform: Option<String>,
    /// CPU アーキテクチャ（例: "amd64", "arm64"）
    pub arch: Option<String>,
    /// アップデート確認の間隔
    pub check_interval: Option<Duration>,
    /// HTTP リクエストのタイムアウト
    pub timeout: Option<Duration>,
}

impl Default for AppUpdaterConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            app_id: String::new(),
            platform: None,
            arch: None,
            check_interval: None,
            timeout: Some(Duration::from_secs(10)),
        }
    }
}
