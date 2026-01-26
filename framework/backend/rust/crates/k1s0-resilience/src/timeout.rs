//! タイムアウト
//!
//! 依存先呼び出しのタイムアウト管理を提供する。

use crate::error::ResilienceError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// デフォルトのタイムアウト（ミリ秒）
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// 最小タイムアウト（ミリ秒）
pub const MIN_TIMEOUT_MS: u64 = 100;

/// 最大タイムアウト（ミリ秒）
pub const MAX_TIMEOUT_MS: u64 = 300_000;

/// タイムアウト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// タイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    timeout_ms: u64,
    /// 最小タイムアウト（ミリ秒）
    #[serde(default = "default_min_timeout_ms")]
    min_timeout_ms: u64,
    /// 最大タイムアウト（ミリ秒）
    #[serde(default = "default_max_timeout_ms")]
    max_timeout_ms: u64,
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_min_timeout_ms() -> u64 {
    MIN_TIMEOUT_MS
}

fn default_max_timeout_ms() -> u64 {
    MAX_TIMEOUT_MS
}

impl TimeoutConfig {
    /// 新しい設定を作成
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            min_timeout_ms: MIN_TIMEOUT_MS,
            max_timeout_ms: MAX_TIMEOUT_MS,
        }
    }

    /// バリデーション
    pub fn validate(&self) -> Result<(), ResilienceError> {
        if self.timeout_ms < self.min_timeout_ms {
            return Err(ResilienceError::config(format!(
                "timeout {}ms is below minimum {}ms",
                self.timeout_ms, self.min_timeout_ms
            )));
        }

        if self.timeout_ms > self.max_timeout_ms {
            return Err(ResilienceError::config(format!(
                "timeout {}ms exceeds maximum {}ms",
                self.timeout_ms, self.max_timeout_ms
            )));
        }

        Ok(())
    }

    /// タイムアウトを取得
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// タイムアウト（ミリ秒）を取得
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// 最小タイムアウトを設定
    pub fn with_min(mut self, ms: u64) -> Self {
        self.min_timeout_ms = ms;
        self
    }

    /// 最大タイムアウトを設定
    pub fn with_max(mut self, ms: u64) -> Self {
        self.max_timeout_ms = ms;
        self
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            min_timeout_ms: MIN_TIMEOUT_MS,
            max_timeout_ms: MAX_TIMEOUT_MS,
        }
    }
}

/// タイムアウトガード
///
/// タイムアウト付きで処理を実行する。
#[derive(Debug)]
pub struct TimeoutGuard {
    config: TimeoutConfig,
}

impl TimeoutGuard {
    /// 新しいガードを作成
    pub fn new(config: TimeoutConfig) -> Result<Self, ResilienceError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// デフォルト設定でガードを作成
    pub fn default_timeout() -> Self {
        Self {
            config: TimeoutConfig::default(),
        }
    }

    /// タイムアウト付きで処理を実行
    pub async fn execute<F, T>(&self, f: F) -> Result<T, ResilienceError>
    where
        F: std::future::Future<Output = Result<T, ResilienceError>>,
    {
        match tokio::time::timeout(self.config.timeout(), f).await {
            Ok(result) => result,
            Err(_) => Err(ResilienceError::timeout(self.config.timeout_ms)),
        }
    }

    /// 設定を取得
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.timeout_ms(), DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn test_timeout_config_new() {
        let config = TimeoutConfig::new(10_000);
        assert_eq!(config.timeout_ms(), 10_000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_timeout_config_validation_min() {
        let config = TimeoutConfig::new(50);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_config_validation_max() {
        let config = TimeoutConfig::new(400_000);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_config_custom_bounds() {
        let config = TimeoutConfig::new(500)
            .with_min(500)
            .with_max(1000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_timeout_guard_new() {
        let config = TimeoutConfig::new(10_000);
        let guard = TimeoutGuard::new(config);
        assert!(guard.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_guard_execute_success() {
        let guard = TimeoutGuard::default_timeout();
        let result = guard
            .execute(async { Ok::<_, ResilienceError>(42) })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_guard_execute_timeout() {
        let config = TimeoutConfig::new(100);
        let guard = TimeoutGuard::new(config).unwrap();

        let result = guard
            .execute(async {
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok::<_, ResilienceError>(42)
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ResilienceError::Timeout { .. }));
    }
}
