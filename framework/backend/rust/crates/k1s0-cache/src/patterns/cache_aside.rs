//! Cache-Aside パターン
//!
//! アプリケーションがキャッシュとデータベースの両方を管理するパターン。
//!
//! ## 動作
//!
//! 1. キャッシュにヒットした場合はその値を返す
//! 2. キャッシュミスの場合はデータソースから取得
//! 3. 取得した値をキャッシュに保存してから返す
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use k1s0_cache::patterns::{CacheAside, CacheAsideConfig};
//!
//! let cache_aside = CacheAside::new(cache_client, config);
//!
//! // ユーザー情報を取得（キャッシュまたはDB）
//! let user = cache_aside.get_or_load(
//!     &format!("user:{}", user_id),
//!     || async { db.find_user(user_id).await },
//! ).await?;
//! ```

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument, warn};

use crate::error::CacheResult;
use crate::operations::CacheOperations;

/// Cache-Aside 設定
#[derive(Debug, Clone)]
pub struct CacheAsideConfig {
    /// デフォルト TTL
    pub default_ttl: Duration,
    /// キャッシュ書き込み失敗時にエラーを伝播するか
    pub fail_on_cache_write_error: bool,
    /// キャッシュ読み取り失敗時にデータソースにフォールバックするか
    pub fallback_on_cache_read_error: bool,
}

impl Default for CacheAsideConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600),
            fail_on_cache_write_error: false,
            fallback_on_cache_read_error: true,
        }
    }
}

impl CacheAsideConfig {
    /// デフォルト TTL を設定
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// キャッシュ書き込み失敗時の挙動を設定
    pub fn fail_on_cache_write_error(mut self, fail: bool) -> Self {
        self.fail_on_cache_write_error = fail;
        self
    }

    /// キャッシュ読み取り失敗時のフォールバックを設定
    pub fn fallback_on_cache_read_error(mut self, fallback: bool) -> Self {
        self.fallback_on_cache_read_error = fallback;
        self
    }
}

/// Cache-Aside パターン実装
pub struct CacheAside<C: CacheOperations> {
    cache: Arc<C>,
    config: CacheAsideConfig,
}

impl<C: CacheOperations> CacheAside<C> {
    /// 新しい CacheAside を作成
    pub fn new(cache: Arc<C>, config: CacheAsideConfig) -> Self {
        Self { cache, config }
    }

    /// デフォルト設定で CacheAside を作成
    pub fn with_default_config(cache: Arc<C>) -> Self {
        Self::new(cache, CacheAsideConfig::default())
    }

    /// 設定を取得
    pub fn config(&self) -> &CacheAsideConfig {
        &self.config
    }

    /// 値を取得（キャッシュまたはローダー）
    #[instrument(skip(self, loader), fields(cache.key = %key))]
    pub async fn get_or_load<T, F, Fut>(
        &self,
        key: &str,
        loader: F,
    ) -> CacheResult<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<T>> + Send,
    {
        self.get_or_load_with_ttl(key, loader, self.config.default_ttl).await
    }

    /// TTL を指定して値を取得
    #[instrument(skip(self, loader), fields(cache.key = %key))]
    pub async fn get_or_load_with_ttl<T, F, Fut>(
        &self,
        key: &str,
        loader: F,
        ttl: Duration,
    ) -> CacheResult<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<T>> + Send,
    {
        // Try to get from cache
        match self.cache.get::<T>(key).await {
            Ok(Some(value)) => {
                debug!(key = %key, "Cache hit");
                return Ok(value);
            }
            Ok(None) => {
                debug!(key = %key, "Cache miss");
            }
            Err(e) => {
                if self.config.fallback_on_cache_read_error {
                    warn!(key = %key, error = %e, "Cache read failed, falling back to loader");
                } else {
                    return Err(e);
                }
            }
        }

        // Load from data source
        let value = loader().await?;

        // Write to cache
        if let Err(e) = self.cache.set(key, &value, Some(ttl)).await {
            if self.config.fail_on_cache_write_error {
                return Err(e);
            }
            warn!(key = %key, error = %e, "Failed to write to cache");
        }

        Ok(value)
    }

    /// 値を無効化
    #[instrument(skip(self), fields(cache.key = %key))]
    pub async fn invalidate(&self, key: &str) -> CacheResult<bool> {
        self.cache.delete(key).await
    }

