use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::PolicyRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeletePolicyError {
    #[error("policy not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeletePolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl DeletePolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeletePolicyError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeletePolicyError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeletePolicyError::NotFound(*id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeletePolicyUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeletePolicyUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeletePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeletePolicyUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeletePolicyError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
