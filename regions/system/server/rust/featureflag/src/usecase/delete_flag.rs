use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::FeatureFlagRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteFlagError {
    #[error("flag not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
}

impl DeleteFlagUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteFlagError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteFlagError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteFlagError::NotFound(*id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteFlagUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteFlagUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteFlagError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteFlagUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteFlagError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
