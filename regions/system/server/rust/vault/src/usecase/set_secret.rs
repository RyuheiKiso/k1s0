use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::repository::{AccessLogRepository, SecretStore};

#[derive(Debug, Clone)]
pub struct SetSecretInput {
    pub path: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SetSecretError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct SetSecretUseCase {
    store: Arc<dyn SecretStore>,
    audit: Arc<dyn AccessLogRepository>,
}

impl SetSecretUseCase {
    pub fn new(store: Arc<dyn SecretStore>, audit: Arc<dyn AccessLogRepository>) -> Self {
        Self { store, audit }
    }

    pub async fn execute(&self, input: &SetSecretInput) -> Result<i64, SetSecretError> {
        let result = self.store.set(&input.path, input.data.clone()).await;

        match &result {
            Ok(_) => {
                let log = SecretAccessLog::new(
                    input.path.clone(),
                    AccessAction::Write,
                    None,
                    true,
                );
                let _ = self.audit.record(&log).await;
            }
            Err(e) => {
                let mut log = SecretAccessLog::new(
                    input.path.clone(),
                    AccessAction::Write,
                    None,
                    false,
                );
                log.error_msg = Some(e.to_string());
                let _ = self.audit.record(&log).await;
            }
        }

        result.map_err(|e| SetSecretError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;
    use crate::domain::repository::secret_store::MockSecretStore;

    #[tokio::test]
    async fn test_set_secret_success() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_set()
            .withf(|path, data| path == "app/db/password" && data.contains_key("password"))
            .returning(|_, _| Ok(1));

        mock_audit
            .expect_record()
            .returning(|_| Ok(()));

        let uc = SetSecretUseCase::new(Arc::new(mock_store), Arc::new(mock_audit));
        let input = SetSecretInput {
            path: "app/db/password".to_string(),
            data: HashMap::from([("password".to_string(), "s3cret".to_string())]),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_set_secret_store_error() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_set()
            .returning(|_, _| Err(anyhow::anyhow!("storage backend unavailable")));

        mock_audit
            .expect_record()
            .returning(|_| Ok(()));

        let uc = SetSecretUseCase::new(Arc::new(mock_store), Arc::new(mock_audit));
        let input = SetSecretInput {
            path: "app/db/password".to_string(),
            data: HashMap::from([("password".to_string(), "s3cret".to_string())]),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SetSecretError::Internal(msg) => assert!(msg.contains("unavailable")),
        }
    }
}
