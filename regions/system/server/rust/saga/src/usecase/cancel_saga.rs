use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::saga_state::SagaStatus;
use crate::domain::repository::SagaRepository;

/// CancelSagaError はキャンセル操作のエラーを型安全に表現する。
#[derive(Debug, thiserror::Error)]
pub enum CancelSagaError {
    #[error("saga not found: {0}")]
    NotFound(Uuid),
    #[error("saga is already in terminal state: {0}")]
    AlreadyTerminal(String),
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

/// CancelSagaUseCase はSagaキャンセルを担う。
pub struct CancelSagaUseCase {
    saga_repo: Arc<dyn SagaRepository>,
}

impl CancelSagaUseCase {
    pub fn new(saga_repo: Arc<dyn SagaRepository>) -> Self {
        Self { saga_repo }
    }

    /// Sagaをキャンセルする。
    pub async fn execute(&self, saga_id: Uuid) -> Result<(), CancelSagaError> {
        let saga = self
            .saga_repo
            .find_by_id(saga_id)
            .await
            .map_err(CancelSagaError::Internal)?
            .ok_or(CancelSagaError::NotFound(saga_id))?;

        if saga.is_terminal() {
            return Err(CancelSagaError::AlreadyTerminal(saga.status.to_string()));
        }

        self.saga_repo
            .update_status(saga_id, &SagaStatus::Cancelled, None)
            .await
            .map_err(CancelSagaError::Internal)?;

        tracing::info!(saga_id = %saga_id, "saga cancelled");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::saga_state::SagaState;
    use crate::domain::repository::saga_repository::MockSagaRepository;

    #[tokio::test]
    async fn test_cancel_saga_success() {
        let saga = SagaState::new("test".to_string(), serde_json::json!({}), None, None);
        let saga_id = saga.saga_id;
        let saga_clone = saga.clone();

        let mut mock = MockSagaRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(saga_clone.clone())));
        mock.expect_update_status().returning(|_, _, _| Ok(()));

        let uc = CancelSagaUseCase::new(Arc::new(mock));
        assert!(uc.execute(saga_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_saga_not_found() {
        let mut mock = MockSagaRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = CancelSagaUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(CancelSagaError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_cancel_saga_already_terminal() {
        let mut saga = SagaState::new("test".to_string(), serde_json::json!({}), None, None);
        saga.complete();
        let saga_id = saga.saga_id;

        let mut mock = MockSagaRepository::new();
        let saga_clone = saga.clone();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(saga_clone.clone())));

        let uc = CancelSagaUseCase::new(Arc::new(mock));
        let result = uc.execute(saga_id).await;
        assert!(matches!(result, Err(CancelSagaError::AlreadyTerminal(_))));
    }
}
