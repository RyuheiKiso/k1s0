use std::sync::Arc;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowTaskRepository;

#[derive(Debug, Clone)]
pub struct ReassignTaskInput {
    pub task_id: String,
    pub new_assignee_id: String,
    pub reason: Option<String>,
    pub actor_id: String,
}

#[derive(Debug, Clone)]
pub struct ReassignTaskOutput {
    pub task: WorkflowTask,
    pub previous_assignee_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReassignTaskError {
    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("invalid task status for reassignment: {0}")]
    InvalidStatus(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ReassignTaskUseCase {
    repo: Arc<dyn WorkflowTaskRepository>,
}

impl ReassignTaskUseCase {
    pub fn new(repo: Arc<dyn WorkflowTaskRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ReassignTaskInput,
    ) -> Result<ReassignTaskOutput, ReassignTaskError> {
        let mut task = self
            .repo
            .find_by_id(&input.task_id)
            .await
            .map_err(|e| ReassignTaskError::Internal(e.to_string()))?
            .ok_or_else(|| ReassignTaskError::TaskNotFound(input.task_id.clone()))?;

        if !task.is_reassignable() {
            return Err(ReassignTaskError::InvalidStatus(task.status.clone()));
        }

        let previous_assignee_id = task.assignee_id.clone();
        task.reassign(input.new_assignee_id.clone());

        self.repo
            .update(&task)
            .await
            .map_err(|e| ReassignTaskError::Internal(e.to_string()))?;

        Ok(ReassignTaskOutput {
            task,
            previous_assignee_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;

    fn assigned_task() -> WorkflowTask {
        WorkflowTask::new(
            "task_001".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "Approval".to_string(),
            Some("user-002".to_string()),
            None,
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(assigned_task())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = ReassignTaskUseCase::new(Arc::new(mock));
        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: Some("reassignment".to_string()),
            actor_id: "user-002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(
            output.previous_assignee_id,
            Some("user-002".to_string())
        );
        assert_eq!(output.task.assignee_id, Some("user-003".to_string()));
    }

    #[tokio::test]
    async fn task_not_found() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = ReassignTaskUseCase::new(Arc::new(mock));
        let input = ReassignTaskInput {
            task_id: "task_missing".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "user-002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ReassignTaskError::TaskNotFound(_)
        ));
    }

    #[tokio::test]
    async fn invalid_status_approved() {
        let mut mock = MockWorkflowTaskRepository::new();
        let mut task = assigned_task();
        task.approve("prev".to_string(), None);
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(task.clone())));

        let uc = ReassignTaskUseCase::new(Arc::new(mock));
        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "user-002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ReassignTaskError::InvalidStatus(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ReassignTaskUseCase::new(Arc::new(mock));
        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "user-002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ReassignTaskError::Internal(_)
        ));
    }
}
