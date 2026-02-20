use std::sync::Arc;

use crate::domain::entity::saga_state::SagaState;
use crate::domain::repository::saga_repository::SagaListParams;
use crate::domain::repository::SagaRepository;

/// ListSagasUseCase はSaga一覧取得を担う。
pub struct ListSagasUseCase {
    saga_repo: Arc<dyn SagaRepository>,
}

impl ListSagasUseCase {
    pub fn new(saga_repo: Arc<dyn SagaRepository>) -> Self {
        Self { saga_repo }
    }

    /// パラメータに基づいてSaga一覧を取得する。
    pub async fn execute(&self, params: SagaListParams) -> anyhow::Result<(Vec<SagaState>, i64)> {
        self.saga_repo.list(&params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::saga_repository::MockSagaRepository;

    #[tokio::test]
    async fn test_list_sagas() {
        let mut mock = MockSagaRepository::new();
        mock.expect_list().returning(|_| Ok((vec![], 0)));

        let uc = ListSagasUseCase::new(Arc::new(mock));
        let params = SagaListParams {
            page: 1,
            page_size: 20,
            ..Default::default()
        };
        let (sagas, total) = uc.execute(params).await.unwrap();
        assert!(sagas.is_empty());
        assert_eq!(total, 0);
    }
}
