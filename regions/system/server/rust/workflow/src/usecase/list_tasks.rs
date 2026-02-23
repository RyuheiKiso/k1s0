use std::sync::Arc;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowTaskRepository;

#[derive(Debug, Clone)]
pub struct ListTasksInput {
    pub assignee_id: Option<String>,
    pub status: Option<String>,
    pub instance_id: Option<String>,
    pub overdue_only: bool,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListTasksOutput {
    pub tasks: Vec<WorkflowTask>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListTasksError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListTasksUseCase {
    repo: Arc<dyn WorkflowTaskRepository>,
}

impl ListTasksUseCase {
    pub fn new(repo: Arc<dyn WorkflowTaskRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListTasksInput,
    ) -> Result<ListTasksOutput, ListTasksError> {
        let (tasks, total_count) = self
            .repo
            .find_all(
                input.assignee_id.clone(),
                input.status.clone(),
                input.instance_id.clone(),
                input.overdue_only,
                input.page,
                input.page_size,
            )
            .await
            .map_err(|e| ListTasksError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListTasksOutput {
            tasks,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _, _| Ok((vec![], 0)));

        let uc = ListTasksUseCase::new(Arc::new(mock));
        let input = ListTasksInput {
            assignee_id: None,
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.tasks.is_empty());
        assert_eq!(output.total_count, 0);
    }

    #[tokio::test]
    async fn has_next_page() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _, _| Ok((vec![], 30)));

        let uc = ListTasksUseCase::new(Arc::new(mock));
        let input = ListTasksInput {
            assignee_id: Some("user-002".to_string()),
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.unwrap().has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListTasksUseCase::new(Arc::new(mock));
        let input = ListTasksInput {
            assignee_id: None,
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ListTasksError::Internal(_)
        ));
    }
}
