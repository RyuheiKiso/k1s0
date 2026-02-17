use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::entity::config_entry::{ConfigEntry, ConfigListResult, ServiceConfigResult};

/// ConfigRepository は設定値の永続化のためのリポジトリトレイト。
/// 実装は PostgreSQL を通じて設定値を管理する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    /// namespace と key で設定値を取得する。
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>>;

    /// namespace 内の設定値一覧を取得する。
    async fn list_by_namespace(
        &self,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> anyhow::Result<ConfigListResult>;

    /// 設定値を作成する。
    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<ConfigEntry>;

    /// 設定値を更新する（楽観的排他制御付き）。
    /// expected_version と現在のバージョンが一致しない場合はエラーを返す。
    async fn update(
        &self,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> anyhow::Result<ConfigEntry>;

    /// 設定値を削除する。
    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<bool>;

    /// サービス名に紐づく設定値を一括取得する。
    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<ServiceConfigResult>;

    /// 設定変更ログを記録する。
    async fn record_change_log(&self, log: &ConfigChangeLog) -> anyhow::Result<()>;

    /// 設定変更ログを namespace と key で取得する。
    async fn list_change_logs(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>>;

    /// ID で設定値を取得する。
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<ConfigEntry>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::{
        ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
    };

    #[tokio::test]
    async fn test_mock_config_repository_find_by_namespace_and_key() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _| {
                Ok(Some(ConfigEntry {
                    id: Uuid::new_v4(),
                    namespace: "system.auth.database".to_string(),
                    key: "max_connections".to_string(),
                    value_json: serde_json::json!(25),
                    version: 3,
                    description: Some("認証サーバーの DB 最大接続数".to_string()),
                    created_by: "admin@example.com".to_string(),
                    updated_by: "admin@example.com".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let result = mock
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(result.is_some());

        let entry = result.unwrap();
        assert_eq!(entry.namespace, "system.auth.database");
        assert_eq!(entry.key, "max_connections");
        assert_eq!(entry.value_json, serde_json::json!(25));
    }

    #[tokio::test]
    async fn test_mock_config_repository_list_by_namespace() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![],
                    pagination: Pagination {
                        total_count: 0,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let result = mock
            .list_by_namespace("system.auth.database", 1, 20, None)
            .await
            .unwrap();
        assert_eq!(result.pagination.page, 1);
        assert_eq!(result.pagination.page_size, 20);
        assert!(result.entries.is_empty());
    }

    #[tokio::test]
    async fn test_mock_config_repository_find_by_service_name() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .withf(|name| name == "auth-server")
            .returning(|_| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![ServiceConfigEntry {
                        namespace: "system.auth.database".to_string(),
                        key: "max_connections".to_string(),
                        value: serde_json::json!(25),
                    }],
                })
            });

        let result = mock.find_by_service_name("auth-server").await.unwrap();
        assert_eq!(result.service_name, "auth-server");
        assert_eq!(result.entries.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_config_repository_delete() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _| Ok(true));

        let result = mock
            .delete("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(result);
    }
}
