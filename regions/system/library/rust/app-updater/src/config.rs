use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppUpdaterConfig {
    pub server_url: String,
    pub app_id: String,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub check_interval: Option<Duration>,
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
