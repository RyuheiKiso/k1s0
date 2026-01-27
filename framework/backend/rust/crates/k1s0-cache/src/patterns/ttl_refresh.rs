//! TTL リフレッシュパターン
//!
//! アクセス時に TTL を更新することで、頻繁にアクセスされるデータを
//! キャッシュに保持し続けるパターン。
//!
//! ## 動作
//!
//! 1. キャッシュにヒットした場合、TTL を更新してから値を返す
//! 2. キャッシュミスの場合はデータソースから取得
//! 3. 取得した値を初期 TTL でキャッシュに保存
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use k1s0_cache::patterns::{TtlRefresh, TtlRefreshConfig};
//!
//! let config = TtlRefreshConfig::default()
//!     .with_initial_ttl(Duration::from_secs(3600))
//!     .with_refresh_ttl(Duration::from_secs(1800));
//!
//! let ttl_refresh = TtlRefresh::new(cache_client, config);
//!
//! // アクセスするたびに TTL がリフレッシュされる
//! let session = ttl_refresh.get_with_refresh("session:abc").await?;
//! ```

use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument, warn};

use crate::error::CacheResult;
use crate::operations::CacheOperations;

/// TTL リフレッシュ設定
#[derive(Debug, Clone)]
pub struct TtlRefreshConfig {
    /// 初回キャッシュ時の TTL
    pub initial_ttl: Duration,
    /// リフレッシュ時の TTL
    pub refresh_ttl: Duration,
    /// リフレッシュを有効にする残り TTL の閾値
    /// 残り TTL がこの値以下の場合にリフレッシュを行う
    pub refresh_threshold: Option<Duration>,
    /// 非同期リフレッシュを有効にするか
    pub async_refresh: bool,
}

impl Default for TtlRefreshConfig {
    fn default() -> Self {
        Self {
            initial_ttl: Duration::from_secs(3600),
            refresh_ttl: Duration::from_secs(3600),
            refresh_threshold: None,
            async_refresh: false,
        }
    }
}

impl TtlRefreshConfig {
    /// 初回 TTL を設定
    pub fn with_initial_ttl(mut self, ttl: Duration) -> Self {
        self.initial_ttl = ttl;
        self
    }

    /// リフレッシュ TTL を設定
    pub fn with_refresh_ttl(mut self, ttl: Duration) -> Self {
        self.refresh_ttl = ttl;
        self
    }

    /// リフレッシュ閾値を設定
    pub fn with_refresh_threshold(mut self, threshold: Duration) -> Self {
        self.refresh_threshold = Some(threshold);
        self
    }

    /// 非同期リフレッシュを設定
    pub fn with_async_refresh(mut self, async_refresh: bool) -> Self {
        self.async_refresh = async_refresh;
        self
    }
}

/// TTL リフレッシュパターン実装
pub struct TtlRefresh<C: CacheOperations> {
    cache: Arc<C>,
    config: TtlRefreshConfig,
}

