use std::time::Duration;

/// `EventBus` の設定。
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    buffer_size: usize,
    handler_timeout: Duration,
}

impl EventBusConfig {
    /// デフォルト設定で新しい `EventBusConfig` を生成する。
    /// `buffer_size`: 1024, `handler_timeout`: 30秒
    #[must_use] 
    pub fn new() -> Self {
        Self {
            buffer_size: 1024,
            handler_timeout: Duration::from_secs(30),
        }
    }

    /// バッファサイズを設定する（ビルダーパターン）。
    #[must_use] 
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// ハンドラータイムアウトを設定する（ビルダーパターン）。
    #[must_use] 
    pub fn handler_timeout(mut self, timeout: Duration) -> Self {
        self.handler_timeout = timeout;
        self
    }

    /// 現在のバッファサイズを取得する。
    #[must_use] 
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// 現在のハンドラータイムアウトを取得する。
    #[must_use] 
    pub fn get_handler_timeout(&self) -> Duration {
        self.handler_timeout
    }
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // デフォルト設定でバッファサイズとタイムアウトが正しい初期値になることを確認する。
    #[test]
    fn test_default_config() {
        let config = EventBusConfig::new();
        assert_eq!(config.get_buffer_size(), 1024);
        assert_eq!(config.get_handler_timeout(), Duration::from_secs(30));
    }

    // ビルダーパターンでバッファサイズとタイムアウトを変更できることを確認する。
    #[test]
    fn test_builder_pattern() {
        let config = EventBusConfig::new()
            .buffer_size(2048)
            .handler_timeout(Duration::from_secs(60));
        assert_eq!(config.get_buffer_size(), 2048);
        assert_eq!(config.get_handler_timeout(), Duration::from_secs(60));
    }
}
