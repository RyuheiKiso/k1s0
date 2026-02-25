/// RuleCache はレートリミットルールのインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きでルール情報をキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::RateLimitRule;

/// キャッシュキーはルールのscope（name相当）。
pub struct RuleCache {
    inner: Cache<String, Arc<RateLimitRule>>,
}

impl RuleCache {
    /// 新しい RuleCache を作成する。
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

    /// scope（name相当）に対応するルールを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get_by_name(&self, name: &str) -> Option<Arc<RateLimitRule>> {
        self.inner.get(name).await
    }

    /// ID に対応するルールを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get_by_id(&self, id: &uuid::Uuid) -> Option<Arc<RateLimitRule>> {
        let key = format!("id:{}", id);
        self.inner.get(&key).await
    }

    /// ルールをキャッシュに追加する（scope と id の両方のキーで格納）。
    pub async fn insert(&self, rule: &RateLimitRule) {
        let arc_rule = Arc::new(rule.clone());
        self.inner
            .insert(rule.scope.clone(), arc_rule.clone())
            .await;
        self.inner
            .insert(format!("id:{}", rule.id), arc_rule)
            .await;
    }

    /// 特定の scope のルールをキャッシュから削除する。
    pub async fn invalidate_by_name(&self, name: &str) {
        self.inner.invalidate(name).await;
    }

    /// 特定の id のルールをキャッシュから削除する。
    pub async fn invalidate_by_id(&self, id: &uuid::Uuid) {
        let key = format!("id:{}", id);
        self.inner.invalidate(&key).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Algorithm, RateLimitRule};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_rule(scope: &str) -> RateLimitRule {
        RateLimitRule {
            id: Uuid::new_v4(),
            scope: scope.to_string(),
            identifier_pattern: format!("{}-pattern", scope),
            limit: 100,
            window_seconds: 60,
            algorithm: Algorithm::TokenBucket,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_insert_and_get_by_name() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("service");

        cache.insert(&rule).await;

        let result = cache.get_by_name("service").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().scope, "service");
    }

    #[tokio::test]
    async fn test_insert_and_get_by_id() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("service");
        let id = rule.id;

        cache.insert(&rule).await;

        let result = cache.get_by_id(&id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = RuleCache::new(100, 60);

        assert!(cache.get_by_name("nonexistent").await.is_none());
        assert!(cache.get_by_id(&Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_by_name() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("service");
        cache.insert(&rule).await;

        cache.invalidate_by_name("service").await;

        assert!(cache.get_by_name("service").await.is_none());
        // id キーはまだ残っている（個別に invalidate する必要がある）
    }

    #[tokio::test]
    async fn test_invalidate_by_id() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("service");
        let id = rule.id;
        cache.insert(&rule).await;

        cache.invalidate_by_id(&id).await;

        assert!(cache.get_by_id(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_rules() {
        let cache = RuleCache::new(100, 60);
        let rule1 = make_rule("service");
        let rule2 = make_rule("user");
        cache.insert(&rule1).await;
        cache.insert(&rule2).await;

        cache.invalidate_by_name("service").await;

        assert!(cache.get_by_name("service").await.is_none());
        assert!(cache.get_by_name("user").await.is_some());
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = RuleCache::new(100, 1);
        let rule = make_rule("service");
        cache.insert(&rule).await;

        assert!(cache.get_by_name("service").await.is_some());

        tokio::time::sleep(Duration::from_millis(1200)).await;

        assert!(cache.get_by_name("service").await.is_none());
    }
}
