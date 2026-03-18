/// PolicyCache はポリシーのインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きでポリシーをキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::policy::Policy;

/// キャッシュキーは UUID 文字列。
pub struct PolicyCache {
    inner: Cache<String, Arc<Policy>>,
}

impl PolicyCache {
    /// 新しい PolicyCache を作成する。
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

    /// ID に対応するポリシーを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, id: &uuid::Uuid) -> Option<Arc<Policy>> {
        self.inner.get(&id.to_string()).await
    }

    /// ポリシーをキャッシュに追加する。
    pub async fn insert(&self, policy: Arc<Policy>) {
        self.inner.insert(policy.id.to_string(), policy).await;
    }

    /// 特定の ID のポリシーをキャッシュから削除する。
    pub async fn invalidate(&self, id: &uuid::Uuid) {
        self.inner.invalidate(&id.to_string()).await;
    }

    /// すべてのキャッシュエントリを削除する。
    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
        // run_pending_tasks は moka 内部のクリーンアップをトリガーする
        self.inner.run_pending_tasks().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_policy(name: &str) -> Arc<Policy> {
        Arc::new(Policy {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: "test".to_string(),
            rego_content: "package test".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let cache = PolicyCache::new(100, 60);
        let policy = make_policy("allow-read");

        cache.insert(policy.clone()).await;

        let result = cache.get(&policy.id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "allow-read");
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = PolicyCache::new(100, 60);
        let id = Uuid::new_v4();

        let result = cache.get(&id).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_entry() {
        let cache = PolicyCache::new(100, 60);
        let policy = make_policy("allow-read");
        let id = policy.id;

        cache.insert(policy).await;
        assert!(cache.get(&id).await.is_some());

        cache.invalidate(&id).await;
        assert!(cache.get(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_keys() {
        let cache = PolicyCache::new(100, 60);
        let p1 = make_policy("policy-1");
        let p2 = make_policy("policy-2");
        let id1 = p1.id;
        let id2 = p2.id;

        cache.insert(p1).await;
        cache.insert(p2).await;

        cache.invalidate(&id1).await;
        assert!(cache.get(&id1).await.is_none());
        assert!(cache.get(&id2).await.is_some());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing() {
        let cache = PolicyCache::new(100, 60);
        let id = Uuid::new_v4();

        let p1 = Arc::new(Policy {
            id,
            name: "v1".to_string(),
            description: "old".to_string(),
            rego_content: "package old".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let p2 = Arc::new(Policy {
            id,
            name: "v2".to_string(),
            description: "new".to_string(),
            rego_content: "package new".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 2,
            enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        cache.insert(p1).await;
        cache.insert(p2).await;

        let result = cache.get(&id).await.unwrap();
        assert_eq!(result.name, "v2");
        assert_eq!(result.version, 2);
    }

    #[tokio::test]
    async fn test_invalidate_all() {
        let cache = PolicyCache::new(100, 60);
        let p1 = make_policy("policy-1");
        let p2 = make_policy("policy-2");
        let id1 = p1.id;
        let id2 = p2.id;

        cache.insert(p1).await;
        cache.insert(p2).await;

        cache.invalidate_all().await;
        assert!(cache.get(&id1).await.is_none());
        assert!(cache.get(&id2).await.is_none());
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = PolicyCache::new(100, 1);
        let policy = make_policy("allow-read");
        let id = policy.id;

        cache.insert(policy).await;
        assert!(cache.get(&id).await.is_some());

        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert!(cache.get(&id).await.is_none());
    }
}
