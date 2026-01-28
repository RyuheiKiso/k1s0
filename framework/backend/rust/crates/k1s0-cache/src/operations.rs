//! キャッシュ操作
//!
//! キャッシュの基本操作（get, set, delete, exists）を定義するトレイト。

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::time::Duration;

use crate::error::CacheResult;

/// キャッシュ操作トレイト
///
/// キャッシュの基本操作を定義する。
#[async_trait]
pub trait CacheOperations: Send + Sync {
    /// 値を取得
    ///
    /// キーが存在しない場合は `Ok(None)` を返す。
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>>;

    /// 値を設定
    ///
    /// TTL を指定した場合、その期間後に自動削除される。
    /// TTL が `None` の場合、デフォルト TTL が適用される。
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()>;

    /// 値を削除
    ///
    /// 削除された場合は `true`、キーが存在しなかった場合は `false` を返す。
    async fn delete(&self, key: &str) -> CacheResult<bool>;

    /// キーの存在確認
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// 値を取得、なければ生成して設定
    ///
    /// キャッシュにヒットした場合はその値を返す。
    /// ミスした場合は `f` を実行して値を生成し、キャッシュに設定してから返す。
    async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        f: F,
        ttl: Option<Duration>,
    ) -> CacheResult<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = CacheResult<T>> + Send;

    /// 複数のキーの値を取得
    async fn mget<T: DeserializeOwned + Send>(
        &self,
        keys: &[&str],
    ) -> CacheResult<Vec<Option<T>>>;

    /// 複数のキーと値を設定
    async fn mset<T: Serialize + Send + Sync>(
        &self,
        items: &[(&str, &T)],
        ttl: Option<Duration>,
    ) -> CacheResult<()>;

    /// 複数のキーを削除
    async fn mdel(&self, keys: &[&str]) -> CacheResult<u64>;

    /// TTL を取得
    ///
    /// キーが存在しない場合は `Ok(None)` を返す。
    /// TTL が設定されていない（永続化されている）場合も `Ok(None)` を返す。
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>>;

    /// TTL を更新
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool>;

    /// キーが存在しない場合のみ設定
    ///
    /// 設定された場合は `true`、キーが既に存在した場合は `false` を返す。
    async fn set_nx<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<bool>;

    /// インクリメント
    ///
    /// キーが存在しない場合は 0 から開始して delta を加算する。
    async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64>;

    /// デクリメント
    ///
    /// キーが存在しない場合は 0 から開始して delta を減算する。
    async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64>;
}

/// キャッシュ操作の拡張トレイト
#[async_trait]
pub trait CacheOperationsExt: CacheOperations {
    /// 値を取得し、TTL をリフレッシュ
    async fn get_and_refresh<T: DeserializeOwned + Send>(
        &self,
        key: &str,
        ttl: Duration,
    ) -> CacheResult<Option<T>> {
        let value = self.get::<T>(key).await?;
        if value.is_some() {
            self.expire(key, ttl).await?;
        }
        Ok(value)
    }

    /// パターンに一致するキーを削除
    ///
    /// 注意: この操作は Redis の SCAN + DEL を使用するため、大量のキーがある場合は遅くなる可能性がある。
    async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64>;
}

/// CacheOperationsExt のデフォルト実装
///
/// デフォルト実装は機能を提供しないプレースホルダー。
/// Redis 実装（`CacheClient`）では適切にオーバーライドされる。
///
/// # 注意
///
/// `delete_pattern` のデフォルト実装は常に 0 を返す。
/// 実際のパターンマッチング削除を使用するには、`CacheClient` を使用すること。
#[async_trait]
impl<T: CacheOperations + ?Sized> CacheOperationsExt for T {
    /// パターンに一致するキーを削除（デフォルト実装は何もしない）
    ///
    /// # パターン構文（Redis 実装時）
    ///
    /// - `*` - 任意の文字列にマッチ
    /// - `?` - 任意の1文字にマッチ
    /// - `[abc]` - a, b, c のいずれかにマッチ
    ///
    /// # 使用例
    ///
    /// ```rust,ignore
    /// use k1s0_cache::CacheOperationsExt;
    ///
    /// // "user:" で始まるすべてのキーを削除
    /// let deleted = cache.delete_pattern("user:*").await?;
    /// println!("Deleted {} keys", deleted);
    /// ```
    async fn delete_pattern(&self, _pattern: &str) -> CacheResult<u64> {
        // デフォルト実装は何もしない
        // CacheClient では適切にオーバーライドされる
        Ok(0)
    }
}

/// ハッシュ操作トレイト
#[async_trait]
pub trait HashOperations: Send + Sync {
    /// ハッシュフィールドを取得
    async fn hget<T: DeserializeOwned + Send>(&self, key: &str, field: &str) -> CacheResult<Option<T>>;

    /// ハッシュフィールドを設定
    async fn hset<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> CacheResult<()>;

    /// ハッシュフィールドを削除
    async fn hdel(&self, key: &str, fields: &[&str]) -> CacheResult<u64>;

