use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::api_registration::ApiSchema;
use crate::domain::repository::ApiSchemaRepository;
use crate::infrastructure::cache::SchemaCache;

/// CachedSchemaRepository は ApiSchemaRepository をキャッシュでラップする。
/// find_by_name でキャッシュヒット時はDBアクセスをスキップする。
/// update / create 時はキャッシュを invalidate して整合性を保つ。
pub struct CachedSchemaRepository {
    inner: Arc<dyn ApiSchemaRepository>,
    cache: Arc<SchemaCache>,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl CachedSchemaRepository {
    /// 新しい CachedSchemaRepository を作成する。
    pub fn new(inner: Arc<dyn ApiSchemaRepository>, cache: Arc<SchemaCache>) -> Self {
        Self {
            inner,
            cache,
            metrics: None,
        }
    }

    /// メトリクス付きの CachedSchemaRepository を作成する。
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
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchema>> {
        // キャッシュヒット確認
        if let Some(cached) = self.cache.get(name).await {
            if let Some(ref m) = self.metrics {
                m.record_cache_hit("api_schemas");
            }
            return Ok(Some((*cached).clone()));
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("api_schemas");
        }

        // キャッシュミス: DBから取得
        let result = self.inner.find_by_name(name).await?;

        // 取得できた場合はキャッシュに格納
        if let Some(ref schema) = result {
            self.cache.insert(Arc::new(schema.clone())).await;
        }

        Ok(result)
    }

    /// find_all はキャッシュを使わず inner に委譲する。
    async fn find_all(
        &self,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        self.inner.find_all(schema_type, page, page_size).await
    }

    /// create は inner に委譲する（新規作成のためキャッシュ操作不要）。
    async fn create(&self, schema: &ApiSchema) -> anyhow::Result<()> {
        self.inner.create(schema).await
    }

    /// update は inner に委譲し、成功時にキャッシュを invalidate する。
    async fn update(&self, schema: &ApiSchema) -> anyhow::Result<()> {
        self.inner.update(schema).await?;

        // 更新成功時はキャッシュを invalidate して古い値を除去
        self.cache.invalidate(&schema.name).await;

        Ok(())
    }
}

#[cfg(test)]
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
    #[tokio::test]
    async fn test_cache_hit_skips_db() {
        let mut mock = MockApiSchemaRepository::new();
        // find_by_name が呼ばれてはいけない
        mock.expect_find_by_name().never();

        let cache = make_cache();
        let schema = make_schema("k1s0-tenant-api");
        // 事前にキャッシュにエントリを挿入
        cache.insert(Arc::new(schema.clone())).await;

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_name("k1s0-tenant-api").await.unwrap();

        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.name, "k1s0-tenant-api");
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する。
    #[tokio::test]
    async fn test_cache_miss_then_store() {
        let schema = make_schema("k1s0-tenant-api");
        let schema_clone = schema.clone();

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_find_by_name()
            .withf(|name| name == "k1s0-tenant-api")
            .once()
            .returning(move |_| Ok(Some(schema_clone.clone())));

        let cache = make_cache();
        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // 1回目: キャッシュミス → DBから取得
        let result = repo.find_by_name("k1s0-tenant-api").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "k1s0-tenant-api");

        // キャッシュにエントリが格納されていることを確認
        let cached = cache.get("k1s0-tenant-api").await;
        assert!(cached.is_some());
    }

    /// update 後にキャッシュが invalidate される。
    #[tokio::test]
    async fn test_update_invalidates_cache() {
        let schema_v1 = make_schema("k1s0-tenant-api");
        let schema_v2 = ApiSchema {
            latest_version: 2,
            version_count: 2,
            ..make_schema("k1s0-tenant-api")
        };

        let mut mock = MockApiSchemaRepository::new();
        mock.expect_update().once().returning(|_| Ok(()));

        let cache = make_cache();
        // 事前にキャッシュにエントリを挿入（古い値）
        cache.insert(Arc::new(schema_v1)).await;

        let repo = CachedSchemaRepository::new(Arc::new(mock), cache.clone());

        // update 実行
        repo.update(&schema_v2).await.unwrap();

        // キャッシュから古いエントリが invalidate されていることを確認
        let cached = cache.get("k1s0-tenant-api").await;
        assert!(
            cached.is_none(),
            "update 後はキャッシュが invalidate されるべき"
        );
    }
}
