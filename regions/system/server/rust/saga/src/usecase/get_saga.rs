use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::saga_state::SagaState;
use crate::domain::entity::saga_step_log::SagaStepLog;
use crate::domain::repository::SagaRepository;

/// GetSagaUseCase はSaga詳細取得を担う。
pub struct GetSagaUseCase {
    saga_repo: Arc<dyn SagaRepository>,
}

impl GetSagaUseCase {
    pub fn new(saga_repo: Arc<dyn SagaRepository>) -> Self {
        Self { saga_repo }
    }

    /// SagaとステップログをIDで取得する。
    pub async fn execute(
        &self,
        saga_id: Uuid,
    ) -> anyhow::Result<Option<(SagaState, Vec<SagaStepLog>)>> {
        let state = self.saga_repo.find_by_id(saga_id).await?;

        match state {
            Some(saga) => {
                let step_logs = self.saga_repo.find_step_logs(saga_id).await?;
                Ok(Some((saga, step_logs)))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::saga_state::SagaState;
    use crate::domain::repository::saga_repository::MockSagaRepository;

    #[tokio::test]
    async fn test_get_saga_found() {
        let saga = SagaState::new("test".to_string(), serde_json::json!({}), None, None);
        let saga_id = saga.saga_id;
        let saga_clone = saga.clone();

        let mut mock = MockSagaRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(saga_clone.clone())));
        mock.expect_find_step_logs().returning(|_| Ok(vec![]));

        let uc = GetSagaUseCase::new(Arc::new(mock));
        let result = uc.execute(saga_id).await.unwrap();
        assert!(result.is_some());
        let (state, logs) = result.unwrap();
        assert_eq!(state.saga_id, saga_id);
        assert!(logs.is_empty());
    }

    #[tokio::test]
    async fn test_get_saga_not_found() {
        let mut mock = MockSagaRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetSagaUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }
}
