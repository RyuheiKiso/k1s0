use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::api_registration::ApiSchema;
use crate::domain::repository::ApiSchemaRepository;
use crate::infrastructure::cache::SchemaCache;

/// `CachedSchemaRepository` は `ApiSchemaRepository` をキャッシュでラップする。
/// `find_by_name` でキャッシュヒット時はDBアクセスをスキップする。
/// DB操作を先行させてからキャッシュを invalidate することで整合性を保つ。
/// キャッシュ invalidation は DB 操作成功後にのみ行い、
/// 「DB 更新成功 → キャッシュ破棄 → 次回アクセス時に最新値をDB取得」の順序を保証する。
pub struct CachedSchemaRepository {
    inner: Arc<dyn ApiSchemaRepository>,
    cache: Arc<SchemaCache>,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl CachedSchemaRepository {
    /// 新しい `CachedSchemaRepository` を作成する。
    pub fn new(inner: Arc<dyn ApiSchemaRepository>, cache: Arc<SchemaCache>) -> Self {
        Self {
            inner,
            cache,
            metrics: None,
        }
    }

    /// メトリクス付きの `CachedSchemaRepository` を作成する。
    pub fn with_metrics(
        inner: Arc<dyn ApiSchemaRepository>,
        cache: Arc<SchemaCache>,
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
impl ApiSchemaRepository for CachedSchemaRepository {
    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    /// `tenant_id` を透過的に inner リポジトリへ渡す。
    async fn find_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<Option<ApiSchema>> {
        // テナント別にキャッシュキーを構築する（テナント間のキャッシュ混在を防ぐ）
        let cache_key = format!("{tenant_id}:{name}");

        // キャッシュヒット確認
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Some(ref m) = self.metrics {
                m.record_cache_hit("api_schemas");
            }
            return Ok(Some((*cached).clone()));
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("api_schemas");
        }

        // キャッシュミス: DBから取得
        let result = self.inner.find_by_name(tenant_id, name).await?;

        // 取得できた場合はテナント別キャッシュキーで格納する
        // SchemaCache は schema.name をキーとするため、テナント別キー（tenant_id:name）を
        // schema の name フィールドに設定してからキャッシュに挿入する
        if let Some(ref schema) = result {
            let mut cache_schema = schema.clone();
            cache_schema.name = cache_key.clone();
            self.cache.insert(Arc::new(cache_schema)).await;
        }

        Ok(result)
    }

    /// `find_all` はキャッシュを使わず inner に委譲する。
    /// `tenant_id` を透過的に inner リポジトリへ渡す。
    async fn find_all(
        &self,
        tenant_id: &str,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        self.inner.find_all(tenant_id, schema_type, page, page_size).await
    }

    /// create は inner に委譲し、成功時にキャッシュを invalidate する。
    /// 新規作成後もキャッシュに古いエントリが残っている可能性があるため
    /// （例: 削除→再作成フローや TTL 期限切れ前のエントリ）、
    /// DB 書き込み成功後にキャッシュを破棄して整合性を保つ。
    /// `tenant_id` を透過的に inner リポジトリへ渡す。
    async fn create(&self, tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()> {
        // DB 操作を先行させる（DB 失敗時はキャッシュ操作を行わない）
        self.inner.create(tenant_id, schema).await?;

        // テナント別キャッシュキーで invalidate する
        let cache_key = format!("{}:{}", tenant_id, schema.name);
        self.cache.invalidate(&cache_key).await;
        tracing::debug!(
            tenant_id = %tenant_id,
            schema_name = %schema.name,
            "create 後にキャッシュを invalidate した"
        );

        Ok(())
    }

    /// update は inner に委譲し、成功時にキャッシュを invalidate する。
    /// DB 操作を先行させることで「DB 更新失敗→キャッシュのみ空」の不整合を防ぐ。
    /// `tenant_id` を透過的に inner リポジトリへ渡す。
    async fn update(&self, tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()> {
        // DB 操作を先行させる（DB 失敗時はキャッシュ操作を行わない）
        self.inner.update(tenant_id, schema).await?;

        // テナント別キャッシュキーで invalidate する
        let cache_key = format!("{}:{}", tenant_id, schema.name);
        self.cache.invalidate(&cache_key).await;
        tracing::debug!(
            tenant_id = %tenant_id,
            schema_name = %schema.name,
            "update 後にキャッシュを invalidate した"
        );

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::api_registration::SchemaType;
    use crate::domain::repository::api_repository::MockApiSchemaRepository;
    use chrono::Utc;

    fn make_schema(name: &str) -> ApiSchema {
        ApiSchema {
            name: name.to_string(),
            description: format!("{} description", name),
            schema_type: SchemaType::OpenApi,
            latest_version: 1,
            version_count: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_cache() -> Arc<SchemaCache> {
        Arc::new(SchemaCache::new(100, 60))
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// テナント別キャッシュキー（tenant_id:name）でキャッシュされた値を返す。
    #[tokio::test]
    async fn test_cache_hit_skips_db() {
        let mut mock = MockApiSchemaRepository::new();
        // find_by_name が呼ばれてはいけない
        mock.expect_find_by_name().never();

        let cache = make_cache();
        let schema = make_schema("k1s0-tenant-api");
        // テナント別キャッシュキーで事前にキャッシュにエントリを挿入
        // SchemaCache は name をキーとするため、テナント別キー "tenant-a:k1s0-tenant-api" で挿入する
        // ただし SchemaCache.get() は name をキーとするため、ここでは name="tenant-a:k1s0-tenant-api" のスキーマを挿入
        let mut cache_schema = schema.clone();
        cache_schema.name = "tenant-a:k1s0-tenant-api".to_string();
        cache.insert(Arc::new(cache_schema)).await;

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_name("tenant-a", "k1s0-tenant-api").await.unwrap();

        assert!(result.is_some());
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する。
    #[tokio::test]
    async fn test_cache_miss_then_store() {
        let schema = make_schema("k1s0-tenant-api");
        let schema_clone = schema.clone();

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_find_by_name()
            .withf(|tenant_id, name| tenant_id == "tenant-a" && name == "k1s0-tenant-api")
            .once()
            .returning(move |_, _| Ok(Some(schema_clone.clone())));

        let cache = make_cache();
        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // 1回目: キャッシュミス → DBから取得
        let result = repo.find_by_name("tenant-a", "k1s0-tenant-api").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "k1s0-tenant-api");

        // テナント別キャッシュキーでエントリが格納されていることを確認
        let cached = cache.get("tenant-a:k1s0-tenant-api").await;
        assert!(cached.is_some());
    }

    /// create 後にキャッシュが invalidate される（再作成フローでの整合性確認）。
    #[tokio::test]
    async fn test_create_invalidates_cache() {
        let schema = make_schema("k1s0-new-api");
        let mut schema_stale = make_schema("k1s0-new-api");
        // テナント別キャッシュキーに合わせてキャッシュキーを設定する
        schema_stale.name = "tenant-a:k1s0-new-api".to_string();

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_create().once().returning(|_, _| Ok(()));

        let cache = make_cache();
        // 事前にキャッシュに古い（ステール）エントリを挿入（再作成シナリオを模倣）
        cache.insert(Arc::new(schema_stale)).await;
        assert!(cache.get("tenant-a:k1s0-new-api").await.is_some(), "前提: キャッシュに古いエントリが存在する");

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // create 実行
        repo.create("tenant-a", &schema).await.unwrap();

        // create 後はキャッシュから古いエントリが invalidate されていることを確認
        let cached = cache.get("tenant-a:k1s0-new-api").await;
        assert!(
            cached.is_none(),
            "create 後はキャッシュが invalidate されるべき"
        );
    }

    /// create が DB エラーの場合はキャッシュを invalidate しない。
    #[tokio::test]
    async fn test_create_db_error_does_not_invalidate_cache() {
        let schema = make_schema("k1s0-new-api");
        let mut schema_stale = make_schema("k1s0-new-api");
        // テナント別キャッシュキーに合わせてキャッシュキーを設定する
        schema_stale.name = "tenant-a:k1s0-new-api".to_string();

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_create()
            .once()
            .returning(|_, _| Err(anyhow::anyhow!("DB error")));

        let cache = make_cache();
        // 事前にキャッシュにエントリを挿入
        cache.insert(Arc::new(schema_stale)).await;

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // create 実行（DB エラー）
        let result = repo.create("tenant-a", &schema).await;
        assert!(result.is_err(), "DB エラー時は Err を返すべき");

        // DB エラー時はキャッシュが invalidate されないことを確認
        let cached = cache.get("tenant-a:k1s0-new-api").await;
        assert!(
            cached.is_some(),
            "DB エラー時はキャッシュが保持されるべき"
        );
    }

    /// update 後にキャッシュが invalidate される。
    #[tokio::test]
    async fn test_update_invalidates_cache() {
        let mut schema_v1 = make_schema("k1s0-tenant-api");
        // テナント別キャッシュキーに合わせてキャッシュキーを設定する
        schema_v1.name = "tenant-a:k1s0-tenant-api".to_string();
        let schema_v2 = ApiSchema {
            latest_version: 2,
            version_count: 2,
            ..make_schema("k1s0-tenant-api")
        };

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_update().once().returning(|_, _| Ok(()));

        let cache = make_cache();
        // 事前にキャッシュにエントリを挿入（古い値）
        cache.insert(Arc::new(schema_v1)).await;

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // update 実行
        repo.update("tenant-a", &schema_v2).await.unwrap();

        // テナント別キャッシュキーで古いエントリが invalidate されていることを確認
        let cached = cache.get("tenant-a:k1s0-tenant-api").await;
        assert!(
            cached.is_none(),
            "update 後はキャッシュが invalidate されるべき"
        );
    }

    /// update が DB エラーの場合はキャッシュを invalidate しない。
    /// DB 操作先行原則により、DB 失敗時はキャッシュを保持して整合性を維持する。
    #[tokio::test]
    async fn test_update_db_error_does_not_invalidate_cache() {
        let mut schema_v1 = make_schema("k1s0-tenant-api");
        // テナント別キャッシュキーに合わせてキャッシュキーを設定する
        schema_v1.name = "tenant-a:k1s0-tenant-api".to_string();
        let schema_v2 = ApiSchema {
            latest_version: 2,
            version_count: 2,
            ..make_schema("k1s0-tenant-api")
        };

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_update()
            .once()
            .returning(|_, _| Err(anyhow::anyhow!("DB error")));

        let cache = make_cache();
        // 事前にキャッシュにエントリを挿入
        cache.insert(Arc::new(schema_v1)).await;

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // update 実行（DB エラー）
        let result = repo.update("tenant-a", &schema_v2).await;
        assert!(result.is_err(), "DB エラー時は Err を返すべき");

        // DB エラー時はキャッシュが invalidate されないことを確認
        let cached = cache.get("tenant-a:k1s0-tenant-api").await;
        assert!(
            cached.is_some(),
            "DB エラー時はキャッシュが保持されるべき（DB 操作先行原則）"
        );
    }
}