    /// ハッシュの全フィールドを取得
    async fn hgetall<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> CacheResult<std::collections::HashMap<String, T>>;

    /// ハッシュフィールドの存在確認
    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool>;

    /// ハッシュのフィールド数を取得
    async fn hlen(&self, key: &str) -> CacheResult<u64>;
}

/// リスト操作トレイト
#[async_trait]
pub trait ListOperations: Send + Sync {
    /// リストの先頭に追加
    async fn lpush<T: Serialize + Send + Sync>(&self, key: &str, values: &[T]) -> CacheResult<u64>;

    /// リストの末尾に追加
    async fn rpush<T: Serialize + Send + Sync>(&self, key: &str, values: &[T]) -> CacheResult<u64>;

    /// リストの先頭から取得して削除
    async fn lpop<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>>;

    /// リストの末尾から取得して削除
    async fn rpop<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>>;

    /// リストの範囲を取得
    async fn lrange<T: DeserializeOwned + Send>(
        &self,
        key: &str,
        start: i64,
        stop: i64,
    ) -> CacheResult<Vec<T>>;

    /// リストの長さを取得
    async fn llen(&self, key: &str) -> CacheResult<u64>;
}

/// セット操作トレイト
#[async_trait]
pub trait SetOperations: Send + Sync {
    /// セットにメンバーを追加
    async fn sadd<T: Serialize + Send + Sync>(&self, key: &str, members: &[T]) -> CacheResult<u64>;

    /// セットからメンバーを削除
    async fn srem<T: Serialize + Send + Sync>(&self, key: &str, members: &[T]) -> CacheResult<u64>;

    /// セットのメンバー確認
    async fn sismember<T: Serialize + Send + Sync>(&self, key: &str, member: &T) -> CacheResult<bool>;

    /// セットの全メンバーを取得
    async fn smembers<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Vec<T>>;

    /// セットのメンバー数を取得
    async fn scard(&self, key: &str) -> CacheResult<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// テスト用のインメモリキャッシュ実装
    struct MockCache {
        data: Arc<RwLock<HashMap<String, (String, Option<std::time::Instant>)>>>,
        default_ttl: Duration,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
                default_ttl: Duration::from_secs(3600),
            }
        }
    }

    #[async_trait]
    impl CacheOperations for MockCache {
        async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
            let data = self.data.read().await;
            if let Some((value, expires)) = data.get(key) {
                if let Some(exp) = expires {
                    if std::time::Instant::now() > *exp {
                        return Ok(None);
                    }
                }
                let result = serde_json::from_str(value)
                    .map_err(|e| crate::error::CacheError::deserialization(e.to_string()))?;
                Ok(Some(result))
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
            let expires = ttl.or(Some(self.default_ttl)).map(|d| std::time::Instant::now() + d);
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
            if let Some(entry) = data.get_mut(key) {
                entry.1 = Some(std::time::Instant::now() + ttl);
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
    async fn test_basic_operations() {
        let cache = MockCache::new();

        // Set and Get
        cache.set("key1", &"value1", None).await.unwrap();
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Exists
        assert!(cache.exists("key1").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        // Delete
        assert!(cache.delete("key1").await.unwrap());
        assert!(!cache.delete("key1").await.unwrap());
        assert!(!cache.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_or_set() {
        let cache = MockCache::new();
        let call_count = Arc::new(AtomicU64::new(0));

        // First call should execute the function
        let count_clone = call_count.clone();
        let value: i32 = cache
            .get_or_set(
                "computed",
                || async move {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(42)
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(value, 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call should return cached value
        let count_clone = call_count.clone();
        let value: i32 = cache
            .get_or_set(
                "computed",
                || async move {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(100)
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(value, 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_mget_mset() {
        let cache = MockCache::new();

        // mset
        let items: Vec<(&str, &i32)> = vec![("a", &1), ("b", &2), ("c", &3)];
        cache.mset(&items, None).await.unwrap();

        // mget
        let values: Vec<Option<i32>> = cache.mget(&["a", "b", "c", "d"]).await.unwrap();
        assert_eq!(values, vec![Some(1), Some(2), Some(3), None]);
    }

    #[tokio::test]
    async fn test_incr_decr() {
        let cache = MockCache::new();

        // incr on non-existent key
        let value = cache.incr("counter", 1).await.unwrap();
        assert_eq!(value, 1);

        // incr on existing key
        let value = cache.incr("counter", 5).await.unwrap();
        assert_eq!(value, 6);

        // decr
        let value = cache.decr("counter", 2).await.unwrap();
        assert_eq!(value, 4);
    }

    #[tokio::test]
    async fn test_set_nx() {
        let cache = MockCache::new();

        // First set should succeed
        assert!(cache.set_nx("unique", &"value", None).await.unwrap());

        // Second set should fail
        assert!(!cache.set_nx("unique", &"other", None).await.unwrap());

        // Value should be the first one
        let value: Option<String> = cache.get("unique").await.unwrap();
        assert_eq!(value, Some("value".to_string()));
    }
}
