#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    pub ping_interval_ms: Option<u64>,
}

impl WsConfig {
    /// URL を指定して `WsConfig` を生成する。その他のフィールドはデフォルト値を使用する。
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
            ping_interval_ms: None,
        }
    }

    #[must_use] 
    pub fn reconnect(mut self, enabled: bool) -> Self {
        self.reconnect = enabled;
        self
    }

    #[must_use] 
    pub fn max_reconnect_attempts(mut self, max: u32) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    #[must_use] 
    pub fn reconnect_delay_ms(mut self, ms: u64) -> Self {
        self.reconnect_delay_ms = ms;
        self
    }

    #[must_use] 
    pub fn ping_interval_ms(mut self, ms: u64) -> Self {
        self.ping_interval_ms = Some(ms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ビルダーメソッドチェーンで全フィールドを設定した WsConfig が正しい値を持つことを確認する。
    #[test]
    fn test_builder() {
        let cfg = WsConfig::new("ws://example.com")
            .reconnect(false)
            .max_reconnect_attempts(3)
            .reconnect_delay_ms(500)
            .ping_interval_ms(30000);

        assert_eq!(cfg.url, "ws://example.com");
        assert!(!cfg.reconnect);
        assert_eq!(cfg.max_reconnect_attempts, 3);
        assert_eq!(cfg.reconnect_delay_ms, 500);
        assert_eq!(cfg.ping_interval_ms, Some(30000));
    }
}
