/// SecretCache は秘密情報のインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きで秘密情報をキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::secret::Secret;

/// キャッシュキーは "path:version" 形式の文字列。
/// version が None の場合は "path:latest" を使用する。
pub struct SecretCache {
    inner: Cache<String, Arc<Secret>>,
}

impl SecretCache {
    /// 新しい SecretCache を作成する。
    ///
    /// # Arguments
    /// * `max_capacity` - キャッシュに保持する最大エントリ数
    /// * `ttl_secs` - エントリの有効期間（秒）
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }

    /// キャッシュキーを生成する。
    pub fn cache_key(path: &str, version: Option<i64>) -> String {
        match version {
            Some(v) => format!("{}:{}", path, v),
            None => format!("{}:latest", path),
        }
    }

    /// キャッシュからシークレットを取得する。
    pub async fn get(&self, path: &str, version: Option<i64>) -> Option<Arc<Secret>> {
        let cache_key = Self::cache_key(path, version);
        self.inner.get(&cache_key).await
    }

    /// シークレットをキャッシュに追加する。
    pub async fn insert(&self, path: &str, version: Option<i64>, secret: Arc<Secret>) {
        let cache_key = Self::cache_key(path, version);
        self.inner.insert(cache_key, secret).await;
    }

    /// 特定のパスに関連するすべてのキャッシュエントリを削除する。
    pub async fn invalidate(&self, path: &str) {
        let prefix = format!("{}:", path);
        let keys_to_remove: Vec<String> = self
            .inner
            .iter()
            .filter(|(k, _v)| k.starts_with(&prefix))
            .map(|(k, _v)| k.as_ref().clone())
            .collect();

        for key in keys_to_remove {
            self.inner.invalidate(&key).await;
        }
    }

    /// 特定のパスとバージョンのキャッシュエントリを削除する。
    pub async fn invalidate_version(&self, path: &str, version: Option<i64>) {
        let cache_key = Self::cache_key(path, version);
        self.inner.invalidate(&cache_key).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_secret(path: &str) -> Arc<Secret> {
        let data = HashMap::from([("key".to_string(), "value".to_string())]);
        Arc::new(Secret::new(path.to_string(), data))
    }

    #[tokio::test]
    async fn test_cache_key_format() {
        assert_eq!(
            SecretCache::cache_key("app/db/password", None),
            "app/db/password:latest"
        );
        assert_eq!(
            SecretCache::cache_key("app/db/password", Some(3)),
            "app/db/password:3"
        );
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let cache = SecretCache::new(100, 60);
        let secret = make_secret("app/db/password");

        cache.insert("app/db/password", None, secret.clone()).await;

        let result = cache.get("app/db/password", None).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "app/db/password");
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = SecretCache::new(100, 60);

        let result = cache.get("nonexistent", None).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_all_versions() {
        let cache = SecretCache::new(100, 60);
        let secret = make_secret("app/db/password");

        cache
            .insert("app/db/password", None, secret.clone())
            .await;
        cache
            .insert("app/db/password", Some(1), secret.clone())
            .await;
        cache
            .insert("app/db/password", Some(2), secret.clone())
            .await;

        cache.invalidate("app/db/password").await;

        assert!(cache.get("app/db/password", None).await.is_none());
        assert!(cache.get("app/db/password", Some(1)).await.is_none());
        assert!(cache.get("app/db/password", Some(2)).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_paths() {
        let cache = SecretCache::new(100, 60);
        let secret1 = make_secret("app/db/password");
        let secret2 = make_secret("app/api/key");

        cache.insert("app/db/password", None, secret1).await;
        cache.insert("app/api/key", None, secret2).await;

        cache.invalidate("app/db/password").await;

        assert!(cache.get("app/db/password", None).await.is_none());
        assert!(cache.get("app/api/key", None).await.is_some());
    }

    #[tokio::test]
    async fn test_invalidate_version() {
        let cache = SecretCache::new(100, 60);
        let secret = make_secret("app/db/password");

        cache
            .insert("app/db/password", None, secret.clone())
            .await;
        cache
            .insert("app/db/password", Some(1), secret.clone())
            .await;

        cache
            .invalidate_version("app/db/password", Some(1))
            .await;

        // latest はまだある
        assert!(cache.get("app/db/password", None).await.is_some());
        // version 1 は削除済み
        assert!(cache.get("app/db/password", Some(1)).await.is_none());
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = SecretCache::new(100, 1);
        let secret = make_secret("app/db/password");

        cache.insert("app/db/password", None, secret).await;
        assert!(cache.get("app/db/password", None).await.is_some());

        tokio::time::sleep(Duration::from_millis(1200)).await;

        assert!(cache.get("app/db/password", None).await.is_none());
    }
}
