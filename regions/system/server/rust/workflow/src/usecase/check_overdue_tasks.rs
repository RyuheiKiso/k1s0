use std::sync::Arc;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowTaskRepository;

#[derive(Debug, Clone)]
pub struct CheckOverdueTasksOutput {
    pub overdue_tasks: Vec<WorkflowTask>,
    pub count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum CheckOverdueTasksError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CheckOverdueTasksUseCase {
    repo: Arc<dyn WorkflowTaskRepository>,
}

impl CheckOverdueTasksUseCase {
    pub fn new(repo: Arc<dyn WorkflowTaskRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<CheckOverdueTasksOutput, CheckOverdueTasksError> {
        let overdue_tasks = self
            .repo
            .find_overdue()
            .await
            .map_err(|e| CheckOverdueTasksError::Internal(e.to_string()))?;

        let count = overdue_tasks.len();

        Ok(CheckOverdueTasksOutput {
            overdue_tasks,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;

    #[tokio::test]
    async fn success_no_overdue() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_overdue().returning(|| Ok(vec![]));

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.overdue_tasks.is_empty());
        assert_eq!(output.count, 0);
    }

    #[tokio::test]
    async fn success_with_overdue() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_overdue().returning(|| {
            let mut task = WorkflowTask::new(
                "task_001".to_string(),
                "inst_001".to_string(),
                "step-1".to_string(),
                "Approval".to_string(),
                Some("user-002".to_string()),
                Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            );
            task.due_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
            Ok(vec![task])
        });

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.count, 1);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_overdue()
            .returning(|| Err(anyhow::anyhow!("db error")));

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(matches!(
            result.unwrap_err(),
            CheckOverdueTasksError::Internal(_)
        ));
    }
}
