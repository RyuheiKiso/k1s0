use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::entity::config_entry::{ConfigEntry, ConfigListResult, ServiceConfigResult};
use crate::domain::repository::ConfigRepository;
use crate::infrastructure::cache::ConfigCache;

/// CachedConfigRepository は ConfigRepository をキャッシュでラップする。
/// find_by_namespace_and_key でキャッシュヒット時はDBアクセスをスキップする。
/// update / delete 時はキャッシュを invalidate して整合性を保つ。
pub struct CachedConfigRepository {
    inner: Arc<dyn ConfigRepository>,
    cache: Arc<ConfigCache>,
}

impl CachedConfigRepository {
    /// 新しい CachedConfigRepository を作成する。
    pub fn new(inner: Arc<dyn ConfigRepository>, cache: Arc<ConfigCache>) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl ConfigRepository for CachedConfigRepository {
    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        // キャッシュヒット確認
        if let Some(cached) = self.cache.get(namespace, key).await {
            return Ok(Some((*cached).clone()));
        }

        // キャッシュミス: DBから取得
        let result = self.inner.find_by_namespace_and_key(namespace, key).await?;

        // 取得できた場合はキャッシュに格納
        if let Some(ref entry) = result {
            self.cache.insert(Arc::new(entry.clone())).await;
        }

        Ok(result)
    }

    /// list_by_namespace はキャッシュを使わず inner に委譲する。
    async fn list_by_namespace(
        &self,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> anyhow::Result<ConfigListResult> {
        self.inner
            .list_by_namespace(namespace, page, page_size, search)
            .await
    }

    /// create はキャッシュを使わず inner に委譲する。
    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<ConfigEntry> {
        self.inner.create(entry).await
    }

    /// update は inner に委譲し、成功時にキャッシュを invalidate する。
    async fn update(
        &self,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> anyhow::Result<ConfigEntry> {
        let result = self
            .inner
            .update(namespace, key, value_json, expected_version, description, updated_by)
            .await?;

        // 更新成功時はキャッシュを invalidate して古い値を除去
        self.cache.invalidate(namespace, key).await;

        Ok(result)
    }

    /// delete は inner に委譲し、成功時にキャッシュを invalidate する。
    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<bool> {
        let deleted = self.inner.delete(namespace, key).await?;

        // 削除成功時はキャッシュを invalidate
        if deleted {
            self.cache.invalidate(namespace, key).await;
        }

        Ok(deleted)
    }

    /// find_by_service_name はキャッシュを使わず inner に委譲する。
    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<ServiceConfigResult> {
        self.inner.find_by_service_name(service_name).await
    }

    /// record_change_log はキャッシュを使わず inner に委譲する。
    async fn record_change_log(&self, log: &ConfigChangeLog) -> anyhow::Result<()> {
        self.inner.record_change_log(log).await
    }

    /// list_change_logs はキャッシュを使わず inner に委譲する。
    async fn list_change_logs(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>> {
        self.inner.list_change_logs(namespace, key).await
    }

    /// find_by_id はキャッシュを使わず inner に委譲する。
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<ConfigEntry>> {
        self.inner.find_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::config_repository::MockConfigRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_entry(namespace: &str, key: &str) -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            value_json: serde_json::json!(42),
            version: 1,
            description: None,
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_cache() -> Arc<ConfigCache> {
        Arc::new(ConfigCache::new(100, 60))
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    #[tokio::test]
    async fn test_cache_hit_skips_db() {
        let mut mock = MockConfigRepository::new();
        // find_by_namespace_and_key が呼ばれてはいけない
        mock.expect_find_by_namespace_and_key().never();

        let cache = make_cache();
        let entry = make_entry("system.auth.database", "max_connections");
        // 事前にキャッシュにエントリを挿入
        cache.insert(Arc::new(entry.clone())).await;

        let repo = CachedConfigRepository::new(Arc::new(mock), cache);
        let result = repo
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap();

        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.namespace, "system.auth.database");
        assert_eq!(cached.key, "max_connections");
        assert_eq!(cached.value_json, serde_json::json!(42));
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する。
    #[tokio::test]
    async fn test_cache_miss_then_store() {
        let entry = make_entry("system.auth.database", "max_connections");
        let entry_clone = entry.clone();

        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .once()
            .returning(move |_, _| Ok(Some(entry_clone.clone())));

        let cache = make_cache();
        let repo = CachedConfigRepository::new(Arc::new(mock), cache.clone());

        // 1回目: キャッシュミス → DBから取得
        let result = repo
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value_json, serde_json::json!(42));

        // キャッシュにエントリが格納されていることを確認
        let cached = cache.get("system.auth.database", "max_connections").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().value_json, serde_json::json!(42));
    }

    /// update 後にキャッシュが invalidate される。
    #[tokio::test]
    async fn test_update_invalidates_cache() {
        let entry_v1 = make_entry("system.auth.database", "max_connections");
        let entry_v2 = ConfigEntry {
            value_json: serde_json::json!(100),
            version: 2,
            ..make_entry("system.auth.database", "max_connections")
        };
        let entry_v2_clone = entry_v2.clone();

        let mut mock = MockConfigRepository::new();
        mock.expect_update()
            .withf(|ns, key, _, _, _, _| {
                ns == "system.auth.database" && key == "max_connections"
            })
            .once()
            .returning(move |_, _, _, _, _, _| Ok(entry_v2_clone.clone()));

        let cache = make_cache();
        // 事前にキャッシュにエントリを挿入（古い値）
        cache.insert(Arc::new(entry_v1)).await;

        let repo = CachedConfigRepository::new(Arc::new(mock), cache.clone());

        // update 実行
        let result = repo
            .update(
                "system.auth.database",
                "max_connections",
                &serde_json::json!(100),
                1,
                None,
                "operator@example.com",
            )
            .await
            .unwrap();
        assert_eq!(result.version, 2);

        // キャッシュから古いエントリが invalidate されていることを確認
        let cached = cache.get("system.auth.database", "max_connections").await;
        assert!(cached.is_none(), "update 後はキャッシュが invalidate されるべき");
    }

    /// delete 後にキャッシュが invalidate される。
    #[tokio::test]
    async fn test_delete_invalidates_cache() {
        let entry = make_entry("system.auth.database", "max_connections");

        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .once()
            .returning(|_, _| Ok(true));

        let cache = make_cache();
        // 事前にキャッシュにエントリを挿入
        cache.insert(Arc::new(entry)).await;

        let repo = CachedConfigRepository::new(Arc::new(mock), cache.clone());

        // delete 実行
        let deleted = repo
            .delete("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(deleted);

        // キャッシュから削除されていることを確認
        let cached = cache.get("system.auth.database", "max_connections").await;
        assert!(cached.is_none(), "delete 後はキャッシュが invalidate されるべき");
    }

    /// delete が false を返したときはキャッシュを invalidate しない。
    #[tokio::test]
    async fn test_delete_not_found_does_not_invalidate_cache() {
        let entry = make_entry("system.auth.database", "max_connections");

        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .once()
            .returning(|_, _| Ok(false)); // 対象なし

        let cache = make_cache();
        cache.insert(Arc::new(entry)).await;

        let repo = CachedConfigRepository::new(Arc::new(mock), cache.clone());

        let deleted = repo
            .delete("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(!deleted);

        // delete=false のときはキャッシュはそのまま残る
        let cached = cache.get("system.auth.database", "max_connections").await;
        assert!(cached.is_some(), "削除対象なしのときはキャッシュを保持すべき");
    }
}
