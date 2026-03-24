// タスク一覧ユースケース。
// RLS テナント分離のため tenant_id をリポジトリに渡す。
use crate::domain::entity::task::{Task, TaskFilter};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;

pub struct ListTasksUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl ListTasksUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    // タスク一覧取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, filter: &TaskFilter) -> anyhow::Result<(Vec<Task>, i64)> {
        let tasks = self.task_repo.find_all(tenant_id, filter).await?;
        let count = self.task_repo.count(tenant_id, filter).await?;
        Ok((tasks, count))
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
    use uuid::Uuid;

    // テスト用のサンプルタスクを生成するヘルパー関数
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

    /// 正常系：フィルター条件を渡すと Task 一覧とページネーション情報（件数）が返ることを確認する
    #[tokio::test]
    async fn test_list_tasks_success() {
        let mut mock = MockTaskRepository::new();
        let project_id = Uuid::new_v4();
        let task1 = sample_task(project_id);
        let task2 = sample_task(project_id);
        let tasks = vec![task1, task2];
        let tasks_clone = tasks.clone();

        // find_all が 2 件のタスクを返すようにモックを設定する
        mock.expect_find_all()
            .times(1)
            .returning(move |_, _| Ok(tasks_clone.clone()));

        // count が 2 を返すようにモックを設定する
        mock.expect_count()
            .times(1)
            .returning(|_, _| Ok(2i64));

        let uc = ListTasksUseCase::new(Arc::new(mock));
        let filter = TaskFilter {
            project_id: Some(project_id),
            ..Default::default()
        };
        let result = uc.execute("system", &filter).await;
        assert!(result.is_ok());
        let (task_list, total) = result.unwrap();
        assert_eq!(task_list.len(), 2);
        assert_eq!(total, 2);
    }

    /// 正常系：一致するタスクがない場合に空リストと件数 0 が返ることを確認する
    #[tokio::test]
    async fn test_list_tasks_empty() {
        let mut mock = MockTaskRepository::new();

        // find_all が空のベクタを返すようにモックを設定する
        mock.expect_find_all()
            .times(1)
            .returning(|_, _| Ok(vec![]));

        // count が 0 を返すようにモックを設定する
        mock.expect_count()
            .times(1)
            .returning(|_, _| Ok(0i64));

        let uc = ListTasksUseCase::new(Arc::new(mock));
        let filter = TaskFilter {
            project_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let result = uc.execute("system", &filter).await;
        assert!(result.is_ok());
        let (task_list, total) = result.unwrap();
        assert!(task_list.is_empty());
        assert_eq!(total, 0);
    }

    /// 異常系：リポジトリの find_all がエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_list_tasks_repository_error() {
        let mut mock = MockTaskRepository::new();

        // find_all がエラーを返す場合のモックを設定し、エラー伝播を検証する
        mock.expect_find_all()
            .times(1)
            .returning(|_, _| Err(crate::domain::error::TaskError::Infrastructure(anyhow::anyhow!("database connection error"))));

        let uc = ListTasksUseCase::new(Arc::new(mock));
        let filter = TaskFilter::default();
        let result = uc.execute("system", &filter).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database connection error"));
    }
}
