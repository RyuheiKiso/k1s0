/// FlagCache はフィーチャーフラグのインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きでフラグをキャッシュする。
/// STATIC-CRITICAL-001 監査対応: キャッシュキーを "{tenant_id}:{flag_key}" 形式にして
/// テナント間のキャッシュ混在を防ぐ。
/// HIGH-005 対応: tenant_id を &str 型で受け取る（migration 006 で TEXT 型に変更済み）。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::feature_flag::FeatureFlag;

/// キャッシュキーは "{tenant_id}:{flag_key}" 形式。
pub struct FlagCache {
    inner: Cache<String, Arc<FeatureFlag>>,
}

impl FlagCache {
    /// 新しい FlagCache を作成する。
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

    /// テナントスコープのキャッシュキーを生成する。
    /// HIGH-005 対応: tenant_id は &str 型（DB の TEXT 型に対応）。
    fn cache_key(tenant_id: &str, flag_key: &str) -> String {
        format!("{tenant_id}:{flag_key}")
    }

    /// tenant_id + flag_key に対応するフラグを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, tenant_id: &str, flag_key: &str) -> Option<Arc<FeatureFlag>> {
        self.inner.get(&Self::cache_key(tenant_id, flag_key)).await
    }

    /// フラグをキャッシュに追加する。
    /// キーは flag.tenant_id + flag.flag_key から自動生成する。
    pub async fn insert(&self, flag: Arc<FeatureFlag>) {
        let key = Self::cache_key(&flag.tenant_id, &flag.flag_key);
        self.inner.insert(key, flag).await;
    }

    /// 特定の tenant_id + flag_key のフラグをキャッシュから削除する。
    pub async fn invalidate(&self, tenant_id: &str, flag_key: &str) {
        self.inner
            .invalidate(&Self::cache_key(tenant_id, flag_key))
            .await;
    }

    /// すべてのキャッシュエントリを削除する。
    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    /// システムテナント文字列: テスト共通（HIGH-005 対応: TEXT 型）
    fn system_tenant() -> &'static str {
        "00000000-0000-0000-0000-000000000001"
    }

    fn make_flag(tenant_id: &str, flag_key: &str, enabled: bool) -> Arc<FeatureFlag> {
        Arc::new(FeatureFlag {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            flag_key: flag_key.to_string(),
            description: format!("Test flag: {}", flag_key),
            enabled,
            variants: vec![],
            rules: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn test_insert_and_get_returns_flag() {
        let cache = FlagCache::new(100, 60);
        let tid = system_tenant();
        let flag = make_flag(tid, "feature.dark-mode", true);

        cache.insert(flag.clone()).await;

        let result = cache.get(tid, "feature.dark-mode").await;
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.flag_key, "feature.dark-mode");
        assert!(cached.enabled);
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = FlagCache::new(100, 60);

        let result = cache.get(system_tenant(), "nonexistent-flag").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_flag() {
        let cache = FlagCache::new(100, 60);
        let tid = system_tenant();
        let flag = make_flag(tid, "feature.dark-mode", true);
        cache.insert(flag).await;

        // 削除前は取得できる
        assert!(cache.get(tid, "feature.dark-mode").await.is_some());

        cache.invalidate(tid, "feature.dark-mode").await;

        // 削除後は取得できない
        let result = cache.get(tid, "feature.dark-mode").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_keys() {
        let cache = FlagCache::new(100, 60);
        let tid = system_tenant();
        let flag1 = make_flag(tid, "feature.dark-mode", true);
        let flag2 = make_flag(tid, "feature.new-ui", false);
        cache.insert(flag1).await;
        cache.insert(flag2).await;

        cache.invalidate(tid, "feature.dark-mode").await;

        // dark-mode は削除済み
        assert!(cache.get(tid, "feature.dark-mode").await.is_none());
        // new-ui は残っている
        assert!(cache.get(tid, "feature.new-ui").await.is_some());
    }

    #[tokio::test]
    async fn test_invalidate_all_removes_everything() {
        let cache = FlagCache::new(100, 60);
        let tid = system_tenant();
        let flag1 = make_flag(tid, "feature.dark-mode", true);
        let flag2 = make_flag(tid, "feature.new-ui", false);
        cache.insert(flag1).await;
        cache.insert(flag2).await;

        cache.invalidate_all().await;

        assert!(cache.get(tid, "feature.dark-mode").await.is_none());
        assert!(cache.get(tid, "feature.new-ui").await.is_none());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing_flag() {
        let cache = FlagCache::new(100, 60);
        let tid = system_tenant();

        let flag_v1 = make_flag(tid, "feature.dark-mode", false);
        let flag_v2 = make_flag(tid, "feature.dark-mode", true);

        cache.insert(flag_v1).await;
        cache.insert(flag_v2).await;

        let result = cache.get(tid, "feature.dark-mode").await.unwrap();
        assert!(result.enabled, "最新のフラグで上書きされるべき");
    }

    /// 異なるテナントの同名フラグはキャッシュで混在しない。
    #[tokio::test]
    async fn test_tenant_isolation_in_cache() {
        let cache = FlagCache::new(100, 60);
        let tid1 = "00000000-0000-0000-0000-000000000001";
        let tid2 = "00000000-0000-0000-0000-000000000002";

        let flag1 = make_flag(tid1, "feature.dark-mode", true);
        let flag2 = make_flag(tid2, "feature.dark-mode", false);
        cache.insert(flag1).await;
        cache.insert(flag2).await;

        // テナント1のフラグは enabled
        let r1 = cache.get(tid1, "feature.dark-mode").await.unwrap();
        assert!(r1.enabled);

        // テナント2のフラグは disabled
        let r2 = cache.get(tid2, "feature.dark-mode").await.unwrap();
        assert!(!r2.enabled);
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        // TTL 1秒のキャッシュで、1秒以上待機後にエントリが消えることを確認
        let cache = FlagCache::new(100, 1);
        let tid = system_tenant();
        let flag = make_flag(tid, "feature.dark-mode", true);
        cache.insert(flag).await;

        // TTL 内は取得できる
        assert!(cache.get(tid, "feature.dark-mode").await.is_some());

        // TTL を超えるまで待機
        tokio::time::sleep(Duration::from_millis(1200)).await;

        // TTL 超過後はエントリが消えている
        let result = cache.get(tid, "feature.dark-mode").await;
        assert!(result.is_none());
    }
}
