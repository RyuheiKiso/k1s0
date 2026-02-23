use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetPolicyError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetPolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl GetPolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<Option<Policy>, GetPolicyError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetPolicyError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn found() {
        let id = Uuid::new_v4();
        let id_clone = id;
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .withf(move |i| *i == id_clone)
            .returning(move |_| {
                Ok(Some(Policy {
                    id,
                    name: "test-policy".to_string(),
                    description: "Test".to_string(),
                    rego_content: "package test".to_string(),
                    version: 1,
                    enabled: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = GetPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
