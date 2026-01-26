//! バルクヘッド
//!
//! 依存先ごとに独立したリソースプールを持ち、障害の波及を防ぐ。

use crate::concurrency::{ConcurrencyConfig, ConcurrencyLimiter};
use crate::error::ResilienceError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// バルクヘッド設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    /// デフォルトの同時実行数
    #[serde(default = "default_max_concurrent")]
    default_max_concurrent: u32,
    /// サービスごとの同時実行数
    #[serde(default)]
    service_limits: HashMap<String, u32>,
    /// 待機タイムアウト（ミリ秒）
    #[serde(default = "default_acquire_timeout_ms")]
    acquire_timeout_ms: u64,
}

fn default_max_concurrent() -> u32 {
    100
}

fn default_acquire_timeout_ms() -> u64 {
    5_000
}

impl BulkheadConfig {
    /// 新しい設定を作成
    pub fn new(default_max_concurrent: u32) -> Self {
        Self {
            default_max_concurrent,
            service_limits: HashMap::new(),
            acquire_timeout_ms: default_acquire_timeout_ms(),
        }
    }

    /// サービスごとの制限を追加
    pub fn with_service_limit(mut self, service: impl Into<String>, limit: u32) -> Self {
        self.service_limits.insert(service.into(), limit);
        self
    }

    /// 待機タイムアウトを設定
    pub fn with_acquire_timeout_ms(mut self, ms: u64) -> Self {
        self.acquire_timeout_ms = ms;
        self
    }

    /// サービスの同時実行数上限を取得
    pub fn get_limit(&self, service: &str) -> u32 {
        self.service_limits
            .get(service)
            .copied()
            .unwrap_or(self.default_max_concurrent)
    }

    /// デフォルトの同時実行数を取得
    pub fn default_max_concurrent(&self) -> u32 {
        self.default_max_concurrent
    }

    /// 待機タイムアウトを取得
    pub fn acquire_timeout_ms(&self) -> u64 {
        self.acquire_timeout_ms
    }
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            default_max_concurrent: default_max_concurrent(),
            service_limits: HashMap::new(),
            acquire_timeout_ms: default_acquire_timeout_ms(),
        }
    }
}

/// バルクヘッド
///
/// 依存先ごとに独立したリミッタを持つ。
#[derive(Debug)]
pub struct Bulkhead {
    config: BulkheadConfig,
    limiters: std::sync::RwLock<HashMap<String, Arc<ConcurrencyLimiter>>>,
}

impl Bulkhead {
    /// 新しいバルクヘッドを作成
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            config,
            limiters: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// デフォルト設定でバルクヘッドを作成
    pub fn default_config() -> Self {
        Self::new(BulkheadConfig::default())
    }

    /// サービスのリミッタを取得（なければ作成）
    fn get_or_create_limiter(&self, service: &str) -> Arc<ConcurrencyLimiter> {
        // まず読み取りロックで確認
        {
            let limiters = self.limiters.read().unwrap();
            if let Some(limiter) = limiters.get(service) {
                return limiter.clone();
            }
        }

        // なければ書き込みロックで作成
        let mut limiters = self.limiters.write().unwrap();

        // 再度確認（レースコンディション対策）
        if let Some(limiter) = limiters.get(service) {
            return limiter.clone();
        }

        let limit = self.config.get_limit(service);
        let config = ConcurrencyConfig::new(limit)
            .with_acquire_timeout_ms(self.config.acquire_timeout_ms);
        let limiter = Arc::new(ConcurrencyLimiter::new(config));

        limiters.insert(service.to_string(), limiter.clone());
        limiter
    }

    /// サービスに対して処理を実行
    pub async fn execute<F, T>(&self, service: &str, f: F) -> Result<T, ResilienceError>
    where
        F: std::future::Future<Output = Result<T, ResilienceError>>,
    {
        let limiter = self.get_or_create_limiter(service);
        limiter.execute(f).await
    }

    /// サービスの利用可能な許可数を取得
    pub fn available_permits(&self, service: &str) -> usize {
        let limiters = self.limiters.read().unwrap();
        limiters
            .get(service)
            .map(|l| l.available_permits())
            .unwrap_or_else(|| self.config.get_limit(service) as usize)
    }

    /// サービスのアクティブな実行数を取得
    pub fn active_count(&self, service: &str) -> u64 {
        let limiters = self.limiters.read().unwrap();
        limiters.get(service).map(|l| l.active_count()).unwrap_or(0)
    }

    /// 設定を取得
    pub fn config(&self) -> &BulkheadConfig {
        &self.config
    }

    /// 登録されているサービス数を取得
    pub fn service_count(&self) -> usize {
        self.limiters.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulkhead_config_default() {
        let config = BulkheadConfig::default();
        assert_eq!(config.default_max_concurrent(), 100);
    }

    #[test]
    fn test_bulkhead_config_service_limit() {
        let config = BulkheadConfig::new(50)
            .with_service_limit("auth-service", 10)
            .with_service_limit("config-service", 20);

        assert_eq!(config.get_limit("auth-service"), 10);
        assert_eq!(config.get_limit("config-service"), 20);
        assert_eq!(config.get_limit("unknown-service"), 50);
    }

    #[test]
    fn test_bulkhead_new() {
        let bulkhead = Bulkhead::default_config();
        assert_eq!(bulkhead.service_count(), 0);
    }

    #[tokio::test]
    async fn test_bulkhead_execute() {
        let bulkhead = Bulkhead::default_config();

        let result = bulkhead
            .execute("auth-service", async { Ok::<_, ResilienceError>(42) })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(bulkhead.service_count(), 1);
    }

    #[tokio::test]
    async fn test_bulkhead_isolates_services() {
        let config = BulkheadConfig::new(100)
            .with_service_limit("auth-service", 10)
            .with_service_limit("config-service", 20);
        let bulkhead = Bulkhead::new(config);

        // auth-service の許可数を確認
        bulkhead
            .execute("auth-service", async { Ok::<_, ResilienceError>(()) })
            .await
            .unwrap();
        assert_eq!(bulkhead.available_permits("auth-service"), 10);

        // config-service の許可数を確認
        bulkhead
            .execute("config-service", async { Ok::<_, ResilienceError>(()) })
            .await
            .unwrap();
        assert_eq!(bulkhead.available_permits("config-service"), 20);
    }

    #[test]
    fn test_bulkhead_available_permits_uninitialized() {
        let config = BulkheadConfig::new(100)
            .with_service_limit("auth-service", 10);
        let bulkhead = Bulkhead::new(config);

        // 未初期化のサービスは設定値を返す
        assert_eq!(bulkhead.available_permits("auth-service"), 10);
        assert_eq!(bulkhead.available_permits("unknown"), 100);
    }
}
