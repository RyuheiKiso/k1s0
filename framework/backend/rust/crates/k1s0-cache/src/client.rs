//! キャッシュクライアント
//!
//! Redis キャッシュクライアントの実装。

#[cfg(feature = "redis")]
use async_trait::async_trait;
#[cfg(feature = "redis")]
use redis::AsyncCommands;
#[cfg(feature = "redis")]
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "redis")]
use std::future::Future;
#[cfg(feature = "redis")]
use std::time::Duration;
#[cfg(feature = "redis")]
use tracing::{debug, instrument, warn};

#[cfg(feature = "redis")]
use crate::config::CacheConfig;
#[cfg(feature = "redis")]
use crate::error::{CacheError, CacheResult};
#[cfg(feature = "redis")]
use crate::metrics::{CacheMetrics, CacheOperation};
#[cfg(feature = "redis")]
use crate::operations::{CacheOperations, HashOperations, ListOperations, SetOperations};
#[cfg(feature = "redis")]
use crate::pool::RedisPool;

/// Redis キャッシュクライアント
#[cfg(feature = "redis")]
pub struct CacheClient {
    pool: RedisPool,
    config: CacheConfig,
    metrics: CacheMetrics,
}

#[cfg(feature = "redis")]
impl CacheClient {
    /// 新しいキャッシュクライアントを作成
    pub async fn new(config: CacheConfig) -> CacheResult<Self> {
        config.validate()?;
        let pool = RedisPool::new(&config).await?;
        let metrics = CacheMetrics::new();

        debug!(
            url = %config.connection_url_redacted(),
            "Cache client connected"
        );

        Ok(Self {
            pool,
            config,
            metrics,
        })
    }

