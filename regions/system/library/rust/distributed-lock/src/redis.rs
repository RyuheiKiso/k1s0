use std::time::Duration;

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError, Script};

use crate::lock::{DistributedLock, LockGuard};
use crate::LockError;

/// Redis-backed distributed lock implementation using a simplified Redlock algorithm.
///
/// Uses atomic SET NX PX for acquisition and Lua scripts for safe release/extend,
/// ensuring that only the lock holder can modify the lock.
#[derive(Clone)]
pub struct RedisDistributedLock {
    conn: MultiplexedConnection,
    key_prefix: String,
}

impl RedisDistributedLock {
    /// Create a new RedisDistributedLock from a Redis URL.
    ///
    /// # Arguments
    /// * `url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    pub async fn new(url: &str) -> Result<Self, LockError> {
        let client = Client::open(url).map_err(map_redis_error)?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(map_redis_error)?;
        Ok(Self {
            conn,
            key_prefix: "lock".to_string(),
        })
    }

    /// Create a new RedisDistributedLock from an existing multiplexed connection.
    pub fn from_connection(conn: MultiplexedConnection) -> Self {
        Self {
            conn,
            key_prefix: "lock".to_string(),
        }
    }

    /// Set a custom key prefix for lock keys.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    fn lock_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

/// Lua script for safe lock release.
/// Only deletes the key if the stored value matches the token.
const RELEASE_SCRIPT: &str = r#"
if redis.call("get", KEYS[1]) == ARGV[1] then
    return redis.call("del", KEYS[1])
else
    return 0
end
"#;

/// Lua script for safe lock extension.
/// Only extends TTL if the stored value matches the token.
const EXTEND_SCRIPT: &str = r#"
if redis.call("get", KEYS[1]) == ARGV[1] then
    return redis.call("pexpire", KEYS[1], ARGV[2])
else
    return 0
end
"#;

#[async_trait]
impl DistributedLock for RedisDistributedLock {
    async fn acquire(&self, key: &str, ttl: Duration) -> Result<LockGuard, LockError> {
        let mut conn = self.conn.clone();
        let full_key = self.lock_key(key);
        let token = uuid::Uuid::new_v4().to_string();
        let millis = ttl.as_millis() as u64;

        // Atomic SET key value NX PX milliseconds
        let result: Option<String> = redis::cmd("SET")
            .arg(&full_key)
            .arg(&token)
            .arg("NX")
            .arg("PX")
            .arg(millis)
            .query_async(&mut conn)
            .await
            .map_err(map_redis_error)?;

        match result {
            Some(_) => Ok(LockGuard {
                key: key.to_string(),
                token,
            }),
            None => Err(LockError::AlreadyLocked(key.to_string())),
        }
    }

    async fn release(&self, guard: LockGuard) -> Result<(), LockError> {
        let mut conn = self.conn.clone();
        let full_key = self.lock_key(&guard.key);

        let script = Script::new(RELEASE_SCRIPT);
        let result: i64 = script
            .key(&full_key)
            .arg(&guard.token)
            .invoke_async(&mut conn)
            .await
            .map_err(map_redis_error)?;

        if result == 1 {
            Ok(())
        } else {
            Err(LockError::TokenMismatch)
        }
    }

    async fn extend(&self, guard: &LockGuard, ttl: Duration) -> Result<(), LockError> {
        let mut conn = self.conn.clone();
        let full_key = self.lock_key(&guard.key);
        let millis = ttl.as_millis() as u64;

        let script = Script::new(EXTEND_SCRIPT);
        let result: i64 = script
            .key(&full_key)
            .arg(&guard.token)
            .arg(millis)
            .invoke_async(&mut conn)
            .await
            .map_err(map_redis_error)?;

        if result == 1 {
            Ok(())
        } else {
            Err(LockError::TokenMismatch)
        }
    }

    async fn is_locked(&self, key: &str) -> Result<bool, LockError> {
        let mut conn = self.conn.clone();
        let full_key = self.lock_key(key);
        let exists: bool = conn.exists(&full_key).await.map_err(map_redis_error)?;
        Ok(exists)
    }
}

fn map_redis_error(err: RedisError) -> LockError {
    LockError::Internal(err.to_string())
}

/// Helper function to format lock keys (exposed for testing without Redis connection).
pub fn format_lock_key(prefix: &str, key: &str) -> String {
    format!("{}:{}", prefix, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_lock_key() {
        assert_eq!(format_lock_key("lock", "myresource"), "lock:myresource");
        assert_eq!(
            format_lock_key("myapp:lock", "resource"),
            "myapp:lock:resource"
        );
    }

    #[test]
    fn test_map_redis_error_to_lock_error() {
        let err = map_redis_error(RedisError::from((
            redis::ErrorKind::IoError,
            "connection refused",
        )));
        match err {
            LockError::Internal(msg) => {
                assert!(msg.contains("connection refused"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_release_script_contains_get_and_del() {
        assert!(RELEASE_SCRIPT.contains("redis.call(\"get\""));
        assert!(RELEASE_SCRIPT.contains("redis.call(\"del\""));
    }

    #[test]
    fn test_extend_script_contains_get_and_pexpire() {
        assert!(EXTEND_SCRIPT.contains("redis.call(\"get\""));
        assert!(EXTEND_SCRIPT.contains("redis.call(\"pexpire\""));
    }
}
