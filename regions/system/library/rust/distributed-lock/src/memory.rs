use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::lock::{DistributedLock, LockGuard};
use crate::LockError;

struct LockEntry {
    token: String,
    expires_at: Instant,
}

impl LockEntry {
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

pub struct InMemoryDistributedLock {
    locks: Mutex<HashMap<String, LockEntry>>,
}

impl InMemoryDistributedLock {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryDistributedLock {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DistributedLock for InMemoryDistributedLock {
    async fn acquire(&self, key: &str, ttl: Duration) -> Result<LockGuard, LockError> {
        let mut locks = self.locks.lock().await;
        if let Some(entry) = locks.get(key) {
            if !entry.is_expired() {
                return Err(LockError::AlreadyLocked(key.to_string()));
            }
        }
        let token = Uuid::new_v4().to_string();
        locks.insert(
            key.to_string(),
            LockEntry {
                token: token.clone(),
                expires_at: Instant::now() + ttl,
            },
        );
        Ok(LockGuard {
            key: key.to_string(),
            token,
        })
    }

    async fn release(&self, guard: LockGuard) -> Result<(), LockError> {
        let mut locks = self.locks.lock().await;
        match locks.get(&guard.key) {
            Some(entry) if entry.token == guard.token => {
                locks.remove(&guard.key);
                Ok(())
            }
            Some(_) => Err(LockError::TokenMismatch),
            None => Err(LockError::LockNotFound(guard.key)),
        }
    }

    async fn extend(&self, guard: &LockGuard, ttl: Duration) -> Result<(), LockError> {
        let mut locks = self.locks.lock().await;
        match locks.get_mut(&guard.key) {
            Some(entry) if entry.token == guard.token => {
                entry.expires_at = Instant::now() + ttl;
                Ok(())
            }
            Some(_) => Err(LockError::TokenMismatch),
            None => Err(LockError::LockNotFound(guard.key.clone())),
        }
    }

    async fn is_locked(&self, key: &str) -> Result<bool, LockError> {
        let locks = self.locks.lock().await;
        Ok(locks.get(key).map_or(false, |e| !e.is_expired()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_and_release() {
        let lock = InMemoryDistributedLock::new();
        let guard = lock.acquire("key1", Duration::from_secs(10)).await.unwrap();
        assert!(lock.is_locked("key1").await.unwrap());

        lock.release(guard).await.unwrap();
        assert!(!lock.is_locked("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_double_acquire_returns_already_locked() {
        let lock = InMemoryDistributedLock::new();
        let _guard = lock.acquire("key1", Duration::from_secs(10)).await.unwrap();
        let result = lock.acquire("key1", Duration::from_secs(10)).await;
        assert!(matches!(result, Err(LockError::AlreadyLocked(_))));
    }

    #[tokio::test]
    async fn test_release_with_wrong_token_returns_token_mismatch() {
        let lock = InMemoryDistributedLock::new();
        let _guard = lock.acquire("key1", Duration::from_secs(10)).await.unwrap();
        let fake_guard = LockGuard {
            key: "key1".to_string(),
            token: "wrong-token".to_string(),
        };
        let result = lock.release(fake_guard).await;
        assert!(matches!(result, Err(LockError::TokenMismatch)));
    }

    #[tokio::test]
    async fn test_extend_updates_ttl() {
        let lock = InMemoryDistributedLock::new();
        let guard = lock.acquire("key1", Duration::from_secs(1)).await.unwrap();
        lock.extend(&guard, Duration::from_secs(60)).await.unwrap();
        assert!(lock.is_locked("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_acquire_after_expiry() {
        let lock = InMemoryDistributedLock::new();
        let _guard = lock.acquire("key1", Duration::from_millis(1)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!lock.is_locked("key1").await.unwrap());
        let _guard2 = lock.acquire("key1", Duration::from_secs(10)).await.unwrap();
        assert!(lock.is_locked("key1").await.unwrap());
    }
}
