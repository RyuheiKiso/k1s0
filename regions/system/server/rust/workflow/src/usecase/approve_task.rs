use std::sync::Arc;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowDefinitionRepository;
use crate::domain::repository::WorkflowInstanceRepository;
use crate::domain::repository::WorkflowTaskRepository;

#[derive(Debug, Clone)]
pub struct ApproveTaskInput {
    pub task_id: String,
    pub actor_id: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApproveTaskOutput {
    pub task: WorkflowTask,
    pub next_task: Option<WorkflowTask>,
    pub instance_status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApproveTaskError {
    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("invalid task status for approval: {0}")]
    InvalidStatus(String),

    #[error("instance not found: {0}")]
    InstanceNotFound(String),

    #[error("workflow definition not found: {0}")]
    DefinitionNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ApproveTaskUseCase {
    task_repo: Arc<dyn WorkflowTaskRepository>,
    instance_repo: Arc<dyn WorkflowInstanceRepository>,
    definition_repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl ApproveTaskUseCase {
    pub fn new(
        task_repo: Arc<dyn WorkflowTaskRepository>,
        instance_repo: Arc<dyn WorkflowInstanceRepository>,
        definition_repo: Arc<dyn WorkflowDefinitionRepository>,
    ) -> Self {
        Self {
            task_repo,
            instance_repo,
            definition_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &ApproveTaskInput,
    ) -> Result<ApproveTaskOutput, ApproveTaskError> {
        let mut task = self
            .task_repo
            .find_by_id(&input.task_id)
            .await
            .map_err(|e| ApproveTaskError::Internal(e.to_string()))?
            .ok_or_else(|| ApproveTaskError::TaskNotFound(input.task_id.clone()))?;

        if !task.is_decidable() {
            return Err(ApproveTaskError::InvalidStatus(task.status.clone()));
        }

        task.approve(input.actor_id.clone(), input.comment.clone());

        self.task_repo
            .update(&task)
            .await
            .map_err(|e| ApproveTaskError::Internal(e.to_string()))?;

        let mut instance = self
            .instance_repo
            .find_by_id(&task.instance_id)
            .await
            .map_err(|e| ApproveTaskError::Internal(e.to_string()))?
            .ok_or_else(|| ApproveTaskError::InstanceNotFound(task.instance_id.clone()))?;

        let definition = self
            .definition_repo
            .find_by_id(&instance.workflow_id)
            .await
            .map_err(|e| ApproveTaskError::Internal(e.to_string()))?
            .ok_or_else(|| {
                ApproveTaskError::DefinitionNotFound(instance.workflow_id.clone())
            })?;

        let current_step = definition.find_step(&task.step_id);
        let next_step_id = current_step.and_then(|s| s.on_approve.as_deref());

        let mut next_task = None;

        match next_step_id {
            Some("end") | None => {
                instance.complete();
                self.instance_repo
                    .update(&instance)
                    .await
                    .map_err(|e| ApproveTaskError::Internal(e.to_string()))?;
            }
            Some(next_id) => {
                if let Some(next_step) = definition.find_step(next_id) {
                    instance.current_step_id = Some(next_id.to_string());
                    self.instance_repo
                        .update(&instance)
                        .await
                        .map_err(|e| ApproveTaskError::Internal(e.to_string()))?;

                    let new_task_id = format!("task_{}", uuid::Uuid::new_v4().simple());
                    let due_at = next_step
                        .timeout_hours
                        .map(|h| chrono::Utc::now() + chrono::Duration::hours(h as i64));
                    let new_task = WorkflowTask::new(
                        new_task_id,
                        instance.id.clone(),
                        next_step.step_id.clone(),
                        next_step.name.clone(),
                        None,
                        due_at,
                    );
                    self.task_repo
                        .create(&new_task)
                        .await
                        .map_err(|e| ApproveTaskError::Internal(e.to_string()))?;
                    next_task = Some(new_task);
                } else {
                    instance.complete();
                    self.instance_repo
                        .update(&instance)
                        .await
                        .map_err(|e| ApproveTaskError::Internal(e.to_string()))?;
                }
            }
        }

        Ok(ApproveTaskOutput {
            task,
            next_task,
            instance_status: instance.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow_definition::WorkflowDefinition;
    use crate::domain::entity::workflow_instance::WorkflowInstance;
    use crate::domain::entity::workflow_step::WorkflowStep;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;

    fn two_step_definition() -> WorkflowDefinition {
        WorkflowDefinition::new(
            "wf_001".to_string(),
            "test".to_string(),
            "".to_string(),
            true,
            vec![
                WorkflowStep::new(
                    "step-1".to_string(),
                    "First".to_string(),
                    "human_task".to_string(),
                    Some("manager".to_string()),
                    Some(48),
                    Some("step-2".to_string()),
                    Some("end".to_string()),
                ),
                WorkflowStep::new(
                    "step-2".to_string(),
                    "Second".to_string(),
                    "human_task".to_string(),
                    Some("finance".to_string()),
                    Some(72),
                    Some("end".to_string()),
                    Some("step-1".to_string()),
                ),
            ],
        )
    }

    fn running_instance() -> WorkflowInstance {
        WorkflowInstance::new(
            "inst_001".to_string(),
            "wf_001".to_string(),
            "test".to_string(),
            "title".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({}),
        )
    }

    fn pending_task() -> WorkflowTask {
        WorkflowTask::new(
            "task_001".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "First".to_string(),
            Some("user-002".to_string()),
            None,
        )
    }

    #[tokio::test]
    async fn success_advances_to_next_step() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut def_mock = MockWorkflowDefinitionRepository::new();

        task_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(pending_task())));
        task_mock.expect_update().returning(|_| Ok(()));
        task_mock.expect_create().returning(|_| Ok(()));
        inst_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(running_instance())));
        inst_mock.expect_update().returning(|_| Ok(()));
        def_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(two_step_definition())));

        let uc = ApproveTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
        );
        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Approved".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.task.status, "approved");
        assert!(output.next_task.is_some());
        assert_eq!(output.instance_status, "running");
    }

    #[tokio::test]
    async fn success_completes_instance_on_last_step() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut def_mock = MockWorkflowDefinitionRepository::new();

        let mut task = pending_task();
        task.step_id = "step-2".to_string();
        task_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(task.clone())));
        task_mock.expect_update().returning(|_| Ok(()));

        let mut inst = running_instance();
        inst.current_step_id = Some("step-2".to_string());
        inst_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(inst.clone())));
        inst_mock.expect_update().returning(|_| Ok(()));

        def_mock
            .expect_find_by_id()
            .returning(|_| Ok(Some(two_step_definition())));

        let uc = ApproveTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
        );
        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.next_task.is_none());
        assert_eq!(output.instance_status, "completed");
    }

    #[tokio::test]
    async fn task_not_found() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let def_mock = MockWorkflowDefinitionRepository::new();

        task_mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = ApproveTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
        );
        let input = ApproveTaskInput {
            task_id: "task_missing".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ApproveTaskError::TaskNotFound(_)
        ));
    }

    #[tokio::test]
    async fn invalid_status() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let def_mock = MockWorkflowDefinitionRepository::new();

        let mut task = pending_task();
        task.approve("prev-actor".to_string(), None);
        task_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(task.clone())));

        let uc = ApproveTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
        );
        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ApproveTaskError::InvalidStatus(_)
        ));
    }
}