impl<C: CacheOperations + 'static> TtlRefresh<C> {
    /// 新しい TtlRefresh を作成
    pub fn new(cache: Arc<C>, config: TtlRefreshConfig) -> Self {
        Self { cache, config }
    }

    /// デフォルト設定で TtlRefresh を作成
    pub fn with_default_config(cache: Arc<C>) -> Self {
        Self::new(cache, TtlRefreshConfig::default())
    }

    /// 設定を取得
    pub fn config(&self) -> &TtlRefreshConfig {
        &self.config
    }

    /// 値を取得し、TTL をリフレッシュ
    #[instrument(skip(self), fields(cache.key = %key))]
    pub async fn get_with_refresh<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>> {
        let value = self.cache.get::<T>(key).await?;

        if value.is_some() {
            if self.should_refresh(key).await {
                self.do_refresh(key).await;
            }
        }

        Ok(value)
    }

    /// 値を取得、なければロードして TTL を設定
    #[instrument(skip(self, loader), fields(cache.key = %key))]
    pub async fn get_or_load_with_refresh<T, F, Fut>(
        &self,
        key: &str,
        loader: F,
    ) -> CacheResult<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<T>> + Send,
    {
        // Try to get from cache
        if let Some(value) = self.cache.get::<T>(key).await? {
            debug!(key = %key, "Cache hit with TTL refresh");
            if self.should_refresh(key).await {
                self.do_refresh(key).await;
            }
            return Ok(value);
        }

        // Cache miss - load from source
        debug!(key = %key, "Cache miss, loading from source");
        let value = loader().await?;

        // Store with initial TTL
        if let Err(e) = self.cache.set(key, &value, Some(self.config.initial_ttl)).await {
            warn!(key = %key, error = %e, "Failed to cache value");
        }

        Ok(value)
    }

    /// リフレッシュが必要かどうかを判定
    async fn should_refresh(&self, key: &str) -> bool {
        if let Some(threshold) = self.config.refresh_threshold {
            // TTL が閾値以下の場合のみリフレッシュ
            match self.cache.ttl(key).await {
                Ok(Some(remaining)) => remaining <= threshold,
                Ok(None) => true, // TTL が設定されていない場合はリフレッシュ
                Err(_) => false,  // エラー時はリフレッシュしない
            }
        } else {
            // 閾値が設定されていない場合は常にリフレッシュ
            true
        }
    }

    /// TTL をリフレッシュ
    async fn do_refresh(&self, key: &str) {
        let cache = self.cache.clone();
        let refresh_ttl = self.config.refresh_ttl;
        let key = key.to_string();

        if self.config.async_refresh {
            // 非同期でリフレッシュ
            tokio::spawn(async move {
                if let Err(e) = cache.expire(&key, refresh_ttl).await {
                    warn!(key = %key, error = %e, "Failed to refresh TTL");
                } else {
                    debug!(key = %key, ttl = ?refresh_ttl, "TTL refreshed");
                }
            });
        } else {
            // 同期でリフレッシュ
            if let Err(e) = self.cache.expire(&key, refresh_ttl).await {
                warn!(key = %key, error = %e, "Failed to refresh TTL");
            } else {
                debug!(key = %key, ttl = ?refresh_ttl, "TTL refreshed");
            }
        }
    }

    /// 明示的に TTL をリフレッシュ
    #[instrument(skip(self), fields(cache.key = %key))]
    pub async fn refresh(&self, key: &str) -> CacheResult<bool> {
        self.cache.expire(key, self.config.refresh_ttl).await
    }

    /// カスタム TTL でリフレッシュ
    #[instrument(skip(self), fields(cache.key = %key))]
    pub async fn refresh_with_ttl(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        self.cache.expire(key, ttl).await
    }
}

/// スライディングウィンドウ TTL
///
/// アクセスするたびに TTL がリセットされるパターン。
/// セッション管理などに適している。
pub struct SlidingWindowTtl<C: CacheOperations> {
    cache: Arc<C>,
    window: Duration,
}

