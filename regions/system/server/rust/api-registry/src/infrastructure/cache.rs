/// SchemaCache は API スキーマ情報のインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きでスキーマ情報をキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::api_registration::ApiSchema;

/// キャッシュキーはスキーマ名（name）。
pub struct SchemaCache {
    inner: Cache<String, Arc<ApiSchema>>,
}

impl SchemaCache {
    /// 新しい SchemaCache を作成する。
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

    /// name に対応するスキーマを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, name: &str) -> Option<Arc<ApiSchema>> {
        self.inner.get(name).await
    }

    /// スキーマをキャッシュに追加する。
    /// キーは schema.name から自動生成する。
    pub async fn insert(&self, schema: Arc<ApiSchema>) {
        self.inner.insert(schema.name.clone(), schema).await;
    }

    /// 特定の name のスキーマをキャッシュから削除する。
    pub async fn invalidate(&self, name: &str) {
        self.inner.invalidate(name).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::api_registration::SchemaType;
    use chrono::Utc;

    fn make_schema(name: &str) -> Arc<ApiSchema> {
        Arc::new(ApiSchema {
            name: name.to_string(),
            description: format!("{} description", name),
            schema_type: SchemaType::OpenApi,
            latest_version: 1,
            version_count: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[tokio::test]
    async fn test_insert_and_get_returns_schema() {
        let cache = SchemaCache::new(100, 60);
        let schema = make_schema("k1s0-tenant-api");

        cache.insert(schema.clone()).await;

        let result = cache.get("k1s0-tenant-api").await;
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.name, "k1s0-tenant-api");
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = SchemaCache::new(100, 60);

        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_entry() {
        let cache = SchemaCache::new(100, 60);
        let schema = make_schema("k1s0-tenant-api");
        cache.insert(schema).await;

        assert!(cache.get("k1s0-tenant-api").await.is_some());

        cache.invalidate("k1s0-tenant-api").await;

        let result = cache.get("k1s0-tenant-api").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_schemas() {
        let cache = SchemaCache::new(100, 60);
        let schema1 = make_schema("k1s0-tenant-api");
        let schema2 = make_schema("k1s0-auth-api");
        cache.insert(schema1).await;
        cache.insert(schema2).await;

        cache.invalidate("k1s0-tenant-api").await;

        assert!(cache.get("k1s0-tenant-api").await.is_none());
        assert!(cache.get("k1s0-auth-api").await.is_some());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing_entry() {
        let cache = SchemaCache::new(100, 60);

        let schema_v1 = make_schema("k1s0-tenant-api");
        cache.insert(schema_v1).await;

        let schema_v2 = Arc::new(ApiSchema {
            name: "k1s0-tenant-api".to_string(),
            description: "Updated description".to_string(),
            schema_type: SchemaType::OpenApi,
            latest_version: 2,
            version_count: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        cache.insert(schema_v2).await;

        let result = cache.get("k1s0-tenant-api").await.unwrap();
        assert_eq!(result.latest_version, 2);
        assert_eq!(result.description, "Updated description");
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = SchemaCache::new(100, 1);
        let schema = make_schema("k1s0-tenant-api");
        cache.insert(schema).await;

        assert!(cache.get("k1s0-tenant-api").await.is_some());

        tokio::time::sleep(Duration::from_millis(1200)).await;

        let result = cache.get("k1s0-tenant-api").await;
        assert!(result.is_none());
    }
}
