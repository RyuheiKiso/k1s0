use std::sync::Arc;

use crate::domain::repository::QuotaPolicyRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteQuotaPolicyError {
    #[error("quota policy not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteQuotaPolicyUseCase {
    repo: Arc<dyn QuotaPolicyRepository>,
}

impl DeleteQuotaPolicyUseCase {
    pub fn new(repo: Arc<dyn QuotaPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DeleteQuotaPolicyError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteQuotaPolicyError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteQuotaPolicyError::NotFound(id.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_delete()
            .withf(|id| id == "quota-1")
            .returning(|_| Ok(true));

        let uc = DeleteQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("quota-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteQuotaPolicyError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("some-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteQuotaPolicyError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
