use std::time::Duration;

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};

use crate::{CacheClient, CacheError};

/// Redis-backed cache client implementation.
///
/// Uses a multiplexed connection for efficient concurrent access.
#[derive(Clone)]
pub struct RedisCacheClient {
    conn: MultiplexedConnection,
    key_prefix: Option<String>,
}

impl RedisCacheClient {
    /// Create a new RedisCacheClient from a Redis URL.
    ///
    /// # Arguments
    /// * `url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    pub async fn new(url: &str) -> Result<Self, CacheError> {
        let client = Client::open(url).map_err(map_redis_error)?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(map_redis_error)?;
        Ok(Self {
            conn,
            key_prefix: None,
        })
    }

    /// Create a new RedisCacheClient from an existing multiplexed connection.
    pub fn from_connection(conn: MultiplexedConnection) -> Self {
        Self {
            conn,
            key_prefix: None,
        }
    }

    /// Set a key prefix for namespace isolation.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    fn prefixed_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

#[async_trait]
impl CacheClient for RedisCacheClient {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.conn.clone();
        let full_key = self.prefixed_key(key);
        let result: Option<String> = conn.get(&full_key).await.map_err(map_redis_error)?;
        Ok(result)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        let full_key = self.prefixed_key(key);
        match ttl {
            Some(duration) => {
                // LOW-008: 安全な型変換（オーバーフロー防止）
                let millis = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
                if millis == 0 {
                    conn.set::<_, _, ()>(&full_key, value)
                        .await
                        .map_err(map_redis_error)?;
                } else {
                    conn.pset_ex::<_, _, ()>(&full_key, value, millis)
                        .await
                        .map_err(map_redis_error)?;
                }
            }
            None => {
                conn.set::<_, _, ()>(&full_key, value)
                    .await
                    .map_err(map_redis_error)?;
            }
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        let full_key = self.prefixed_key(key);
        let count: i64 = conn.del(&full_key).await.map_err(map_redis_error)?;
        Ok(count > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        let full_key = self.prefixed_key(key);
        let exists: bool = conn.exists(&full_key).await.map_err(map_redis_error)?;
        Ok(exists)
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        let full_key = self.prefixed_key(key);
        // LOW-008: 安全な型変換（オーバーフロー防止）
        let millis = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);

        // Use SET with NX and PX for atomic set-if-not-exists with TTL
        let result: Option<String> = redis::cmd("SET")
            .arg(&full_key)
            .arg(value)
            .arg("NX")
            .arg("PX")
            .arg(millis)
            .query_async(&mut conn)
            .await
            .map_err(map_redis_error)?;

        Ok(result.is_some())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        let full_key = self.prefixed_key(key);
        // LOW-008: 安全な型変換（オーバーフロー防止）
        let millis = i64::try_from(ttl.as_millis()).unwrap_or(i64::MAX);
        let result: bool = conn
            .pexpire(&full_key, millis)
            .await
            .map_err(map_redis_error)?;
        Ok(result)
    }
}

fn map_redis_error(err: RedisError) -> CacheError {
    CacheError::ConnectionError(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Redis エラーが CacheError::ConnectionError に正しく変換されることを確認する。
    #[test]
    fn test_map_redis_error() {
        let err = map_redis_error(RedisError::from((
            redis::ErrorKind::IoError,
            "connection refused",
        )));
        match err {
            CacheError::ConnectionError(msg) => {
                assert!(msg.contains("connection refused"));
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    // Integration tests requiring a running Redis instance should be placed
    // in tests/ directory or gated behind a feature flag.
    // The trait implementation is verified through the InMemoryCacheClient tests
    // (same interface), and the Redis-specific logic is tested via integration tests.
}
