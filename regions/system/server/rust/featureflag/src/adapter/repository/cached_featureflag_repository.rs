use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::repository::FeatureFlagRepository;
use crate::infrastructure::cache::FlagCache;

/// `CachedFeatureFlagRepository` は `FeatureFlagRepository` をキャッシュでラップする。
/// `find_by_key` でキャッシュヒット時はDBアクセスをスキップする。
/// update / delete / create 時はキャッシュを invalidate して整合性を保つ。
/// STATIC-CRITICAL-001 監査対応: 全メソッドに `tenant_id` パラメータを追加。
/// HIGH-005 対応: `tenant_id` は &str 型（migration 006 で DB の TEXT 型に変更済み）。
pub struct CachedFeatureFlagRepository {
    inner: Arc<dyn FeatureFlagRepository>,
    cache: Arc<FlagCache>,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl CachedFeatureFlagRepository {
    /// 新しい `CachedFeatureFlagRepository` を作成する。
    #[allow(dead_code)]
    pub fn new(inner: Arc<dyn FeatureFlagRepository>, cache: Arc<FlagCache>) -> Self {
        Self {
            inner,
            cache,
            metrics: None,
        }
    }

    /// メトリクス付きの `CachedFeatureFlagRepository` を作成する。
    pub fn with_metrics(
        inner: Arc<dyn FeatureFlagRepository>,
        cache: Arc<FlagCache>,
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
impl FeatureFlagRepository for CachedFeatureFlagRepository {
    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_key(&self, tenant_id: &str, flag_key: &str) -> anyhow::Result<FeatureFlag> {
        // テナントスコープでキャッシュヒット確認
        if let Some(cached) = self.cache.get(tenant_id, flag_key).await {
            if let Some(ref m) = self.metrics {
                m.record_cache_hit("feature_flags");
            }
            return Ok((*cached).clone());
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("feature_flags");
        }

        // キャッシュミス: DBから取得
        let flag = self.inner.find_by_key(tenant_id, flag_key).await?;

        // キャッシュに格納（flag.tenant_id + flag.flag_key がキー）
        self.cache.insert(Arc::new(flag.clone())).await;

        Ok(flag)
    }

    /// `find_all` はキャッシュを使わず inner に委譲する。
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<FeatureFlag>> {
        self.inner.find_all(tenant_id).await
    }

    /// create は inner に委譲する（新規作成なのでキャッシュ invalidate は不要）。
    async fn create(&self, tenant_id: &str, flag: &FeatureFlag) -> anyhow::Result<()> {
        self.inner.create(tenant_id, flag).await
    }

    /// update は inner に委譲し、成功時にキャッシュを invalidate する。
    async fn update(&self, tenant_id: &str, flag: &FeatureFlag) -> anyhow::Result<()> {
        self.inner.update(tenant_id, flag).await?;

        // 更新成功時はキャッシュを invalidate して古い値を除去
        self.cache.invalidate(tenant_id, &flag.flag_key).await;

        Ok(())
    }

    /// delete は inner に委譲し、成功時はキャッシュ全体を invalidate する。
    /// （ID から `flag_key` への逆引きがキャッシュにないため `invalidate_all` を使用）
    async fn delete(&self, tenant_id: &str, id: &Uuid) -> anyhow::Result<bool> {
        let deleted = self.inner.delete(tenant_id, id).await?;

        if deleted {
            // ID から flag_key が分からないため、関連エントリを確実に除去するために全クリア
            self.cache.invalidate_all().await;
        }

        Ok(deleted)
    }

    /// `exists_by_key` はキャッシュを使わず inner に委譲する。
    async fn exists_by_key(&self, tenant_id: &str, flag_key: &str) -> anyhow::Result<bool> {
        // テナントスコープでキャッシュに存在する場合は true を即返却
        if self.cache.get(tenant_id, flag_key).await.is_some() {
            return Ok(true);
        }
        self.inner.exists_by_key(tenant_id, flag_key).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use chrono::Utc;
    use uuid::Uuid;

    /// システムテナント文字列: テスト共通（HIGH-005 対応: TEXT 型）
    fn system_tenant() -> &'static str {
        "00000000-0000-0000-0000-000000000001"
    }

    fn make_flag(flag_key: &str, enabled: bool) -> FeatureFlag {
        FeatureFlag {
            id: Uuid::new_v4(),
            tenant_id: system_tenant().to_string(),
            flag_key: flag_key.to_string(),
            description: format!("Test flag: {}", flag_key),
            enabled,
            variants: vec![],
            rules: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_cache() -> Arc<FlagCache> {
        Arc::new(FlagCache::new(100, 60))
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    #[tokio::test]
    async fn test_cache_hit_skips_db() {
        let mut mock = MockFeatureFlagRepository::new();
        // find_by_key が呼ばれてはいけない
        mock.expect_find_by_key().never();

        let cache = make_cache();
        let flag = make_flag("feature.dark-mode", true);
        // 事前にキャッシュにフラグを挿入
        cache.insert(Arc::new(flag.clone())).await;

        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache);
        let result = repo
            .find_by_key(system_tenant(), "feature.dark-mode")
            .await
            .unwrap();

        assert_eq!(result.flag_key, "feature.dark-mode");
        assert!(result.enabled);
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する。
    #[tokio::test]
    async fn test_cache_miss_then_store() {
        let flag = make_flag("feature.dark-mode", true);
        let flag_clone = flag.clone();

        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "feature.dark-mode")
            .once()
            .returning(move |_, _| Ok(flag_clone.clone()));

        let cache = make_cache();
        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache.clone());

        // 1回目: キャッシュミス → DBから取得
        let result = repo
            .find_by_key(system_tenant(), "feature.dark-mode")
            .await
            .unwrap();
        assert_eq!(result.flag_key, "feature.dark-mode");
        assert!(result.enabled);

        // キャッシュにフラグが格納されていることを確認
        let cached = cache.get(system_tenant(), "feature.dark-mode").await;
        assert!(cached.is_some());
        assert!(cached.unwrap().enabled);
    }

    /// update 後にキャッシュが invalidate される。
    #[tokio::test]
    async fn test_update_invalidates_cache() {
        let flag = make_flag("feature.dark-mode", true);

        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_update().once().returning(|_, _| Ok(()));

        let cache = make_cache();
        // 事前にキャッシュにフラグを挿入
        cache.insert(Arc::new(flag.clone())).await;

        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache.clone());

        // update 実行
        let updated_flag = make_flag("feature.dark-mode", false);
        repo.update(system_tenant(), &updated_flag).await.unwrap();

        // キャッシュから古いフラグが invalidate されていることを確認
        let cached = cache.get(system_tenant(), "feature.dark-mode").await;
        assert!(
            cached.is_none(),
            "update 後はキャッシュが invalidate されるべき"
        );
    }

    /// delete 後にキャッシュが invalidate される。
    #[tokio::test]
    async fn test_delete_invalidates_cache() {
        let flag = make_flag("feature.dark-mode", true);
        let flag_id = flag.id;

        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_delete().once().returning(|_, _| Ok(true));

        let cache = make_cache();
        cache.insert(Arc::new(flag)).await;

        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache.clone());

        let deleted = repo.delete(system_tenant(), &flag_id).await.unwrap();
        assert!(deleted);

        // キャッシュから削除されていることを確認
        let cached = cache.get(system_tenant(), "feature.dark-mode").await;
        assert!(
            cached.is_none(),
            "delete 後はキャッシュが invalidate されるべき"
        );
    }

    /// delete が false を返したときはキャッシュを invalidate しない。
    #[tokio::test]
    async fn test_delete_not_found_does_not_invalidate_cache() {
        let flag = make_flag("feature.dark-mode", true);

        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_delete().once().returning(|_, _| Ok(false)); // 対象なし

        let cache = make_cache();
        cache.insert(Arc::new(flag)).await;

        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache.clone());

        let deleted = repo.delete(system_tenant(), &Uuid::new_v4()).await.unwrap();
        assert!(!deleted);

        // delete=false のときはキャッシュはそのまま残る
        let cached = cache.get(system_tenant(), "feature.dark-mode").await;
        assert!(
            cached.is_some(),
            "削除対象なしのときはキャッシュを保持すべき"
        );
    }

    /// exists_by_key はキャッシュにある場合 true を即返却する。
    #[tokio::test]
    async fn test_exists_by_key_cache_hit() {
        let mut mock = MockFeatureFlagRepository::new();
        // exists_by_key が呼ばれてはいけない
        mock.expect_exists_by_key().never();

        let cache = make_cache();
        let flag = make_flag("feature.dark-mode", true);
        cache.insert(Arc::new(flag)).await;

        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache);
        let exists = repo
            .exists_by_key(system_tenant(), "feature.dark-mode")
            .await
            .unwrap();
        assert!(exists);
    }

    /// exists_by_key はキャッシュにない場合 inner に委譲する。
    #[tokio::test]
    async fn test_exists_by_key_cache_miss_delegates() {
        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_exists_by_key()
            .withf(|_tid, key| key == "feature.new-ui")
            .once()
            .returning(|_, _| Ok(false));

        let cache = make_cache();
        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache);
        let exists = repo
            .exists_by_key(system_tenant(), "feature.new-ui")
            .await
            .unwrap();
        assert!(!exists);
    }

    /// find_all は常に inner に委譲する。
    #[tokio::test]
    async fn test_find_all_delegates_to_inner() {
        let flag = make_flag("feature.dark-mode", true);
        let flag_clone = flag.clone();

        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む1引数シグネチャ
        mock.expect_find_all()
            .once()
            .returning(move |_| Ok(vec![flag_clone.clone()]));

        let cache = make_cache();
        let repo = CachedFeatureFlagRepository::new(Arc::new(mock), cache);
        let flags = repo.find_all(system_tenant()).await.unwrap();
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].flag_key, "feature.dark-mode");
    }
}
