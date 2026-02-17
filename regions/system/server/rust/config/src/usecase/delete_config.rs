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
    pub async fn execute(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<(), DeleteConfigError> {
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::config_repository::MockConfigRepository;

    #[tokio::test]
    async fn test_delete_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _| Ok(true));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute("system.auth.database", "max_connections")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .returning(|_, _| Ok(false));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute("nonexistent.namespace", "missing_key")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteConfigError::NotFound(ns, key) => {
                assert_eq!(ns, "nonexistent.namespace");
                assert_eq!(key, "missing_key");
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("connection refused")));

        let uc = DeleteConfigUseCase::new(Arc::new(mock));
        let result = uc
            .execute("system.auth.database", "max_connections")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
