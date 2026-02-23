use std::sync::Arc;

use crate::domain::entity::config_entry::ConfigEntry;
use crate::domain::repository::ConfigRepository;

/// GetConfigError は設定値取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetConfigError {
    #[error("config not found: {0}/{1}")]
    NotFound(String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetConfigUseCase は設定値取得ユースケース。
pub struct GetConfigUseCase {
    config_repo: Arc<dyn ConfigRepository>,
}

impl GetConfigUseCase {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self { config_repo }
    }

    /// namespace と key で設定値を取得する。
    pub async fn execute(&self, namespace: &str, key: &str) -> Result<ConfigEntry, GetConfigError> {
        self.config_repo
            .find_by_namespace_and_key(namespace, key)
            .await
            .map_err(|e| GetConfigError::Internal(e.to_string()))?
            .ok_or_else(|| GetConfigError::NotFound(namespace.to_string(), key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            description: Some("認証サーバーの DB 最大接続数".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        let expected_entry = entry.clone();

        mock.expect_find_by_namespace_and_key()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(move |_, _| Ok(Some(entry.clone())));

        let uc = GetConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("system.auth.database", "max_connections").await;
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.id, expected_entry.id);
        assert_eq!(entry.namespace, expected_entry.namespace);
        assert_eq!(entry.key, expected_entry.key);
        assert_eq!(entry.value_json, serde_json::json!(25));
    }

    #[tokio::test]
    async fn test_get_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));

        let uc = GetConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent.namespace", "missing_key").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetConfigError::NotFound(ns, key) => {
                assert_eq!(ns, "nonexistent.namespace");
                assert_eq!(key, "missing_key");
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Err(anyhow::anyhow!("connection refused")));

        let uc = GetConfigUseCase::new(Arc::new(mock));
        let result = uc.execute("system.auth.database", "max_connections").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }
}
