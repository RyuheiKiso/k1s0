//! キャッシュ実装

use std::collections::HashMap;
use std::future::Future;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::domain::{ConfigError, Setting};

/// キャッシュエントリ
struct CacheEntry {
    setting: Setting,
    expires_at: Instant,
}

/// 設定キャッシュトレイト
pub trait SettingCache: Send + Sync {
    /// キャッシュから取得
    fn get(&self, key: &str) -> impl Future<Output = Option<Setting>> + Send;

    /// キャッシュに保存
    fn set(&self, key: &str, setting: &Setting) -> impl Future<Output = Result<(), ConfigError>> + Send;

    /// キャッシュから削除
    fn delete(&self, key: &str) -> impl Future<Output = Result<bool, ConfigError>> + Send;

    /// キャッシュをクリア
    fn clear(&self) -> impl Future<Output = Result<(), ConfigError>> + Send;
}

/// インメモリキャッシュ
pub struct InMemoryCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

impl InMemoryCache {
    /// 新しいキャッシュを作成（デフォルトTTL: 5分）
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(300))
    }

    /// TTL指定でキャッシュを作成
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// キャッシュサイズを取得
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// 期限切れエントリを削除
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();
        entries.retain(|_, entry| entry.expires_at > now);
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingCache for InMemoryCache {
    async fn get(&self, key: &str) -> Option<Setting> {
        let entries = self.entries.read().unwrap();
        entries.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.setting.clone())
            } else {
                None
            }
        })
    }

    async fn set(&self, key: &str, setting: &Setting) -> Result<(), ConfigError> {
        let entry = CacheEntry {
            setting: setting.clone(),
            expires_at: Instant::now() + self.ttl,
        };
        let mut entries = self.entries.write().unwrap();
        entries.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, ConfigError> {
        let mut entries = self.entries.write().unwrap();
        Ok(entries.remove(key).is_some())
    }

    async fn clear(&self) -> Result<(), ConfigError> {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = InMemoryCache::new();

        let setting = Setting::new(1, "service", "dev", "key", "value");
        cache.set("test-key", &setting).await.unwrap();

        let result = cache.get("test-key").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "value");
    }

    #[tokio::test]
    async fn test_cache_get_not_found() {
        let cache = InMemoryCache::new();

        let result = cache.get("unknown").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = InMemoryCache::new();

        let setting = Setting::new(1, "service", "dev", "key", "value");
        cache.set("test-key", &setting).await.unwrap();

        let deleted = cache.delete("test-key").await.unwrap();
        assert!(deleted);

        let result = cache.get("test-key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = InMemoryCache::new();

        let setting = Setting::new(1, "service", "dev", "key", "value");
        cache.set("key1", &setting).await.unwrap();
        cache.set("key2", &setting).await.unwrap();

        assert_eq!(cache.len(), 2);

        cache.clear().await.unwrap();
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = InMemoryCache::with_ttl(Duration::from_millis(10));

        let setting = Setting::new(1, "service", "dev", "key", "value");
        cache.set("test-key", &setting).await.unwrap();

        // キャッシュに存在する
        assert!(cache.get("test-key").await.is_some());

        // 期限切れを待つ
        tokio::time::sleep(Duration::from_millis(20)).await;

        // 期限切れで取得できない
        assert!(cache.get("test-key").await.is_none());
    }
}
