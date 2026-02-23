#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    pub ping_interval_ms: Option<u64>,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost".to_string(),
            reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
            ping_interval_ms: None,
        }
    }
}

impl WsConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    pub fn reconnect(mut self, enabled: bool) -> Self {
        self.reconnect = enabled;
        self
    }

    pub fn max_reconnect_attempts(mut self, max: u32) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    pub fn reconnect_delay_ms(mut self, ms: u64) -> Self {
        self.reconnect_delay_ms = ms;
        self
    }

    pub fn ping_interval_ms(mut self, ms: u64) -> Self {
        self.ping_interval_ms = Some(ms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = WsConfig::default();
        assert_eq!(cfg.url, "ws://localhost");
        assert!(cfg.reconnect);
        assert_eq!(cfg.max_reconnect_attempts, 5);
        assert_eq!(cfg.reconnect_delay_ms, 1000);
        assert!(cfg.ping_interval_ms.is_none());
    }

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