    /// 設定を取得
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// メトリクスを取得
    pub fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }

    /// プールの接続状態を取得
    pub fn pool_status(&self) -> crate::pool::PoolStatus {
        self.pool.status()
    }

    /// キーにプレフィックスを付与
    fn prefixed_key(&self, key: &str) -> String {
        self.config.prefixed_key(key)
    }

    /// 接続を取得
    async fn get_connection(
        &self,
    ) -> CacheResult<bb8::PooledConnection<'_, bb8_redis::RedisConnectionManager>> {
        self.pool.get().await
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl CacheOperations for CacheClient {
    #[instrument(skip(self), fields(cache.key = %key))]
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let result: Option<String> = conn
            .get(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        match result {
            Some(json) => {
                self.metrics.record_hit(CacheOperation::Get);
                let value = serde_json::from_str(&json)
                    .map_err(|e| CacheError::deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => {
                self.metrics.record_miss(CacheOperation::Get);
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, value), fields(cache.key = %key))]
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let prefixed = self.prefixed_key(key);
        let json =
            serde_json::to_string(value).map_err(|e| CacheError::serialization(e.to_string()))?;

        let ttl = ttl.unwrap_or_else(|| self.config.default_ttl.default_ttl());
        let ttl = self.config.default_ttl.clamp_ttl(ttl);

        let mut conn = self.get_connection().await?;

        conn.set_ex::<_, _, ()>(&prefixed, &json, ttl.as_secs())
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::Set);
        Ok(())
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let deleted: i64 = conn
            .del(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::Delete);
        Ok(deleted > 0)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let exists: bool = conn
            .exists(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(exists)
    }

    #[instrument(skip(self, f), fields(cache.key = %key))]
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
        // Try to get from cache first
        if let Some(value) = self.get::<T>(key).await? {
            return Ok(value);
        }

        // Cache miss - compute value
        let value = f().await?;

        // Store in cache (best effort - don't fail if caching fails)
        if let Err(e) = self.set(key, &value, ttl).await {
            warn!(error = %e, "Failed to cache computed value");
        }

        Ok(value)
    }

    #[instrument(skip(self))]
    async fn mget<T: DeserializeOwned + Send>(&self, keys: &[&str]) -> CacheResult<Vec<Option<T>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let prefixed_keys: Vec<String> = keys.iter().map(|k| self.prefixed_key(k)).collect();
        let mut conn = self.get_connection().await?;

        let results: Vec<Option<String>> = conn
            .mget(&prefixed_keys)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        let mut values = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Some(json) => {
                    self.metrics.record_hit(CacheOperation::Get);
                    let value = serde_json::from_str(&json)
                        .map_err(|e| CacheError::deserialization(e.to_string()))?;
                    values.push(Some(value));
                }
                None => {
                    self.metrics.record_miss(CacheOperation::Get);
                    values.push(None);
                }
            }
        }

        Ok(values)
    }

    #[instrument(skip(self, items))]
    async fn mset<T: Serialize + Send + Sync>(
        &self,
        items: &[(&str, &T)],
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let ttl = ttl.unwrap_or_else(|| self.config.default_ttl.default_ttl());
        let ttl = self.config.default_ttl.clamp_ttl(ttl);
        let ttl_secs = ttl.as_secs();

        let mut conn = self.get_connection().await?;

        // Use pipeline for atomicity
        let mut pipe = redis::pipe();

        for (key, value) in items {
            let prefixed = self.prefixed_key(key);
            let json = serde_json::to_string(value)
                .map_err(|e| CacheError::serialization(e.to_string()))?;
            pipe.set_ex(&prefixed, &json, ttl_secs);
        }

        let _: () = pipe.query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operations(CacheOperation::Set, items.len() as u64);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn mdel(&self, keys: &[&str]) -> CacheResult<u64> {
        if keys.is_empty() {
            return Ok(0);
        }

        let prefixed_keys: Vec<String> = keys.iter().map(|k| self.prefixed_key(k)).collect();
        let mut conn = self.get_connection().await?;

        let deleted: u64 = conn
            .del(&prefixed_keys)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operations(CacheOperation::Delete, deleted);
        Ok(deleted)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let ttl: i64 = conn
            .ttl(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        if ttl < 0 {
            return Ok(None);
        }

        Ok(Some(Duration::from_secs(ttl as u64)))
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let result: bool = conn
            .expire(&prefixed, ttl.as_secs() as i64)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(result)
    }

    #[instrument(skip(self, value), fields(cache.key = %key))]
    async fn set_nx<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<bool> {
        let prefixed = self.prefixed_key(key);
        let json =
            serde_json::to_string(value).map_err(|e| CacheError::serialization(e.to_string()))?;

        let mut conn = self.get_connection().await?;

        let ttl = ttl.unwrap_or_else(|| self.config.default_ttl.default_ttl());
        let ttl = self.config.default_ttl.clamp_ttl(ttl);

        // Use SET NX EX command
        let result: Option<String> = redis::cmd("SET")
            .arg(&prefixed)
            .arg(&json)
            .arg("NX")
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        let set = result.is_some();
        if set {
            self.metrics.record_operation(CacheOperation::Set);
        }
        Ok(set)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let value: i64 = conn
            .incr(&prefixed, delta)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::Incr);
        Ok(value)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let value: i64 = conn
            .decr(&prefixed, delta)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::Decr);
        Ok(value)
    }
}

#[cfg(feature = "redis")]
impl CacheClient {
    /// パターンに一致するキーを削除
    ///
    /// 注意: この操作は Redis の SCAN + DEL を使用するため、大量のキーがある場合は遅くなる可能性がある。
    ///
    /// # パターン構文
    ///
    /// Redis の SCAN コマンドのパターンマッチングを使用:
    /// - `*` - 任意の文字列にマッチ
    /// - `?` - 任意の1文字にマッチ
    /// - `[abc]` - a, b, c のいずれかにマッチ
    /// - `[^abc]` または `[!abc]` - a, b, c 以外にマッチ
    /// - `[a-z]` - a から z の範囲にマッチ
    ///
    /// # 使用例
    ///
    /// ```rust,ignore
    /// // "user:" で始まるすべてのキーを削除
    /// let deleted = client.delete_by_pattern("user:*").await?;
    ///
    /// // "session:abc123:*" にマッチするキーを削除
    /// let deleted = client.delete_by_pattern("session:abc123:*").await?;
    /// ```
    #[instrument(skip(self), fields(cache.pattern = %pattern))]
    pub async fn delete_by_pattern(&self, pattern: &str) -> CacheResult<u64> {
        self.delete_pattern_internal(pattern).await
    }

