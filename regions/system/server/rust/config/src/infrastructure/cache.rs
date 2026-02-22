/// ConfigCache は設定値のインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きで設定値をキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::config_entry::ConfigEntry;

/// キャッシュキーは "namespace:key" 形式の文字列。
pub struct ConfigCache {
    inner: Cache<String, Arc<ConfigEntry>>,
}

impl ConfigCache {
    /// 新しい ConfigCache を作成する。
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

    /// "namespace:key" 形式のキャッシュキーを生成する。
    pub fn cache_key(namespace: &str, key: &str) -> String {
        format!("{}:{}", namespace, key)
    }

    /// namespace と key に対応するエントリを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, namespace: &str, key: &str) -> Option<Arc<ConfigEntry>> {
        let cache_key = Self::cache_key(namespace, key);
        self.inner.get(&cache_key).await
    }

    /// エントリをキャッシュに追加する。
    /// キーは entry.namespace と entry.key から自動生成する。
    pub async fn insert(&self, entry: Arc<ConfigEntry>) {
        let cache_key = Self::cache_key(&entry.namespace, &entry.key);
        self.inner.insert(cache_key, entry).await;
    }

    /// 特定の namespace と key のエントリをキャッシュから削除する。
    pub async fn invalidate(&self, namespace: &str, key: &str) {
        let cache_key = Self::cache_key(namespace, key);
        self.inner.invalidate(&cache_key).await;
    }

    /// 指定した namespace に属するすべてのエントリをキャッシュから削除する。
    /// moka v0.12 の `invalidate_entries_if` は `invalidation_closures` feature が
    /// 必要なため、iter() でキーを収集してから個別に invalidate する。
    pub async fn invalidate_namespace(&self, namespace: &str) {
        let prefix = format!("{}:", namespace);
        // キャッシュの全キーをイテレートして対象を収集
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_entry(namespace: &str, key: &str) -> Arc<ConfigEntry> {
        Arc::new(ConfigEntry {
            id: Uuid::new_v4(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            value_json: serde_json::json!(42),
            version: 1,
            description: None,
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn test_cache_key_format() {
        let key = ConfigCache::cache_key("system.auth.database", "max_connections");
        assert_eq!(key, "system.auth.database:max_connections");
    }

    #[tokio::test]
    async fn test_insert_and_get_returns_entry() {
        let cache = ConfigCache::new(100, 60);
        let entry = make_entry("system.auth.database", "max_connections");

        cache.insert(entry.clone()).await;

        let result = cache.get("system.auth.database", "max_connections").await;
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.namespace, "system.auth.database");
        assert_eq!(cached.key, "max_connections");
        assert_eq!(cached.value_json, serde_json::json!(42));
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = ConfigCache::new(100, 60);

        let result = cache.get("nonexistent.namespace", "missing_key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_entry() {
        let cache = ConfigCache::new(100, 60);
        let entry = make_entry("system.auth.database", "max_connections");
        cache.insert(entry).await;

        // 削除前は取得できる
        assert!(cache
            .get("system.auth.database", "max_connections")
            .await
            .is_some());

        cache
            .invalidate("system.auth.database", "max_connections")
            .await;

        // 削除後は取得できない
        let result = cache.get("system.auth.database", "max_connections").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_keys() {
        let cache = ConfigCache::new(100, 60);
        let entry1 = make_entry("system.auth.database", "max_connections");
        let entry2 = make_entry("system.auth.database", "ssl_mode");
        cache.insert(entry1).await;
        cache.insert(entry2).await;

        cache
            .invalidate("system.auth.database", "max_connections")
            .await;

        // max_connections は削除済み
        assert!(cache
            .get("system.auth.database", "max_connections")
            .await
            .is_none());
        // ssl_mode は残っている
        assert!(cache
            .get("system.auth.database", "ssl_mode")
            .await
            .is_some());
    }

    #[tokio::test]
    async fn test_invalidate_namespace_removes_all_entries_in_namespace() {
        let cache = ConfigCache::new(100, 60);
        let entry1 = make_entry("system.auth.database", "max_connections");
        let entry2 = make_entry("system.auth.database", "ssl_mode");
        let entry3 = make_entry("system.auth.jwt", "secret_key");
        cache.insert(entry1).await;
        cache.insert(entry2).await;
        cache.insert(entry3).await;

        cache.invalidate_namespace("system.auth.database").await;

        // system.auth.database の全エントリは削除済み
        assert!(cache
            .get("system.auth.database", "max_connections")
            .await
            .is_none());
        assert!(cache
            .get("system.auth.database", "ssl_mode")
            .await
            .is_none());
        // system.auth.jwt は残っている
        assert!(cache.get("system.auth.jwt", "secret_key").await.is_some());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing_entry() {
        let cache = ConfigCache::new(100, 60);

        let entry_v1 = Arc::new(ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 1,
            description: None,
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let entry_v2 = Arc::new(ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(50),
            version: 2,
            description: None,
            created_by: "admin@example.com".to_string(),
            updated_by: "operator@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        cache.insert(entry_v1).await;
        cache.insert(entry_v2).await;

        let result = cache
            .get("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert_eq!(result.value_json, serde_json::json!(50));
        assert_eq!(result.version, 2);
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        // TTL 1秒のキャッシュで、1秒以上待機後にエントリが消えることを確認
        let cache = ConfigCache::new(100, 1);
        let entry = make_entry("system.auth.database", "max_connections");
        cache.insert(entry).await;

        // TTL 内は取得できる
        assert!(cache
            .get("system.auth.database", "max_connections")
            .await
            .is_some());

        // TTL を超えるまで待機
        tokio::time::sleep(Duration::from_millis(1200)).await;

        // TTL 超過後はエントリが消えている
        let result = cache.get("system.auth.database", "max_connections").await;
        assert!(result.is_none());
    }
}