    /// 値を更新（Write-Through）
    #[instrument(skip(self, value, writer), fields(cache.key = %key))]
    pub async fn update<T, F, Fut>(
        &self,
        key: &str,
        value: &T,
        writer: F,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<()>> + Send,
    {
        self.update_with_ttl(key, value, writer, self.config.default_ttl).await
    }

    /// TTL を指定して値を更新
    #[instrument(skip(self, value, writer), fields(cache.key = %key))]
    pub async fn update_with_ttl<T, F, Fut>(
        &self,
        key: &str,
        value: &T,
        writer: F,
        ttl: Duration,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<()>> + Send,
    {
        // Write to data source first
        writer().await?;

        // Then update cache
        if let Err(e) = self.cache.set(key, value, Some(ttl)).await {
            if self.config.fail_on_cache_write_error {
                return Err(e);
            }
            warn!(key = %key, error = %e, "Failed to update cache");
        }

        Ok(())
    }
}

/// Cache-Aside トレイト
///
/// データソースとの統合を容易にするためのトレイト。
#[async_trait]
pub trait CacheAsideLoader<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// キーからデータをロード
    async fn load(&self, key: &str) -> CacheResult<Option<T>>;

    /// データを保存
    async fn save(&self, key: &str, value: &T) -> CacheResult<()>;

    /// データを削除
    async fn remove(&self, key: &str) -> CacheResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::RwLock;

    // Mock cache for testing
    struct MockCache {
        data: RwLock<HashMap<String, String>>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CacheOperations for MockCache {
        async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
            let data = self.data.read().await;
            if let Some(json) = data.get(key) {
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
            _ttl: Option<Duration>,
        ) -> CacheResult<()> {
            let json = serde_json::to_string(value)
                .map_err(|e| crate::error::CacheError::serialization(e.to_string()))?;
            let mut data = self.data.write().await;
            data.insert(key.to_string(), json);
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

        async fn mget<T: DeserializeOwned + Send>(
            &self,
            keys: &[&str],
        ) -> CacheResult<Vec<Option<T>>> {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(self.get(key).await?);
            }
            Ok(results)
        }

        async fn mset<T: Serialize + Send + Sync>(
            &self,
            items: &[(&str, &T)],
            ttl: Option<Duration>,
        ) -> CacheResult<()> {
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

        async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
            Ok(None)
        }

        async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<bool> {
            Ok(true)
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
    async fn test_cache_aside_hit() {
        let cache = Arc::new(MockCache::new());
        let cache_aside = CacheAside::with_default_config(cache.clone());

        // Pre-populate cache
        cache.set("key1", &"cached_value", None).await.unwrap();

        let load_count = Arc::new(AtomicU32::new(0));
        let load_count_clone = load_count.clone();

        let value: String = cache_aside
            .get_or_load("key1", || async move {
                load_count_clone.fetch_add(1, Ordering::SeqCst);
                Ok("loaded_value".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "cached_value");
        assert_eq!(load_count.load(Ordering::SeqCst), 0); // Loader not called
    }

    #[tokio::test]
    async fn test_cache_aside_miss() {
        let cache = Arc::new(MockCache::new());
        let cache_aside = CacheAside::with_default_config(cache.clone());

        let load_count = Arc::new(AtomicU32::new(0));
        let load_count_clone = load_count.clone();

        let value: String = cache_aside
            .get_or_load("key1", || async move {
                load_count_clone.fetch_add(1, Ordering::SeqCst);
                Ok("loaded_value".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "loaded_value");
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        // Second call should hit cache
        let load_count_clone = load_count.clone();
        let value: String = cache_aside
            .get_or_load("key1", || async move {
                load_count_clone.fetch_add(1, Ordering::SeqCst);
                Ok("should_not_be_called".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "loaded_value");
        assert_eq!(load_count.load(Ordering::SeqCst), 1); // Still 1
    }

    #[tokio::test]
    async fn test_cache_aside_invalidate() {
        let cache = Arc::new(MockCache::new());
        let cache_aside = CacheAside::with_default_config(cache.clone());

        cache.set("key1", &"value1", None).await.unwrap();

        assert!(cache_aside.invalidate("key1").await.unwrap());
        assert!(!cache_aside.invalidate("key1").await.unwrap()); // Already deleted
    }
}