    /// パターンマッチング削除の内部実装
    async fn delete_pattern_internal(&self, pattern: &str) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(pattern);
        let mut conn = self.get_connection().await?;

        let mut deleted = 0u64;
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&prefixed)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut *conn)
                .await
                .map_err(|e| CacheError::internal(e.to_string()))?;

            if !keys.is_empty() {
                let count: u64 = conn
                    .del(&keys)
                    .await
                    .map_err(|e| CacheError::internal(e.to_string()))?;
                deleted += count;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        self.metrics.record_operations(CacheOperation::Delete, deleted);
        Ok(deleted)
    }

    /// パターンに一致するキーをスキャン（削除せずにキー一覧を取得）
    ///
    /// 注意: この操作は Redis の SCAN を使用するため、大量のキーがある場合は遅くなる可能性がある。
    #[instrument(skip(self), fields(cache.pattern = %pattern))]
    pub async fn scan_keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let prefixed = self.prefixed_key(pattern);
        let mut conn = self.get_connection().await?;

        let mut all_keys = Vec::new();
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&prefixed)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut *conn)
                .await
                .map_err(|e| CacheError::internal(e.to_string()))?;

            all_keys.extend(keys);

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
    }
}

/// CacheOperationsExt の CacheClient 実装
#[cfg(feature = "redis")]
#[async_trait]
impl crate::operations::CacheOperationsExt for CacheClient {
    async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64> {
        self.delete_pattern_internal(pattern).await
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl HashOperations for CacheClient {
    #[instrument(skip(self), fields(cache.key = %key, cache.field = %field))]
    async fn hget<T: DeserializeOwned + Send>(
        &self,
        key: &str,
        field: &str,
    ) -> CacheResult<Option<T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let result: Option<String> = conn
            .hget(&prefixed, field)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        match result {
            Some(json) => {
                self.metrics.record_hit(CacheOperation::HGet);
                let value = serde_json::from_str(&json)
                    .map_err(|e| CacheError::deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => {
                self.metrics.record_miss(CacheOperation::HGet);
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, value), fields(cache.key = %key, cache.field = %field))]
    async fn hset<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> CacheResult<()> {
        let prefixed = self.prefixed_key(key);
        let json =
            serde_json::to_string(value).map_err(|e| CacheError::serialization(e.to_string()))?;

        let mut conn = self.get_connection().await?;

        conn.hset::<_, _, _, ()>(&prefixed, field, &json)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::HSet);
        Ok(())
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn hdel(&self, key: &str, fields: &[&str]) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let deleted: u64 = conn
            .hdel(&prefixed, fields)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::HDel);
        Ok(deleted)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn hgetall<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> CacheResult<std::collections::HashMap<String, T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let raw: std::collections::HashMap<String, String> = conn
            .hgetall(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        let mut result = std::collections::HashMap::new();
        for (field, json) in raw {
            let value = serde_json::from_str(&json)
                .map_err(|e| CacheError::deserialization(e.to_string()))?;
            result.insert(field, value);
        }

        self.metrics.record_operation(CacheOperation::HGetAll);
        Ok(result)
    }

    #[instrument(skip(self), fields(cache.key = %key, cache.field = %field))]
    async fn hexists(&self, key: &str, field: &str) -> CacheResult<bool> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let exists: bool = conn
            .hexists(&prefixed, field)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(exists)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn hlen(&self, key: &str) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let len: u64 = conn
            .hlen(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(len)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl ListOperations for CacheClient {
    #[instrument(skip(self, values), fields(cache.key = %key))]
    async fn lpush<T: Serialize + Send + Sync>(&self, key: &str, values: &[T]) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let json_values: Vec<String> = values
            .iter()
            .map(|v| serde_json::to_string(v))
            .collect::<Result<_, _>>()
            .map_err(|e| CacheError::serialization(e.to_string()))?;

        let len: u64 = conn
            .lpush(&prefixed, &json_values)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::LPush);
        Ok(len)
    }

    #[instrument(skip(self, values), fields(cache.key = %key))]
    async fn rpush<T: Serialize + Send + Sync>(&self, key: &str, values: &[T]) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let json_values: Vec<String> = values
            .iter()
            .map(|v| serde_json::to_string(v))
            .collect::<Result<_, _>>()
            .map_err(|e| CacheError::serialization(e.to_string()))?;

        let len: u64 = conn
            .rpush(&prefixed, &json_values)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::RPush);
        Ok(len)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn lpop<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let result: Option<String> = conn
            .lpop(&prefixed, None)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        match result {
            Some(json) => {
                let value = serde_json::from_str(&json)
                    .map_err(|e| CacheError::deserialization(e.to_string()))?;
                self.metrics.record_operation(CacheOperation::LPop);
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn rpop<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let result: Option<String> = conn
            .rpop(&prefixed, None)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        match result {
            Some(json) => {
                let value = serde_json::from_str(&json)
                    .map_err(|e| CacheError::deserialization(e.to_string()))?;
                self.metrics.record_operation(CacheOperation::RPop);
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn lrange<T: DeserializeOwned + Send>(
        &self,
        key: &str,
        start: i64,
        stop: i64,
    ) -> CacheResult<Vec<T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let results: Vec<String> = conn
            .lrange(&prefixed, start as isize, stop as isize)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        let mut values = Vec::with_capacity(results.len());
        for json in results {
            let value = serde_json::from_str(&json)
                .map_err(|e| CacheError::deserialization(e.to_string()))?;
            values.push(value);
        }

        self.metrics.record_operation(CacheOperation::LRange);
        Ok(values)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn llen(&self, key: &str) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let len: u64 = conn
            .llen(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(len)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl SetOperations for CacheClient {
    #[instrument(skip(self, members), fields(cache.key = %key))]
    async fn sadd<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        members: &[T],
    ) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let json_members: Vec<String> = members
            .iter()
            .map(|m| serde_json::to_string(m))
            .collect::<Result<_, _>>()
            .map_err(|e| CacheError::serialization(e.to_string()))?;

        let added: u64 = conn
            .sadd(&prefixed, &json_members)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::SAdd);
        Ok(added)
    }

    #[instrument(skip(self, members), fields(cache.key = %key))]
    async fn srem<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        members: &[T],
    ) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let json_members: Vec<String> = members
            .iter()
            .map(|m| serde_json::to_string(m))
            .collect::<Result<_, _>>()
            .map_err(|e| CacheError::serialization(e.to_string()))?;

        let removed: u64 = conn
            .srem(&prefixed, &json_members)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        self.metrics.record_operation(CacheOperation::SRem);
        Ok(removed)
    }

    #[instrument(skip(self, member), fields(cache.key = %key))]
    async fn sismember<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        member: &T,
    ) -> CacheResult<bool> {
        let prefixed = self.prefixed_key(key);
        let json =
            serde_json::to_string(member).map_err(|e| CacheError::serialization(e.to_string()))?;

        let mut conn = self.get_connection().await?;

        let is_member: bool = conn
            .sismember(&prefixed, &json)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(is_member)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn smembers<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Vec<T>> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let results: Vec<String> = conn
            .smembers(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        let mut values = Vec::with_capacity(results.len());
        for json in results {
            let value = serde_json::from_str(&json)
                .map_err(|e| CacheError::deserialization(e.to_string()))?;
            values.push(value);
        }

        self.metrics.record_operation(CacheOperation::SMembers);
        Ok(values)
    }

    #[instrument(skip(self), fields(cache.key = %key))]
    async fn scard(&self, key: &str) -> CacheResult<u64> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.get_connection().await?;

        let count: u64 = conn
            .scard(&prefixed)
            .await
            .map_err(|e| CacheError::internal(e.to_string()))?;

        Ok(count)
    }
}
