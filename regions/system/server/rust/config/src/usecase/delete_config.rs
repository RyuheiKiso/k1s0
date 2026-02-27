use std::sync::Arc;

use crate::domain::repository::ConfigRepository;

/// DeleteConfigError は設定値削除に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum DeleteConfigError {
    #[error("config not found: {0}/{1}")]
    NotFound(String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// DeleteConfigUseCase は設定値削除ユースケース。
pub struct DeleteConfigUseCase {
    config_repo: Arc<dyn ConfigRepository>,
}

impl DeleteConfigUseCase {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self { config_repo }
    }

    /// 設定値を削除する。
    /// deleted_by は削除実行者を表す（監査ログ用）。
    pub async fn execute(
        &self,
        namespace: &str,
        key: &str,
        deleted_by: &str,
    ) -> Result<(), DeleteConfigError> {
        // 旧値を取得（監査ログ用）
        let old_entry = self
            .config_repo
            .find_by_namespace_and_key(namespace, key)
            .await
            .ok()
            .flatten();

        let deleted = self
            .config_repo
            .delete(namespace, key)
            .await
            .map_err(|e| DeleteConfigError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteConfigError::NotFound(
                namespace.to_string(),
                key.to_string(),
            ));
        }

        // 監査ログ記録（ベストエフォート）
        if let Some(ref entry) = old_entry {
            let change_log = crate::domain::entity::config_change_log::ConfigChangeLog::new(
                crate::domain::entity::config_change_log::CreateChangeLogRequest {
                    config_entry_id: entry.id,
                    namespace: namespace.to_string(),
                    key: key.to_string(),
                    old_value: Some(entry.value_json.clone()),
                    new_value: None,
                    old_version: entry.version,
                    new_version: entry.version + 1,
                    change_type: "DELETED".to_string(),
                    changed_by: deleted_by.to_string(),
                },
            );
            if let Err(e) = self.config_repo.record_change_log(&change_log).await {
                tracing::warn!(
                    namespace = %namespace,
                    key = %key,
                    error = %e,
                    "failed to record config change log for deletion (best-effort)"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::ConfigEntry;
    use crate::domain::repository::config_repository::MockConfigRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_test_entry() -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: Some("DB max connections".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_delete_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(move |_, _| Ok(Some(entry.clone())));
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _| Ok(true));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute(
                "system.auth.database",
                "max_connections",
                "admin@example.com",
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));
        mock.expect_delete().returning(|_, _| Ok(false));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute("nonexistent.namespace", "missing_key", "admin@example.com")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteConfigError::NotFound(ns, key) => {
                assert_eq!(ns, "nonexistent.namespace");
                assert_eq!(key, "missing_key");
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));
        mock.expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("connection refused")));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute("system.auth.database", "max_connections", "admin@example.com")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_config_records_change_log() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(move |_, _| Ok(Some(entry.clone())));
        mock.expect_delete().returning(|_, _| Ok(true));
        mock.expect_record_change_log()
            .withf(|log| {
                log.change_type == "DELETED"
                    && log.namespace == "system.auth.database"
                    && log.key == "max_connections"
                    && log.old_value == Some(serde_json::json!(25))
                    && log.new_value.is_none()
                    && log.changed_by == "operator@example.com"
            })
            .returning(|_| Ok(()));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute(
                "system.auth.database",
                "max_connections",
                "operator@example.com",
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_config_change_log_failure_is_best_effort() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(move |_, _| Ok(Some(entry.clone())));
        mock.expect_delete().returning(|_, _| Ok(true));
        mock.expect_record_change_log()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute(
                "system.auth.database",
                "max_connections",
                "admin@example.com",
            )
            .await;
        // 監査ログ失敗しても削除自体は成功する
        assert!(result.is_ok());
    }
}
