use std::sync::Arc;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::repository::{AccessLogRepository, SecretStore};

#[derive(Debug, Clone)]
pub struct DeleteSecretInput {
    pub path: String,
    pub versions: Vec<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteSecretError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteSecretUseCase {
    store: Arc<dyn SecretStore>,
    audit: Arc<dyn AccessLogRepository>,
}

impl DeleteSecretUseCase {
    pub fn new(store: Arc<dyn SecretStore>, audit: Arc<dyn AccessLogRepository>) -> Self {
        Self { store, audit }
    }

    pub async fn execute(&self, input: &DeleteSecretInput) -> Result<(), DeleteSecretError> {
        let result = self
            .store
            .delete(&input.path, input.versions.clone())
            .await;

        match &result {
            Ok(()) => {
                let log = SecretAccessLog::new(
                    input.path.clone(),
                    AccessAction::Delete,
                    None,
                    true,
                );
                let _ = self.audit.record(&log).await;
            }
            Err(e) => {
                let mut log = SecretAccessLog::new(
                    input.path.clone(),
                    AccessAction::Delete,
                    None,
                    false,
                );
                log.error_msg = Some(e.to_string());
                let _ = self.audit.record(&log).await;
            }
        }

        result.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                DeleteSecretError::NotFound(input.path.clone())
            } else {
                DeleteSecretError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;
    use crate::domain::repository::secret_store::MockSecretStore;

    #[tokio::test]
    async fn test_delete_secret_success() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_delete()
            .withf(|path, versions| path == "app/db/password" && versions == &[1])
            .returning(|_, _| Ok(()));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = DeleteSecretUseCase::new(Arc::new(mock_store), Arc::new(mock_audit));
        let input = DeleteSecretInput {
            path: "app/db/password".to_string(),
            versions: vec![1],
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_secret_not_found() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("secret not found: nonexistent")));

        mock_audit.expect_record().returning(|_| Ok(()));

        let uc = DeleteSecretUseCase::new(Arc::new(mock_store), Arc::new(mock_audit));
        let input = DeleteSecretInput {
            path: "nonexistent".to_string(),
            versions: vec![],
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteSecretError::NotFound(path) => assert_eq!(path, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
