use std::sync::Arc;

use tracing::{error, info, warn};

use crate::domain::repository::{SagaRepository, WorkflowRepository};
use crate::usecase::ExecuteSagaUseCase;

/// RecoverSagasUseCase は起動時に未完了Sagaを再開する。
pub struct RecoverSagasUseCase {
    saga_repo: Arc<dyn SagaRepository>,
    workflow_repo: Arc<dyn WorkflowRepository>,
    execute_saga_uc: Arc<ExecuteSagaUseCase>,
}

impl RecoverSagasUseCase {
    pub fn new(
        saga_repo: Arc<dyn SagaRepository>,
        workflow_repo: Arc<dyn WorkflowRepository>,
        execute_saga_uc: Arc<ExecuteSagaUseCase>,
    ) -> Self {
        Self {
            saga_repo,
            workflow_repo,
            execute_saga_uc,
        }
    }

    /// 未完了Sagaを検索し、バックグラウンドで再開する。
    pub async fn execute(&self) -> anyhow::Result<usize> {
        let incomplete = self.saga_repo.find_incomplete().await?;
        let count = incomplete.len();

        if count == 0 {
            info!("no incomplete sagas to recover");
            return Ok(0);
        }

        info!(count = count, "recovering incomplete sagas");

        for saga in incomplete {
            let saga_id = saga.saga_id;
            let workflow_name = saga.workflow_name.clone();

            match self.workflow_repo.get(&workflow_name).await {
                Ok(Some(workflow)) => {
                    let execute_uc = self.execute_saga_uc.clone();
                    tokio::spawn(async move {
                        info!(saga_id = %saga_id, workflow = %workflow_name, "resuming saga");
                        if let Err(e) = execute_uc.run(saga_id, &workflow).await {
                            error!(
                                saga_id = %saga_id,
                                error = %e,
                                "failed to recover saga"
                            );
                        }
                    });
                }
                Ok(None) => {
                    warn!(
                        saga_id = %saga_id,
                        workflow = %workflow_name,
                        "workflow not found for incomplete saga, skipping"
                    );
                }
                Err(e) => {
                    error!(
                        saga_id = %saga_id,
                        workflow = %workflow_name,
                        error = %e,
                        "failed to load workflow for recovery"
                    );
                }
            }
        }

        info!(count = count, "saga recovery initiated");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::saga_state::SagaState;
    use crate::domain::entity::workflow::WorkflowDefinition;
    use crate::domain::repository::saga_repository::MockSagaRepository;
    use crate::domain::repository::workflow_repository::MockWorkflowRepository;
    use crate::infrastructure::grpc_caller::MockGrpcStepCaller;

    #[tokio::test]
    async fn test_recover_no_incomplete() {
        let mut mock_saga_repo = MockSagaRepository::new();
        mock_saga_repo
            .expect_find_incomplete()
            .returning(|| Ok(vec![]));

        let mock_workflow_repo = MockWorkflowRepository::new();
        let mock_caller = MockGrpcStepCaller::new();

        let execute_uc = Arc::new(ExecuteSagaUseCase::new(
            Arc::new(MockSagaRepository::new()),
            Arc::new(mock_caller),
            None,
        ));

        let uc = RecoverSagasUseCase::new(
            Arc::new(mock_saga_repo),
            Arc::new(mock_workflow_repo),
            execute_uc,
        );

        let count = uc.execute().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_recover_with_incomplete_sagas() {
        let saga = SagaState::new(
            "test-workflow".to_string(),
            serde_json::json!({}),
            None,
            None,
        );

        let mut mock_saga_repo = MockSagaRepository::new();
        let saga_clone = saga.clone();
        mock_saga_repo
            .expect_find_incomplete()
            .returning(move || Ok(vec![saga_clone.clone()]));

        let yaml = r#"
name: test-workflow
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let workflow = WorkflowDefinition::from_yaml(yaml).unwrap();
        let workflow_clone = workflow.clone();

        let mut mock_workflow_repo = MockWorkflowRepository::new();
        mock_workflow_repo
            .expect_get()
            .returning(move |_| Ok(Some(workflow_clone.clone())));

        let mut mock_saga_repo2 = MockSagaRepository::new();
        mock_saga_repo2.expect_find_by_id().returning(|_| Ok(None));

        let mock_caller = MockGrpcStepCaller::new();

        let execute_uc = Arc::new(ExecuteSagaUseCase::new(
            Arc::new(mock_saga_repo2),
            Arc::new(mock_caller),
            None,
        ));

        let uc = RecoverSagasUseCase::new(
            Arc::new(mock_saga_repo),
            Arc::new(mock_workflow_repo),
            execute_uc,
        );

        let count = uc.execute().await.unwrap();
        assert_eq!(count, 1);
    }
}
