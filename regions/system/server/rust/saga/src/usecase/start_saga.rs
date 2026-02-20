use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

use crate::domain::entity::saga_state::SagaState;
use crate::domain::repository::{SagaRepository, WorkflowRepository};
use crate::usecase::ExecuteSagaUseCase;

/// StartSagaUseCase はSagaの開始を担う。
pub struct StartSagaUseCase {
    saga_repo: Arc<dyn SagaRepository>,
    workflow_repo: Arc<dyn WorkflowRepository>,
    execute_saga_uc: Arc<ExecuteSagaUseCase>,
}

impl StartSagaUseCase {
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

    /// Sagaを開始する。ワークフロー存在確認 → SagaState作成 → バックグラウンド実行。
    pub async fn execute(
        &self,
        workflow_name: String,
        payload: serde_json::Value,
        correlation_id: Option<String>,
        initiated_by: Option<String>,
    ) -> anyhow::Result<Uuid> {
        // Verify workflow exists
        let workflow = self
            .workflow_repo
            .get(&workflow_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("workflow not found: {}", workflow_name))?;

        // Create saga state
        let state = SagaState::new(workflow_name.clone(), payload, correlation_id, initiated_by);
        let saga_id = state.saga_id;

        // Persist
        self.saga_repo.create(&state).await?;

        info!(
            saga_id = %saga_id,
            workflow = %workflow_name,
            "saga started, launching background execution"
        );

        // Launch background execution
        let execute_uc = self.execute_saga_uc.clone();
        tokio::spawn(async move {
            if let Err(e) = execute_uc.run(saga_id, &workflow).await {
                tracing::error!(
                    saga_id = %saga_id,
                    error = %e,
                    "saga execution failed"
                );
            }
        });

        Ok(saga_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow::WorkflowDefinition;
    use crate::domain::repository::saga_repository::MockSagaRepository;
    use crate::domain::repository::workflow_repository::MockWorkflowRepository;
    use crate::infrastructure::grpc_caller::MockGrpcStepCaller;

    #[tokio::test]
    async fn test_start_saga_workflow_not_found() {
        let mut mock_workflow_repo = MockWorkflowRepository::new();
        mock_workflow_repo.expect_get().returning(|_| Ok(None));

        let mock_saga_repo = MockSagaRepository::new();
        let mock_caller = MockGrpcStepCaller::new();

        let execute_uc = Arc::new(ExecuteSagaUseCase::new(
            Arc::new(mock_saga_repo),
            Arc::new(mock_caller),
            None,
        ));

        let mut mock_saga_repo2 = MockSagaRepository::new();
        let uc = StartSagaUseCase::new(
            Arc::new(mock_saga_repo2),
            Arc::new(mock_workflow_repo),
            execute_uc,
        );

        let result = uc
            .execute("nonexistent".to_string(), serde_json::json!({}), None, None)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("workflow not found"));
    }

    #[tokio::test]
    async fn test_start_saga_success() {
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

        let mut mock_saga_repo = MockSagaRepository::new();
        mock_saga_repo.expect_create().returning(|_| Ok(()));
        mock_saga_repo.expect_find_by_id().returning(|_| Ok(None));

        let mut mock_caller = MockGrpcStepCaller::new();
        mock_caller
            .expect_call_step()
            .returning(|_, _, _| Ok(serde_json::json!({})));

        let execute_uc = Arc::new(ExecuteSagaUseCase::new(
            Arc::new(MockSagaRepository::new()),
            Arc::new(mock_caller),
            None,
        ));

        let uc = StartSagaUseCase::new(
            Arc::new(mock_saga_repo),
            Arc::new(mock_workflow_repo),
            execute_uc,
        );

        let result = uc
            .execute(
                "test-workflow".to_string(),
                serde_json::json!({"order_id": "123"}),
                Some("corr-001".to_string()),
                Some("user-1".to_string()),
            )
            .await;
        assert!(result.is_ok());

        // Brief delay to let background task start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
