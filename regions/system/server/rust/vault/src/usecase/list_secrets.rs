use std::sync::Arc;

use crate::domain::repository::SecretStore;

#[derive(Debug, thiserror::Error)]
pub enum ListSecretsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListSecretsUseCase {
    store: Arc<dyn SecretStore>,
}

impl ListSecretsUseCase {
    pub fn new(store: Arc<dyn SecretStore>) -> Self {
        Self { store }
    }

    pub async fn execute(&self, path_prefix: &str) -> Result<Vec<String>, ListSecretsError> {
        self.store
            .list(path_prefix)
            .await
            .map_err(|e| ListSecretsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::secret_store::MockSecretStore;

    #[tokio::test]
    async fn test_list_secrets_success_multiple() {
        let mut mock_store = MockSecretStore::new();

        mock_store
            .expect_list()
            .withf(|prefix| prefix == "app/")
            .returning(|_| {
                Ok(vec![
                    "app/db/password".to_string(),
                    "app/api/key".to_string(),
                ])
            });

        let uc = ListSecretsUseCase::new(Arc::new(mock_store));
        let result = uc.execute("app/").await;

        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"app/db/password".to_string()));
        assert!(paths.contains(&"app/api/key".to_string()));
    }

    #[tokio::test]
    async fn test_list_secrets_empty() {
        let mut mock_store = MockSecretStore::new();

        mock_store
            .expect_list()
            .withf(|prefix| prefix == "nonexistent/")
            .returning(|_| Ok(vec![]));

        let uc = ListSecretsUseCase::new(Arc::new(mock_store));
        let result = uc.execute("nonexistent/").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
