use std::sync::Arc;

use crate::domain::entity::quota::QuotaPolicy;
use crate::domain::repository::QuotaPolicyRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetQuotaPolicyError {
    #[error("quota policy not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetQuotaPolicyUseCase {
    repo: Arc<dyn QuotaPolicyRepository>,
}

impl GetQuotaPolicyUseCase {
    pub fn new(repo: Arc<dyn QuotaPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &str) -> Result<QuotaPolicy, GetQuotaPolicyError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetQuotaPolicyError::Internal(e.to_string()))?
            .ok_or_else(|| GetQuotaPolicyError::NotFound(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::quota::{Period, SubjectType};
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    fn sample_policy() -> QuotaPolicy {
        QuotaPolicy::new(
            "test".to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            1000,
            Period::Daily,
            true,
            None,
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockQuotaPolicyRepository::new();
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();

        mock.expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));

        let uc = GetQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "test");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaPolicyError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetQuotaPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute("some-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaPolicyError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
