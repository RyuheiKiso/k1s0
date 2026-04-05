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
    #[allow(dead_code)]
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

    /// CRIT-005 対応: キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<RateLimitRule> {
        // キャッシュヒット確認（テナント ID が一致する場合のみ返却する）
        if let Some(cached) = self.cache.get_by_id(id).await {
            if cached.tenant_id == tenant_id {
                if let Some(ref m) = self.metrics {
                    m.record_cache_hit("rate_limit_rules");
                }
                return Ok((*cached).clone());
            }
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("rate_limit_rules");
        }

        // キャッシュミス: DBから取得
        let rule = self.inner.find_by_id(id, tenant_id).await?;

        // キャッシュに格納
        self.cache.insert(&rule).await;

        Ok(rule)
    }

    /// CRIT-005 対応: キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_name(
        &self,
        name: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Option<RateLimitRule>> {
        // キャッシュヒット確認（テナント ID が一致する場合のみ返却する）
        if let Some(cached) = self.cache.get_by_name(name).await {
            if cached.tenant_id == tenant_id {
                if let Some(ref m) = self.metrics {
                    m.record_cache_hit("rate_limit_rules");
                }
                return Ok(Some((*cached).clone()));
            }
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("rate_limit_rules");
        }

        // キャッシュミス: DBから取得
        let result = self.inner.find_by_name(name, tenant_id).await?;

        // 取得できた場合はキャッシュに格納
        if let Some(ref rule) = result {
            self.cache.insert(rule).await;
        }

        Ok(result)
    }

    /// CRIT-005 対応: scope でルールを取得する（キャッシュなしで inner に委譲）。
    async fn find_by_scope(
        &self,
        scope: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<RateLimitRule>> {
        self.inner.find_by_scope(scope, tenant_id).await
    }

    /// CRIT-005 対応: 全ルールを取得する（inner に委譲）。
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<RateLimitRule>> {
        self.inner.find_all(tenant_id).await
    }

    /// CRIT-005 対応: ページネーションでルールを取得する（inner に委譲）。
    async fn find_page(
        &self,
        page: u32,
        page_size: u32,
        scope: Option<String>,
        enabled_only: bool,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<RateLimitRule>, u64)> {
        self.inner
            .find_page(page, page_size, scope, enabled_only, tenant_id)
            .await
    }

    /// ルールを更新し、キャッシュも更新する。
    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()> {
        self.inner.update(rule).await?;
        self.cache.insert(rule).await;
        Ok(())
    }

    /// CRIT-005 対応: ルールを削除し、キャッシュも無効化する。
    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool> {
        let deleted = self.inner.delete(id, tenant_id).await?;
        if deleted {
            self.cache.invalidate_by_id(id).await;
        }
        Ok(deleted)
    }

    /// PostgreSQL リポジトリではRedis状態のリセットは行わない（state_storeが担当）。
    async fn reset_state(&self, key: &str) -> anyhow::Result<()> {
        self.inner.reset_state(key).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::Algorithm;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;
    use chrono::Utc;

    fn make_rule(scope: &str) -> RateLimitRule {
        RateLimitRule {
            id: Uuid::new_v4(),
            name: scope.to_string(),
            scope: scope.to_string(),
            identifier_pattern: format!("{}-pattern", scope),
            limit: 100,
            window_seconds: 60,
            algorithm: Algorithm::TokenBucket,
            enabled: true,
            // CRIT-005 対応: テナント ID を含める。
            tenant_id: "tenant-a".to_string(),
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
        let rule = make_rule("service");
        cache.insert(&rule).await;

        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_name("service", "tenant-a").await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().scope, "service");
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する（find_by_id）。
    #[tokio::test]
    async fn test_find_by_id_cache_hit_skips_db() {
        let mut mock = MockRateLimitRepository::new();
        mock.expect_find_by_id().never();

        let cache = make_cache();
        let rule = make_rule("service");
        let id = rule.id;
        cache.insert(&rule).await;

        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_id(&id, "tenant-a").await.unwrap();

        assert_eq!(result.id, id);
        assert_eq!(result.scope, "service");
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する（find_by_name）。
    #[tokio::test]
    async fn test_find_by_name_cache_miss_then_store() {
        let rule = make_rule("service");
        let rule_clone = rule.clone();

        let mut mock = MockRateLimitRepository::new();
        mock.expect_find_by_name()
            .withf(|name, _tenant_id| name == "service")
            .once()
            .returning(move |_, _| Ok(Some(rule_clone.clone())));

        let cache = make_cache();
        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache.clone());

        let result = repo.find_by_name("service", "tenant-a").await.unwrap();
        assert!(result.is_some());

        // キャッシュに格納されていることを確認
        let cached = cache.get_by_name("service").await;
        assert!(cached.is_some());
    }

    /// create 後にキャッシュに格納される。
    #[tokio::test]
    async fn test_create_populates_cache() {
        let rule = make_rule("service");
        let rule_clone = rule.clone();

        let mut mock = MockRateLimitRepository::new();
        mock.expect_create()
            .once()
            .returning(move |_| Ok(rule_clone.clone()));

        let cache = make_cache();
        let repo = CachedRateLimitRepository::new(Arc::new(mock), cache.clone());

        let created = repo.create(&rule).await.unwrap();
        assert_eq!(created.scope, "service");

        // キャッシュに格納されていることを確認
        let cached = cache.get_by_name("service").await;
        assert!(cached.is_some());
    }
}
