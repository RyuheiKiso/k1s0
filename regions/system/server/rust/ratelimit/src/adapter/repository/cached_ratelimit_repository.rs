use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::RateLimitRule;
use crate::domain::repository::RateLimitRepository;
use crate::infrastructure::cache::RuleCache;

/// CachedRateLimitRepository は RateLimitRepository をキャッシュでラップする。
/// find_by_id / find_by_name でキャッシュヒット時はDBアクセスをスキップする。
/// create 時はキャッシュを invalidate して整合性を保つ。
pub struct CachedRateLimitRepository {
    inner: Arc<dyn RateLimitRepository>,
    cache: Arc<RuleCache>,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl CachedRateLimitRepository {
    /// 新しい CachedRateLimitRepository を作成する。
    pub fn new(inner: Arc<dyn RateLimitRepository>, cache: Arc<RuleCache>) -> Self {
        Self {
            inner,
            cache,
            metrics: None,
        }
    }

    /// メトリクス付きの CachedRateLimitRepository を作成する。
    pub fn with_metrics(
        inner: Arc<dyn RateLimitRepository>,
        cache: Arc<RuleCache>,
        metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    ) -> Self {
        Self {
            inner,
            cache,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl RateLimitRepository for CachedRateLimitRepository {
    /// create は inner に委譲し、成功時にキャッシュに格納する。
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule> {
        let created = self.inner.create(rule).await?;

        // 作成成功時はキャッシュに格納
        self.cache.insert(&created).await;

        Ok(created)
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<RateLimitRule> {
        // キャッシュヒット確認
        if let Some(cached) = self.cache.get_by_id(id).await {
            if let Some(ref m) = self.metrics {
                m.record_cache_hit("rate_limit_rules");
            }
            return Ok((*cached).clone());
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("rate_limit_rules");
        }

        // キャッシュミス: DBから取得
        let rule = self.inner.find_by_id(id).await?;

        // キャッシュに格納
        self.cache.insert(&rule).await;

        Ok(rule)
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<RateLimitRule>> {
        // キャッシュヒット確認
        if let Some(cached) = self.cache.get_by_name(name).await {
            if let Some(ref m) = self.metrics {
                m.record_cache_hit("rate_limit_rules");
            }
            return Ok(Some((*cached).clone()));
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("rate_limit_rules");
        }

        // キャッシュミス: DBから取得
        let result = self.inner.find_by_name(name).await?;

        // 取得できた場合はキャッシュに格納
        if let Some(ref rule) = result {
            self.cache.insert(rule).await;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Algorithm;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;
    use chrono::Utc;

    fn make_rule(name: &str) -> RateLimitRule {
        RateLimitRule {
            id: Uuid::new_v4(),
            name: name.to_string(),
            key: format!("{}-key", name),
            limit: 100,
            window_secs: 60,
            algorithm: Algorithm::TokenBucket,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_cache() -> Arc<RuleCache> {
        Arc::new(RuleCache::new(100, 60))
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する（find_by_name）。
    #[tokio::test]
    async fn test_find_by_name_cache_hit_skips_db() {
        let mut mock = MockRateLimitRepository::new();
        mock.expect_find_by_name().never();

        let cache = make_cache();
        let rule = make_rule("api-global");
        cache.insert(&rule).await;

        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_name("api-global").await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "api-global");
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する（find_by_id）。
    #[tokio::test]
    async fn test_find_by_id_cache_hit_skips_db() {
        let mut mock = MockRateLimitRepository::new();
        mock.expect_find_by_id().never();

        let cache = make_cache();
        let rule = make_rule("api-global");
        let id = rule.id;
        cache.insert(&rule).await;

        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_id(&id).await.unwrap();

        assert_eq!(result.id, id);
        assert_eq!(result.name, "api-global");
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する（find_by_name）。
    #[tokio::test]
    async fn test_find_by_name_cache_miss_then_store() {
        let rule = make_rule("api-global");
        let rule_clone = rule.clone();

        let mut mock = MockRateLimitRepository::new();
        mock.expect_find_by_name()
            .withf(|name| name == "api-global")
            .once()
            .returning(move |_| Ok(Some(rule_clone.clone())));

        let cache = make_cache();
        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache.clone());

        let result = repo.find_by_name("api-global").await.unwrap();
        assert!(result.is_some());

        // キャッシュに格納されていることを確認
        let cached = cache.get_by_name("api-global").await;
        assert!(cached.is_some());
    }

    /// create 後にキャッシュに格納される。
    #[tokio::test]
    async fn test_create_populates_cache() {
        let rule = make_rule("api-global");
        let rule_clone = rule.clone();

        let mut mock = MockRateLimitRepository::new();
        mock.expect_create()
            .once()
            .returning(move |_| Ok(rule_clone.clone()));

        let cache = make_cache();
        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache.clone());

        let created = repo.create(&rule).await.unwrap();
        assert_eq!(created.name, "api-global");

        // キャッシュに格納されていることを確認
        let cached = cache.get_by_name("api-global").await;
        assert!(cached.is_some());
    }
}
