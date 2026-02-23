use std::sync::Arc;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowDefinitionRepository;
use crate::domain::repository::WorkflowInstanceRepository;
use crate::domain::repository::WorkflowTaskRepository;

#[derive(Debug, Clone)]
pub struct StartInstanceInput {
    pub workflow_id: String,
    pub title: String,
    pub initiator_id: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct StartInstanceOutput {
    pub instance: WorkflowInstance,
    pub first_task: Option<WorkflowTask>,
}

#[derive(Debug, thiserror::Error)]
pub enum StartInstanceError {
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("workflow disabled: {0}")]
    WorkflowDisabled(String),

    #[error("workflow has no steps: {0}")]
    NoSteps(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct StartInstanceUseCase {
    definition_repo: Arc<dyn WorkflowDefinitionRepository>,
    instance_repo: Arc<dyn WorkflowInstanceRepository>,
    task_repo: Arc<dyn WorkflowTaskRepository>,
}

impl StartInstanceUseCase {
    pub fn new(
        definition_repo: Arc<dyn WorkflowDefinitionRepository>,
        instance_repo: Arc<dyn WorkflowInstanceRepository>,
        task_repo: Arc<dyn WorkflowTaskRepository>,
    ) -> Self {
        Self {
            definition_repo,
            instance_repo,
            task_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &StartInstanceInput,
    ) -> Result<StartInstanceOutput, StartInstanceError> {
        let definition = self
            .definition_repo
            .find_by_id(&input.workflow_id)
            .await
            .map_err(|e| StartInstanceError::Internal(e.to_string()))?
            .ok_or_else(|| StartInstanceError::WorkflowNotFound(input.workflow_id.clone()))?;

        if !definition.enabled {
            return Err(StartInstanceError::WorkflowDisabled(
                input.workflow_id.clone(),
            ));
        }

        let first_step = definition
            .first_step()
            .ok_or_else(|| StartInstanceError::NoSteps(input.workflow_id.clone()))?;

        let instance_id = format!("inst_{}", uuid::Uuid::new_v4().simple());
        let instance = WorkflowInstance::new(
            instance_id.clone(),
            definition.id.clone(),
            definition.name.clone(),
            input.title.clone(),
            input.initiator_id.clone(),
            Some(first_step.step_id.clone()),
            input.context.clone(),
        );

        self.instance_repo
            .create(&instance)
            .await
            .map_err(|e| StartInstanceError::Internal(e.to_string()))?;

        let task_id = format!("task_{}", uuid::Uuid::new_v4().simple());
        let due_at = first_step
            .timeout_hours
            .map(|h| chrono::Utc::now() + chrono::Duration::hours(h as i64));
        let task = WorkflowTask::new(
            task_id,
            instance_id,
            first_step.step_id.clone(),
            first_step.name.clone(),
            None,
            due_at,
        );

        self.task_repo
            .create(&task)
            .await
            .map_err(|e| StartInstanceError::Internal(e.to_string()))?;

        Ok(StartInstanceOutput {
            instance,
            first_task: Some(task),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow_definition::WorkflowDefinition;
    use crate::domain::entity::workflow_step::WorkflowStep;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;

    fn sample_definition() -> WorkflowDefinition {
        WorkflowDefinition::new(
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "".to_string(),
            true,
            vec![WorkflowStep::new(
                "step-1".to_string(),
                "Approval".to_string(),
                "human_task".to_string(),
                Some("manager".to_string()),
                Some(48),
                Some("end".to_string()),
                Some("end".to_string()),
            )],
        )
    }

    #[tokio::test]
    async fn success() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut task_mock = MockWorkflowTaskRepository::new();

        def_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(sample_definition())));
        inst_mock.expect_create().returning(|_| Ok(()));
        task_mock.expect_create().returning(|_| Ok(()));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
        );
        let input = StartInstanceInput {
            workflow_id: "wf_001".to_string(),
            title: "PC Purchase".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({"item": "laptop"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.instance.status, "running");
        assert!(output.first_task.is_some());
    }

    #[tokio::test]
    async fn workflow_not_found() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();

        def_mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
        );
        let input = StartInstanceInput {
            workflow_id: "wf_missing".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            StartInstanceError::WorkflowNotFound(_)
        ));
    }

    #[tokio::test]
    async fn workflow_disabled() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();

        let mut def = sample_definition();
        def.enabled = false;
        def_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(def.clone())));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
        );
        let input = StartInstanceInput {
            workflow_id: "wf_001".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            StartInstanceError::WorkflowDisabled(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();

        def_mock
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
        );
        let input = StartInstanceInput {
            workflow_id: "wf_001".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            StartInstanceError::Internal(_)
        ));
    }
}
