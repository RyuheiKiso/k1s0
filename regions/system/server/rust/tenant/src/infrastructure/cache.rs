use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use tracing::debug;
use uuid::Uuid;

use crate::domain::entity::Tenant;
use crate::domain::repository::TenantRepository;

/// テナント検索結果のインメモリキャッシュ。
/// find_by_id / find_by_name の結果をキャッシュし、DB 負荷を軽減する。
/// create/update 操作時にキャッシュを無効化して整合性を保つ。
pub struct CachedTenantRepository {
    /// 委譲先の実際のリポジトリ実装
    inner: Arc<dyn TenantRepository>,
    /// UUID → Tenant のキャッシュ
    id_cache: Cache<Uuid, Arc<Tenant>>,
    /// name → Tenant のキャッシュ
    name_cache: Cache<String, Arc<Tenant>>,
}

/// テナントキャッシュの設定パラメータ
#[derive(Debug, Clone)]
pub struct TenantCacheConfig {
    /// キャッシュエントリの有効期限（秒）
    pub ttl_secs: u64,
    /// キャッシュの最大エントリ数
    pub max_entries: u64,
}

impl Default for TenantCacheConfig {
    /// デフォルト: TTL 60秒、最大100エントリ
    fn default() -> Self {
        Self {
            ttl_secs: 60,
            max_entries: 100,
        }
    }
}

impl CachedTenantRepository {
    /// キャッシュ付きリポジトリを生成する。
    /// inner: 実際のDB操作を行うリポジトリ
    /// config: キャッシュの設定（TTL、最大エントリ数）
    pub fn new(inner: Arc<dyn TenantRepository>, config: &TenantCacheConfig) -> Self {
        let id_cache = Cache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.ttl_secs))
            .build();
        let name_cache = Cache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.ttl_secs))
            .build();
        Self {
            inner,
            id_cache,
            name_cache,
        }
    }

    /// IDキャッシュとnameキャッシュの両方にテナントを格納する。
    async fn populate_caches(&self, tenant: &Tenant) {
        let arc_tenant = Arc::new(tenant.clone());
        self.id_cache.insert(tenant.id, arc_tenant.clone()).await;
        self.name_cache
            .insert(tenant.name.clone(), arc_tenant)
            .await;
    }

    /// 指定テナントに関連するキャッシュエントリを無効化する。
    /// create/update/delete 操作後に呼び出して整合性を保つ。
    async fn invalidate_tenant(&self, tenant: &Tenant) {
        debug!(tenant_id = %tenant.id, tenant_name = %tenant.name, "テナントキャッシュを無効化");
        self.id_cache.invalidate(&tenant.id).await;
        self.name_cache.invalidate(&tenant.name).await;
    }

    /// 全キャッシュエントリを無効化する。
    #[allow(dead_code)]
    pub async fn invalidate_all(&self) {
        debug!("全テナントキャッシュを無効化");
        self.id_cache.invalidate_all();
        self.name_cache.invalidate_all();
        self.id_cache.run_pending_tasks().await;
        self.name_cache.run_pending_tasks().await;
    }
}

#[async_trait]
impl TenantRepository for CachedTenantRepository {
    /// IDでテナントを検索する。キャッシュヒット時はDBアクセスを省略する。
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>> {
        // キャッシュヒット時はそのまま返す
        if let Some(cached) = self.id_cache.get(id).await {
            debug!(tenant_id = %id, "テナントキャッシュヒット (by id)");
            return Ok(Some((*cached).clone()));
        }

        // キャッシュミス時はDBから取得してキャッシュに格納
        let result = self.inner.find_by_id(id).await?;
        if let Some(ref tenant) = result {
            self.populate_caches(tenant).await;
        }
        Ok(result)
    }

    /// 名前でテナントを検索する。キャッシュヒット時はDBアクセスを省略する。
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>> {
        // キャッシュヒット時はそのまま返す
        if let Some(cached) = self.name_cache.get(name).await {
            debug!(tenant_name = %name, "テナントキャッシュヒット (by name)");
            return Ok(Some((*cached).clone()));
        }

        // キャッシュミス時はDBから取得してキャッシュに格納
        let result = self.inner.find_by_name(name).await?;
        if let Some(ref tenant) = result {
            self.populate_caches(tenant).await;
        }
        Ok(result)
    }

