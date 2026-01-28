//! Write-Through パターン
//!
//! キャッシュとデータベースに同時書き込みするパターン。
//!
//! ## 動作
//!
//! 1. データベースに書き込み
//! 2. 成功した場合、同じ値をキャッシュにも書き込み
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use k1s0_cache::patterns::{WriteThrough, WriteThroughConfig};
//!
//! let write_through = WriteThrough::new(cache_client, config);
//!
//! // ユーザー情報を保存（DB + キャッシュ）
//! write_through.write(
//!     &format!("user:{}", user_id),
//!     &user,
//!     || async { db.save_user(&user).await },
//! ).await?;
//! ```

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument, warn};

use crate::error::{CacheError, CacheResult};
use crate::operations::CacheOperations;

/// Write-Through 設定
#[derive(Debug, Clone)]
pub struct WriteThroughConfig {
    /// デフォルト TTL
    pub default_ttl: Duration,
    /// キャッシュ書き込み失敗時にエラーを伝播するか
    ///
    /// `true`: キャッシュ書き込みが失敗した場合、全体をエラーとする
    /// `false`: DB 書き込み成功ならキャッシュ失敗は警告のみ（デフォルト）
    pub fail_on_cache_error: bool,
    /// DB 書き込み失敗時にキャッシュを無効化するか
    pub invalidate_on_db_error: bool,
}

impl Default for WriteThroughConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600),
            fail_on_cache_error: false,
            invalidate_on_db_error: true,
        }
    }
}

impl WriteThroughConfig {
    /// デフォルト TTL を設定
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// キャッシュエラー時の挙動を設定
    pub fn fail_on_cache_error(mut self, fail: bool) -> Self {
        self.fail_on_cache_error = fail;
        self
    }

    /// DB エラー時のキャッシュ無効化を設定
    pub fn invalidate_on_db_error(mut self, invalidate: bool) -> Self {
        self.invalidate_on_db_error = invalidate;
        self
    }
}

/// Write-Through パターン実装
pub struct WriteThrough<C: CacheOperations> {
    cache: Arc<C>,
    config: WriteThroughConfig,
}

impl<C: CacheOperations> WriteThrough<C> {
    /// 新しい WriteThrough を作成
    pub fn new(cache: Arc<C>, config: WriteThroughConfig) -> Self {
        Self { cache, config }
    }

    /// デフォルト設定で作成
    pub fn with_default_config(cache: Arc<C>) -> Self {
        Self::new(cache, WriteThroughConfig::default())
    }

    /// 設定を取得
    pub fn config(&self) -> &WriteThroughConfig {
        &self.config
    }

