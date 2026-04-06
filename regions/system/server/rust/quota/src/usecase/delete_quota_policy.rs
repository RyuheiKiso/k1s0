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

    /// CRITICAL-RUST-001 監査対応: tenant_id を受け取り delete に渡して RLS を有効にする。
    pub async fn execute(&self, id: &str, tenant_id: &str) -> Result<(), DeleteQuotaPolicyError> {
        let deleted = self
            .repo
            .delete(id, tenant_id)
            .await
            .map_err(|e| DeleteQuotaPolicyError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteQuotaPolicyError::NotFound(id.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_delete()
            .withf(|id, _tenant_id| id == "quota-1")
            .returning(|_, _| Ok(true));

        let uc = DeleteQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("quota-1", "tenant-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_delete().returning(|_, _| Ok(false));

        let uc = DeleteQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent", "tenant-1").await;
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
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = DeleteQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("some-id", "tenant-1").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteQuotaPolicyError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