    /// テナント一覧を取得する。ページネーション付きのためキャッシュ対象外。
    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)> {
        self.inner.list(page, page_size).await
    }

    /// テナントを作成する。作成後にキャッシュを無効化して整合性を保つ。
    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let result = self.inner.create(tenant).await?;
        self.invalidate_tenant(tenant).await;
        Ok(result)
    }

    /// テナントを更新する。更新後にキャッシュを無効化して整合性を保つ。
    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let result = self.inner.update(tenant).await?;
        self.invalidate_tenant(tenant).await;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Plan, TenantStatus};
    use crate::domain::repository::MockTenantRepository;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// テスト用のテナントを生成するヘルパー
    fn make_tenant(id: Uuid, name: &str) -> Tenant {
        Tenant {
            id,
            name: name.to_string(),
            display_name: format!("Display {}", name),
            status: TenantStatus::Active,
            plan: Plan::Professional,
            owner_id: None,
            settings: serde_json::json!({}),
            keycloak_realm: None,
            db_schema: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_caches_result() {
        let id = Uuid::new_v4();
        let tenant = make_tenant(id, "acme");
        let tenant_clone = tenant.clone();

        // DB呼び出し回数を追跡する
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(Some(tenant_clone.clone()))
        });

        let config = TenantCacheConfig {
            ttl_secs: 60,
            max_entries: 100,
        };
        let cached_repo = CachedTenantRepository::new(Arc::new(mock), &config);

        // 1回目: キャッシュミス → DBアクセス
        let result1 = cached_repo
            .find_by_id(&id)
            .await
            .expect("find_by_id failed");
        assert!(result1.is_some());
        assert_eq!(result1.expect("should be some").name, "acme");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // 2回目: キャッシュヒット → DBアクセスなし
        let result2 = cached_repo
            .find_by_id(&id)
            .await
            .expect("find_by_id failed");
        assert!(result2.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // 呼び出し回数は増えない
    }

    #[tokio::test]
    async fn test_find_by_name_caches_result() {
        let id = Uuid::new_v4();
        let tenant = make_tenant(id, "beta");
        let tenant_clone = tenant.clone();

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_name().returning(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(Some(tenant_clone.clone()))
        });

        let config = TenantCacheConfig {
            ttl_secs: 60,
            max_entries: 100,
        };
        let cached_repo = CachedTenantRepository::new(Arc::new(mock), &config);

        // 1回目: キャッシュミス
        let result1 = cached_repo
            .find_by_name("beta")
            .await
            .expect("find_by_name failed");
        assert!(result1.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // 2回目: キャッシュヒット
        let result2 = cached_repo
            .find_by_name("beta")
            .await
            .expect("find_by_name failed");
        assert!(result2.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_create_invalidates_cache() {
        let id = Uuid::new_v4();
        let tenant = make_tenant(id, "gamma");
        let tenant_clone = tenant.clone();
        let tenant_for_create = tenant.clone();

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(Some(tenant_clone.clone()))
        });
        mock.expect_create().returning(|_| Ok(()));

        let config = TenantCacheConfig {
            ttl_secs: 60,
            max_entries: 100,
        };
        let cached_repo = CachedTenantRepository::new(Arc::new(mock), &config);

        // キャッシュを温める
        let _ = cached_repo.find_by_id(&id).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // create でキャッシュが無効化される
        cached_repo
            .create(&tenant_for_create)
            .await
            .expect("create failed");

        // 再取得時にDBアクセスが発生する（キャッシュ無効化の確認）
        let _ = cached_repo.find_by_id(&id).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_update_invalidates_cache() {
        let id = Uuid::new_v4();
        let tenant = make_tenant(id, "delta");
        let tenant_clone = tenant.clone();
        let tenant_for_update = tenant.clone();

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(Some(tenant_clone.clone()))
        });
        mock.expect_update().returning(|_| Ok(()));

        let config = TenantCacheConfig {
            ttl_secs: 60,
            max_entries: 100,
        };
        let cached_repo = CachedTenantRepository::new(Arc::new(mock), &config);

        // キャッシュを温める
        let _ = cached_repo.find_by_id(&id).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // update でキャッシュが無効化される
        cached_repo
            .update(&tenant_for_update)
            .await
            .expect("update failed");

        // 再取得時にDBアクセスが発生する
        let _ = cached_repo.find_by_id(&id).await;
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
