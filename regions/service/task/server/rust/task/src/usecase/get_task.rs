// タスク取得ユースケース。
// RLS テナント分離のため tenant_id をリポジトリに渡す。
use crate::domain::entity::task::{Task, TaskChecklistItem};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetTaskUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl GetTaskUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    /// タスクを ID で取得する
    // タスク取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<Task>> {
        self.task_repo.find_by_id(tenant_id, id).await
    }

    /// チェックリストを取得する
    // チェックリスト取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn get_checklist(&self, tenant_id: &str, task_id: Uuid) -> anyhow::Result<Vec<TaskChecklistItem>> {
        self.task_repo.find_checklist(tenant_id, task_id).await
    }
}

#[cfg(test)]
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::task::{TaskPriority, TaskStatus};
    use crate::domain::repository::task_repository::MockTaskRepository;
    use chrono::Utc;
    use mockall::predicate::*;
    use uuid::Uuid;

    // テスト用のサンプルタスクを生成するヘルパー関数
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

    /// 正常系：ID を渡すと Task が返ることを確認する
    #[tokio::test]
    async fn test_get_task_success() {
        let mut mock = MockTaskRepository::new();
        let task = sample_task();
        let task_id = task.id;
        let task_clone = task.clone();

        // 指定した ID でリポジトリが Some(Task) を返すようにモックを設定する
        mock.expect_find_by_id()
            .with(always(), eq(task_id))
            .times(1)
            .returning(move |_, _| Ok(Some(task_clone.clone())));

        let uc = GetTaskUseCase::new(Arc::new(mock));
        let result = uc.execute("system", task_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, task_id);
    }

    /// 異常系：存在しない ID の場合に None（NotFound 相当）が返ることを確認する
    #[tokio::test]
    async fn test_get_task_not_found() {
        let mut mock = MockTaskRepository::new();
        let unknown_id = Uuid::new_v4();

        // リポジトリが None を返す場合（タスクが存在しない）のモックを設定する
        mock.expect_find_by_id()
            .with(always(), eq(unknown_id))
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = GetTaskUseCase::new(Arc::new(mock));
        let result = uc.execute("system", unknown_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// 異常系：リポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_get_task_repository_error() {
        let mut mock = MockTaskRepository::new();
        let task_id = Uuid::new_v4();

        // リポジトリがエラーを返す場合のモックを設定し、エラー伝播を検証する
        mock.expect_find_by_id()
            .with(always(), eq(task_id))
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database connection error")));

        let uc = GetTaskUseCase::new(Arc::new(mock));
        let result = uc.execute("system", task_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database connection error"));
    }
}
