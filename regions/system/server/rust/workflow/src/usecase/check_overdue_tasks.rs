use std::sync::Arc;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowTaskRepository;
use crate::infrastructure::notification_request_producer::NotificationRequestPublisher;

#[derive(Debug, Clone)]
pub struct CheckOverdueTasksOutput {
    pub overdue_tasks: Vec<WorkflowTask>,
    pub count: usize,
    pub published_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum CheckOverdueTasksError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CheckOverdueTasksUseCase {
    repo: Arc<dyn WorkflowTaskRepository>,
    notification_request_publisher: Arc<dyn NotificationRequestPublisher>,
}

impl CheckOverdueTasksUseCase {
    pub fn new(
        repo: Arc<dyn WorkflowTaskRepository>,
        notification_request_publisher: Arc<dyn NotificationRequestPublisher>,
    ) -> Self {
        Self {
            repo,
            notification_request_publisher,
        }
    }

    pub async fn execute(&self) -> Result<CheckOverdueTasksOutput, CheckOverdueTasksError> {
        let overdue_tasks = self
            .repo
            .find_overdue()
            .await
            .map_err(|e| CheckOverdueTasksError::Internal(e.to_string()))?;

        let count = overdue_tasks.len();
        let mut published_count = 0usize;

        for task in &overdue_tasks {
            if self
                .notification_request_publisher
                .publish_task_overdue(task)
                .await
                .is_ok()
            {
                published_count += 1;
            }
        }

        Ok(CheckOverdueTasksOutput {
            overdue_tasks,
            count,
            published_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;
    use crate::infrastructure::notification_request_producer::MockNotificationRequestPublisher;

    #[tokio::test]
    async fn success_no_overdue() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_overdue().returning(|| Ok(vec![]));
        let mut publisher = MockNotificationRequestPublisher::new();
        publisher.expect_publish_task_overdue().times(0);

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock), Arc::new(publisher));
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.overdue_tasks.is_empty());
        assert_eq!(output.count, 0);
        assert_eq!(output.published_count, 0);
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
        let mut publisher = MockNotificationRequestPublisher::new();
        publisher
            .expect_publish_task_overdue()
            .times(1)
            .returning(|_| Ok(()));

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock), Arc::new(publisher));
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.count, 1);
        assert_eq!(output.published_count, 1);
    }

    #[tokio::test]
    async fn publish_failure_is_ignored() {
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
        let mut publisher = MockNotificationRequestPublisher::new();
        publisher
            .expect_publish_task_overdue()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("kafka error")));

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock), Arc::new(publisher));
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.count, 1);
        assert_eq!(output.published_count, 0);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowTaskRepository::new();
        mock.expect_find_overdue()
            .returning(|| Err(anyhow::anyhow!("db error")));
        let mut publisher = MockNotificationRequestPublisher::new();
        publisher.expect_publish_task_overdue().times(0);

        let uc = CheckOverdueTasksUseCase::new(Arc::new(mock), Arc::new(publisher));
        let result = uc.execute().await;
        assert!(matches!(
            result.unwrap_err(),
            CheckOverdueTasksError::Internal(_)
        ));
    }
}
