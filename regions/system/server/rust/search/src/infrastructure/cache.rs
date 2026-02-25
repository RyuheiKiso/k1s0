/// IndexCache は検索インデックスのインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きで SearchIndex をキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::search_index::SearchIndex;

/// キャッシュキーはインデックス名の文字列。
pub struct IndexCache {
    inner: Cache<String, Arc<SearchIndex>>,
}

impl IndexCache {
    /// 新しい IndexCache を作成する。
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

    /// インデックス名に対応するエントリを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, name: &str) -> Option<Arc<SearchIndex>> {
        self.inner.get(name).await
    }

    /// エントリをキャッシュに追加する。
    /// キーは index.name から自動生成する。
    pub async fn insert(&self, index: Arc<SearchIndex>) {
        self.inner.insert(index.name.clone(), index).await;
    }

    /// 特定のインデックス名のエントリをキャッシュから削除する。
    pub async fn invalidate(&self, name: &str) {
        self.inner.invalidate(name).await;
    }

    /// すべてのキャッシュエントリを削除する。
    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
        // moka の invalidate_all は非同期ではないが、
        // エビクション通知をフラッシュするため run_pending_tasks を呼ぶ
        self.inner.run_pending_tasks().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_index(name: &str) -> Arc<SearchIndex> {
        Arc::new(SearchIndex {
            id: Uuid::new_v4(),
            name: name.to_string(),
            mapping: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        })
    }

    #[tokio::test]
    async fn test_insert_and_get_returns_entry() {
        let cache = IndexCache::new(100, 60);
        let index = make_index("products");

        cache.insert(index.clone()).await;

        let result = cache.get("products").await;
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.name, "products");
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = IndexCache::new(100, 60);

        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_entry() {
        let cache = IndexCache::new(100, 60);
        let index = make_index("products");
        cache.insert(index).await;

        assert!(cache.get("products").await.is_some());

        cache.invalidate("products").await;

        let result = cache.get("products").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_keys() {
        let cache = IndexCache::new(100, 60);
        let idx1 = make_index("products");
        let idx2 = make_index("users");
        cache.insert(idx1).await;
        cache.insert(idx2).await;

        cache.invalidate("products").await;

        assert!(cache.get("products").await.is_none());
        assert!(cache.get("users").await.is_some());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing_entry() {
        let cache = IndexCache::new(100, 60);

        let idx_v1 = Arc::new(SearchIndex {
            id: Uuid::new_v4(),
            name: "products".to_string(),
            mapping: serde_json::json!({"version": 1}),
            created_at: chrono::Utc::now(),
        });

        let idx_v2 = Arc::new(SearchIndex {
            id: Uuid::new_v4(),
            name: "products".to_string(),
            mapping: serde_json::json!({"version": 2}),
            created_at: chrono::Utc::now(),
        });

        cache.insert(idx_v1).await;
        cache.insert(idx_v2).await;

        let result = cache.get("products").await.unwrap();
        assert_eq!(result.mapping, serde_json::json!({"version": 2}));
    }

    #[tokio::test]
    async fn test_invalidate_all() {
        let cache = IndexCache::new(100, 60);
        cache.insert(make_index("products")).await;
        cache.insert(make_index("users")).await;

        cache.invalidate_all().await;

        assert!(cache.get("products").await.is_none());
        assert!(cache.get("users").await.is_none());
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = IndexCache::new(100, 1);
        let index = make_index("products");
        cache.insert(index).await;

        // TTL 内は取得できる
        assert!(cache.get("products").await.is_some());

        // TTL を超えるまで待機
        tokio::time::sleep(Duration::from_millis(1200)).await;

        // TTL 超過後はエントリが消えている
        let result = cache.get("products").await;
        assert!(result.is_none());
    }
}
