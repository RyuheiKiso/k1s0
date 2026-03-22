// タスク作成ユースケース。Outbox パターンでイベントを発行する。
use crate::domain::entity::task::{CreateTask, Task};
use crate::domain::repository::task_repository::TaskRepository;
use crate::domain::service::task_service::TaskService;
use std::sync::Arc;

pub struct CreateTaskUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl CreateTaskUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    // タスク作成の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, input: &CreateTask, created_by: &str) -> anyhow::Result<Task> {
        TaskService::validate_title(&input.title)?;
        self.task_repo.create(input, created_by).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::task::{TaskPriority, TaskStatus};
    use crate::domain::repository::task_repository::MockTaskRepository;
    use chrono::Utc;
    use mockall::predicate::*;
    use uuid::Uuid;

    fn sample_task(project_id: Uuid) -> Task {
        Task {
            id: Uuid::new_v4(),
            project_id,
            title: "Fix bug".to_string(),
            description: None,
            status: TaskStatus::Open,
            priority: TaskPriority::Medium,
            assignee_id: None,
            reporter_id: None,
            due_date: None,
            labels: vec![],
            created_by: "user1".to_string(),
            updated_by: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_task_success() {
        let mut mock = MockTaskRepository::new();
        let project_id = Uuid::new_v4();
        let task = sample_task(project_id);
        let task_clone = task.clone();

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(task_clone.clone()));

        let uc = CreateTaskUseCase::new(Arc::new(mock));
        let input = CreateTask {
            project_id,
            title: "Fix bug".to_string(),
            description: None,
            priority: TaskPriority::Medium,
            assignee_id: None,
            due_date: None,
            labels: vec![],
            checklist: vec![],
        };
        let result = uc.execute(&input, "user1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Fix bug");
    }

    #[tokio::test]
    async fn test_create_task_empty_title() {
        let mock = MockTaskRepository::new();
        let uc = CreateTaskUseCase::new(Arc::new(mock));
        let input = CreateTask {
            project_id: Uuid::new_v4(),
            title: "".to_string(),
            description: None,
            priority: TaskPriority::Medium,
            assignee_id: None,
            due_date: None,
            labels: vec![],
            checklist: vec![],
        };
        let result = uc.execute(&input, "user1").await;
        assert!(result.is_err());
    }
}
