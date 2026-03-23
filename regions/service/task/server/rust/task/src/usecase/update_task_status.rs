// タスクステータス更新ユースケース。楽観的ロックでバージョン検証する。
use crate::domain::entity::task::{Task, UpdateTaskStatus};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateTaskStatusUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl UpdateTaskStatusUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    // タスクステータス更新の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(
        &self,
        id: Uuid,
        input: &UpdateTaskStatus,
        updated_by: &str,
    ) -> anyhow::Result<Task> {
        let task = self
            .task_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", id))?;

        // ステータス遷移バリデーション
        task.transition_to(input.status.clone())?;

        self.task_repo.update_status(id, input, updated_by).await
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

    fn sample_task() -> Task {
        Task {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
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
    async fn test_update_status_success() {
        let mut mock = MockTaskRepository::new();
        let task = sample_task();
        let task_id = task.id;
        let task_clone = task.clone();
        let mut updated = task.clone();
        updated.status = TaskStatus::InProgress;
        updated.version = 2;
        let updated_clone = updated.clone();

        mock.expect_find_by_id()
            .with(eq(task_id))
            .times(1)
            .returning(move |_| Ok(Some(task_clone.clone())));

        mock.expect_update_status()
            .with(eq(task_id), always(), always())
            .times(1)
            .returning(move |_, _, _| Ok(updated_clone.clone()));

        let uc = UpdateTaskStatusUseCase::new(Arc::new(mock));
        let input = UpdateTaskStatus {
            status: TaskStatus::InProgress,
            expected_version: 1,
        };
        let result = uc.execute(task_id, &input, "user1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_update_status_invalid_transition() {
        let mut mock = MockTaskRepository::new();
        let task = sample_task();
        let task_id = task.id;
        let task_clone = task.clone();

        mock.expect_find_by_id()
            .with(eq(task_id))
            .times(1)
            .returning(move |_| Ok(Some(task_clone.clone())));

        let uc = UpdateTaskStatusUseCase::new(Arc::new(mock));
        let input = UpdateTaskStatus {
            status: TaskStatus::Done, // invalid: Open -> Done
            expected_version: 1,
        };
        let result = uc.execute(task_id, &input, "user1").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid status transition"));
    }
}
