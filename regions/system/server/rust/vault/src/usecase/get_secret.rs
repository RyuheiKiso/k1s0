use std::sync::Arc;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::entity::secret::Secret;
use crate::domain::repository::{AccessLogRepository, SecretStore};

#[derive(Debug, Clone)]
pub struct GetSecretInput {
    pub path: String,
    pub version: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetSecretError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetSecretUseCase {
    store: Arc<dyn SecretStore>,
    audit: Arc<dyn AccessLogRepository>,
}

impl GetSecretUseCase {
    pub fn new(store: Arc<dyn SecretStore>, audit: Arc<dyn AccessLogRepository>) -> Self {
        Self { store, audit }
    }

    pub async fn execute(&self, input: &GetSecretInput) -> Result<Secret, GetSecretError> {
        let result = self.store.get(&input.path, input.version).await;

        match &result {
            Ok(_) => {
                let log = SecretAccessLog::new(
                    input.path.clone(),
                    AccessAction::Read,
                    None,
                    true,
                );
                let _ = self.audit.record(&log).await;
            }
            Err(e) => {
                let mut log = SecretAccessLog::new(
                    input.path.clone(),
                    AccessAction::Read,
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
                GetSecretError::NotFound(input.path.clone())
            } else {
                GetSecretError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;
    use crate::domain::repository::secret_store::MockSecretStore;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_get_secret_success() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        let data = HashMap::from([("password".to_string(), "s3cret".to_string())]);
        let secret = Secret::new("app/db/password".to_string(), data);
        let expected_path = secret.path.clone();

        mock_store
            .expect_get()
            .withf(|path, version| path == "app/db/password" && version.is_none())
            .returning(move |_, _| Ok(secret.clone()));

        mock_audit
            .expect_record()
            .returning(|_| Ok(()));

        let uc = GetSecretUseCase::new(Arc::new(mock_store), Arc::new(mock_audit));
        let input = GetSecretInput {
            path: "app/db/password".to_string(),
            version: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let secret = result.unwrap();
        assert_eq!(secret.path, expected_path);
        assert_eq!(secret.versions[0].value.data["password"], "s3cret");
    }

    #[tokio::test]
    async fn test_get_secret_not_found() {
        let mut mock_store = MockSecretStore::new();
        let mut mock_audit = MockAccessLogRepository::new();

        mock_store
            .expect_get()
            .returning(|_, _| Err(anyhow::anyhow!("secret not found: nonexistent")));

        mock_audit
            .expect_record()
            .returning(|_| Ok(()));

        let uc = GetSecretUseCase::new(Arc::new(mock_store), Arc::new(mock_audit));
        let input = GetSecretInput {
            path: "nonexistent".to_string(),
            version: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetSecretError::NotFound(path) => assert_eq!(path, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