impl<C: CacheOperations + 'static> SlidingWindowTtl<C> {
    /// 新しい SlidingWindowTtl を作成
    pub fn new(cache: Arc<C>, window: Duration) -> Self {
        Self { cache, window }
    }

    /// 値を取得し、ウィンドウをスライド
    pub async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
        let value = self.cache.get::<T>(key).await?;
        if value.is_some() {
            // Always refresh on access
            let _ = self.cache.expire(key, self.window).await;
        }
        Ok(value)
    }

    /// 値を設定
    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> CacheResult<()> {
        self.cache.set(key, value, Some(self.window)).await
    }

    /// 値を削除
    pub async fn delete(&self, key: &str) -> CacheResult<bool> {
        self.cache.delete(key).await
    }

    /// 明示的にウィンドウをリセット
    pub async fn touch(&self, key: &str) -> CacheResult<bool> {
        self.cache.expire(key, self.window).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    // Mock cache with TTL support
    struct MockCacheWithTtl {
        data: RwLock<HashMap<String, (String, Option<std::time::Instant>)>>,
    }

    impl MockCacheWithTtl {
        fn new() -> Self {
            Self {
                data: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl CacheOperations for MockCacheWithTtl {
        async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
            let data = self.data.read().await;
            if let Some((json, expires)) = data.get(key) {
                if let Some(exp) = expires {
                    if std::time::Instant::now() > *exp {
                        return Ok(None);
                    }
                }
                let value = serde_json::from_str(json)
                    .map_err(|e| crate::error::CacheError::deserialization(e.to_string()))?;
                Ok(Some(value))
            } else {
                Ok(None)
            }
        }

        async fn set<T: Serialize + Send + Sync>(
            &self,
            key: &str,
            value: &T,
            ttl: Option<Duration>,
        ) -> CacheResult<()> {
            let json = serde_json::to_string(value)
                .map_err(|e| crate::error::CacheError::serialization(e.to_string()))?;
            let expires = ttl.map(|d| std::time::Instant::now() + d);
            let mut data = self.data.write().await;
            data.insert(key.to_string(), (json, expires));
            Ok(())
        }

        async fn delete(&self, key: &str) -> CacheResult<bool> {
            let mut data = self.data.write().await;
            Ok(data.remove(key).is_some())
        }

        async fn exists(&self, key: &str) -> CacheResult<bool> {
            let data = self.data.read().await;
            Ok(data.contains_key(key))
        }

        async fn get_or_set<T, F, Fut>(
            &self,
            key: &str,
            f: F,
            ttl: Option<Duration>,
        ) -> CacheResult<T>
        where
            T: Serialize + DeserializeOwned + Send + Sync,
            F: FnOnce() -> Fut + Send,
            Fut: Future<Output = CacheResult<T>> + Send,
        {
            if let Some(value) = self.get::<T>(key).await? {
                return Ok(value);
            }
            let value = f().await?;
            self.set(key, &value, ttl).await?;
            Ok(value)
        }

        async fn mget<T: DeserializeOwned + Send>(&self, keys: &[&str]) -> CacheResult<Vec<Option<T>>> {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(self.get(key).await?);
            }
            Ok(results)
        }

        async fn mset<T: Serialize + Send + Sync>(&self, items: &[(&str, &T)], ttl: Option<Duration>) -> CacheResult<()> {
            for (key, value) in items {
                self.set(key, value, ttl).await?;
            }
            Ok(())
        }

        async fn mdel(&self, keys: &[&str]) -> CacheResult<u64> {
            let mut count = 0;
            for key in keys {
                if self.delete(key).await? {
                    count += 1;
                }
            }
            Ok(count)
        }

        async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
            let data = self.data.read().await;
            if let Some((_, Some(expires))) = data.get(key) {
                let now = std::time::Instant::now();
                if *expires > now {
                    return Ok(Some(*expires - now));
                }
            }
            Ok(None)
        }

        async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
            let mut data = self.data.write().await;
            if let Some((json, _)) = data.get(key).cloned() {
                data.insert(key.to_string(), (json, Some(std::time::Instant::now() + ttl)));
                return Ok(true);
            }
            Ok(false)
        }

        async fn set_nx<T: Serialize + Send + Sync>(
            &self,
            key: &str,
            value: &T,
            ttl: Option<Duration>,
        ) -> CacheResult<bool> {
            if self.exists(key).await? {
                return Ok(false);
            }
            self.set(key, value, ttl).await?;
            Ok(true)
        }

        async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64> {
            let current: Option<i64> = self.get(key).await?;
            let new_value = current.unwrap_or(0) + delta;
            self.set(key, &new_value, None).await?;
            Ok(new_value)
        }

        async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64> {
            self.incr(key, -delta).await
        }
    }

    #[tokio::test]
    async fn test_ttl_refresh_config() {
        let config = TtlRefreshConfig::default()
            .with_initial_ttl(Duration::from_secs(600))
            .with_refresh_ttl(Duration::from_secs(300))
            .with_refresh_threshold(Duration::from_secs(60));

        assert_eq!(config.initial_ttl, Duration::from_secs(600));
        assert_eq!(config.refresh_ttl, Duration::from_secs(300));
        assert_eq!(config.refresh_threshold, Some(Duration::from_secs(60)));
    }

    #[tokio::test]
    async fn test_ttl_refresh_get_or_load() {
        let cache = Arc::new(MockCacheWithTtl::new());
        let config = TtlRefreshConfig::default()
            .with_initial_ttl(Duration::from_secs(100));
        let ttl_refresh = TtlRefresh::new(cache.clone(), config);

        // First call - cache miss
        let value: String = ttl_refresh
            .get_or_load_with_refresh("key1", || async {
                Ok("loaded_value".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "loaded_value");

        // Second call - cache hit
        let value: String = ttl_refresh
            .get_or_load_with_refresh("key1", || async {
                Ok("should_not_load".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "loaded_value");
    }

    #[tokio::test]
    async fn test_sliding_window() {
        let cache = Arc::new(MockCacheWithTtl::new());
        let sliding = SlidingWindowTtl::new(cache.clone(), Duration::from_secs(60));

        // Set value
        sliding.set("session:123", &"user_data").await.unwrap();

        // Get should refresh TTL
        let value: Option<String> = sliding.get("session:123").await.unwrap();
        assert_eq!(value, Some("user_data".to_string()));

        // Touch should refresh TTL
        assert!(sliding.touch("session:123").await.unwrap());

        // Delete
        assert!(sliding.delete("session:123").await.unwrap());
        let value: Option<String> = sliding.get("session:123").await.unwrap();
        assert!(value.is_none());
    }
}
