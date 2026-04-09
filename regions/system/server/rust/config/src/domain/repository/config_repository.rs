use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::entity::config_entry::{ConfigEntry, ConfigListResult, ServiceConfigResult};
use crate::domain::error::ConfigRepositoryError;
use async_trait::async_trait;
use uuid::Uuid;

/// `ConfigRepository` は設定値の永続化のためのリポジトリトレイト。
/// 実装は `PostgreSQL` を通じて設定値を管理する。
/// STATIC-CRITICAL-001 監査対応: 全クエリに `tenant_id` フィルタを追加してテナント分離を強制する。
#[cfg_attr(test, mockall::automock)]
// テナント分離・バージョン管理・メタデータで引数が多くなるためアーキテクチャ上の制約として許容する
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    /// `tenant_id` + namespace + key で設定値を取得する。
    async fn find_by_namespace_and_key(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<Option<ConfigEntry>, ConfigRepositoryError>;

    /// テナント内の namespace 設定値一覧を取得する。
    async fn list_by_namespace(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> Result<ConfigListResult, ConfigRepositoryError>;

    /// 設定値を更新する（楽観的排他制御付き）。
    /// `expected_version` と現在のバージョンが一致しない場合はエラーを返す。
    async fn update(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> Result<ConfigEntry, ConfigRepositoryError>;

    /// 設定値を削除する。
    async fn delete(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<bool, ConfigRepositoryError>;

    /// サービス名に紐づく設定値を一括取得する。
    async fn find_by_service_name(
        &self,
        tenant_id: Uuid,
        service_name: &str,
    ) -> Result<ServiceConfigResult, ConfigRepositoryError>;

    /// 設定変更ログを記録する。
    async fn record_change_log(&self, log: &ConfigChangeLog) -> Result<(), ConfigRepositoryError>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::{
        ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
    };

    /// システムテナントUUID: 全テスト共通
    fn system_tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    #[tokio::test]
    async fn test_mock_config_repository_find_by_namespace_and_key() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .withf(|_tid, ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _, _| {
                Ok(Some(ConfigEntry {
                    id: Uuid::new_v4(),
                    namespace: "system.auth.database".to_string(),
                    key: "max_connections".to_string(),
                    value_json: serde_json::json!(25),
                    version: 3,
                    description: "認証サーバーの DB 最大接続数".to_string(),
                    created_by: "admin@example.com".to_string(),
                    updated_by: "admin@example.com".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let result = mock
            .find_by_namespace_and_key(system_tenant(), "system.auth.database", "max_connections")
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
            .returning(|_, _, page, page_size, _| {
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
            .list_by_namespace(system_tenant(), "system.auth.database", 1, 20, None)
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
            .withf(|_tid, name| name == "auth-server")
            .returning(|_, _| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![ServiceConfigEntry {
                        namespace: "system.auth.database".to_string(),
                        key: "max_connections".to_string(),
                        value: serde_json::json!(25),
                        version: 3,
                    }],
                })
            });

        let result = mock
            .find_by_service_name(system_tenant(), "auth-server")
            .await
            .unwrap();
        assert_eq!(result.service_name, "auth-server");
        assert_eq!(result.entries.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_config_repository_delete() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .withf(|_tid, ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _, _| Ok(true));

        let result = mock
            .delete(system_tenant(), "system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(result);
    }
}