    /// 値を書き込み（DB + キャッシュ）
    ///
    /// DB に書き込んでから、同じ値をキャッシュにも書き込む。
    #[instrument(skip(self, value, db_writer), fields(cache.key = %key))]
    pub async fn write<T, F, Fut>(
        &self,
        key: &str,
        value: &T,
        db_writer: F,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<()>> + Send,
    {
        self.write_with_ttl(key, value, db_writer, self.config.default_ttl).await
    }

    /// TTL を指定して値を書き込み
    #[instrument(skip(self, value, db_writer), fields(cache.key = %key))]
    pub async fn write_with_ttl<T, F, Fut>(
        &self,
        key: &str,
        value: &T,
        db_writer: F,
        ttl: Duration,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<()>> + Send,
    {
        // 1. DB に書き込み
        if let Err(e) = db_writer().await {
            warn!(key = %key, error = %e, "DB write failed");

            // DB 書き込み失敗時、キャッシュを無効化
            if self.config.invalidate_on_db_error {
                let _ = self.cache.delete(key).await;
            }

            return Err(e);
        }

        debug!(key = %key, "DB write succeeded");

        // 2. キャッシュに書き込み
        if let Err(e) = self.cache.set(key, value, Some(ttl)).await {
            if self.config.fail_on_cache_error {
                return Err(e);
            }
            warn!(key = %key, error = %e, "Cache write failed after DB write");
        } else {
            debug!(key = %key, "Cache write succeeded");
        }

        Ok(())
    }

    /// 複数の値を書き込み
    ///
    /// 各アイテムを順番に書き込む。一つでも失敗すると全体がエラーになる。
    #[instrument(skip(self, items, db_writer))]
    pub async fn write_many<T, F, Fut>(
        &self,
        items: &[(&str, T)],
        db_writer: F,
    ) -> CacheResult<()>
    where
        T: Serialize + Send + Sync + Clone,
        F: FnOnce(&[(&str, T)]) -> Fut + Send,
        Fut: Future<Output = CacheResult<()>> + Send,
    {
        // 1. DB に一括書き込み
        db_writer(items).await?;

        // 2. キャッシュに書き込み
        for (key, value) in items {
            if let Err(e) = self.cache.set(key, value, Some(self.config.default_ttl)).await {
                if self.config.fail_on_cache_error {
                    return Err(e);
                }
                warn!(key = %key, error = %e, "Cache write failed");
            }
        }

        Ok(())
    }

    /// 値を削除（DB + キャッシュ）
    #[instrument(skip(self, db_deleter), fields(cache.key = %key))]
    pub async fn delete<F, Fut>(
        &self,
        key: &str,
        db_deleter: F,
    ) -> CacheResult<()>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<()>> + Send,
    {
        // 1. DB から削除
        db_deleter().await?;

        // 2. キャッシュから削除
        if let Err(e) = self.cache.delete(key).await {
            if self.config.fail_on_cache_error {
                return Err(e);
            }
            warn!(key = %key, error = %e, "Cache delete failed after DB delete");
        }

        Ok(())
    }

    /// 値を取得（キャッシュまたは DB）
    ///
    /// Cache-Aside パターンと同様に、キャッシュミス時は DB から取得してキャッシュに格納する。
    #[instrument(skip(self, db_loader), fields(cache.key = %key))]
    pub async fn read<T, F, Fut>(
        &self,
        key: &str,
        db_loader: F,
    ) -> CacheResult<Option<T>>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<Option<T>>> + Send,
    {
        // キャッシュから取得を試みる
        match self.cache.get::<T>(key).await {
            Ok(Some(value)) => {
                debug!(key = %key, "Cache hit");
                return Ok(Some(value));
            }
            Ok(None) => {
                debug!(key = %key, "Cache miss");
            }
            Err(e) => {
                warn!(key = %key, error = %e, "Cache read failed, falling back to DB");
            }
        }

        // DB から取得
        let value = db_loader().await?;

        // キャッシュに格納
        if let Some(ref v) = value {
            if let Err(e) = self.cache.set(key, v, Some(self.config.default_ttl)).await {
                warn!(key = %key, error = %e, "Failed to cache DB result");
            }
        }

        Ok(value)
    }
}

/// Write-Through ストアトレイト
///
/// データストアとの統合を容易にするためのトレイト。
#[async_trait]
pub trait WriteThroughStore<T>: Send + Sync
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

    // テスト用の MockCache
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
                    .map_err(|e| CacheError::deserialization(e.to_string()))?;
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
                .map_err(|e| CacheError::serialization(e.to_string()))?;
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

        async fn get_or_set<T, F, Fut>(&self, key: &str, f: F, ttl: Option<Duration>) -> CacheResult<T>
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

        async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
            Ok(None)
        }

        async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<bool> {
            Ok(true)
        }

        async fn set_nx<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<bool> {
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
    async fn test_write_through_basic() {
        let cache = Arc::new(MockCache::new());
        let write_through = WriteThrough::with_default_config(cache.clone());

        let db_write_count = Arc::new(AtomicU32::new(0));
        let count_clone = db_write_count.clone();

        // Write-through 書き込み
        write_through
            .write("user:1", &"Alice", || async move {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await
            .unwrap();

        // DB に書き込まれた
        assert_eq!(db_write_count.load(Ordering::SeqCst), 1);

        // キャッシュにも書き込まれた
        let cached: Option<String> = cache.get("user:1").await.unwrap();
        assert_eq!(cached, Some("Alice".to_string()));
    }

    #[tokio::test]
    async fn test_write_through_db_failure() {
        let cache = Arc::new(MockCache::new());
        let write_through = WriteThrough::with_default_config(cache.clone());

        // DB 書き込み失敗
        let result: CacheResult<()> = write_through
            .write("user:1", &"Alice", || async move {
                Err(CacheError::internal("DB write failed"))
            })
            .await;

        // エラーが返される
        assert!(result.is_err());

        // キャッシュには書き込まれていない
        let cached: Option<String> = cache.get("user:1").await.unwrap();
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_write_through_read() {
        let cache = Arc::new(MockCache::new());
        let write_through = WriteThrough::with_default_config(cache.clone());

        let db_read_count = Arc::new(AtomicU32::new(0));

        // 最初の読み取り（キャッシュミス、DB から取得）
        let count_clone = db_read_count.clone();
        let value: Option<String> = write_through
            .read("user:1", || async move {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(Some("Bob".to_string()))
            })
            .await
            .unwrap();

        assert_eq!(value, Some("Bob".to_string()));
        assert_eq!(db_read_count.load(Ordering::SeqCst), 1);

        // 2回目の読み取り（キャッシュヒット）
        let count_clone = db_read_count.clone();
        let value: Option<String> = write_through
            .read("user:1", || async move {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(Some("Should not be called".to_string()))
            })
            .await
            .unwrap();

        assert_eq!(value, Some("Bob".to_string()));
        assert_eq!(db_read_count.load(Ordering::SeqCst), 1); // DB は呼ばれていない
    }
}
