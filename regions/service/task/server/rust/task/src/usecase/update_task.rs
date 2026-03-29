// タスク更新ユースケース。
// タイトル・説明・優先度・担当者・期限・ラベルを部分更新する。
// RLS テナント分離のため tenant_id をリポジトリに渡す。
use crate::domain::entity::task::{Task, UpdateTask};
use crate::domain::repository::task_repository::TaskRepository;
use crate::domain::service::task_service::TaskService;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateTaskUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl UpdateTaskUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    /// タスクを更新する
    // タスク更新の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, id: Uuid, input: &UpdateTask, updated_by: &str) -> anyhow::Result<Task> {
        // タイトルが指定されている場合はバリデーションを実行する
        if let Some(ref title) = input.title {
            TaskService::validate_title(title)?;
        }
        // TaskError を anyhow::Error に変換して戻り値の型を合わせる
        self.task_repo.update(tenant_id, id, input, updated_by).await.map_err(anyhow::Error::from)
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

    fn sample_task(id: Uuid) -> Task {
        Task {
            id,
            project_id: Uuid::new_v4(),
            title: "Original".to_string(),
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

    /// 正常系：タスクが更新されて Task が返ることを確認する
    #[tokio::test]
    async fn test_update_task_success() {
        let mut mock = MockTaskRepository::new();
        let id = Uuid::new_v4();
        let updated = Task { title: "Updated".to_string(), ..sample_task(id) };
        let updated_clone = updated.clone();

        // update が更新済み Task を返すモックを設定する
        mock.expect_update()
            .with(always(), eq(id), always(), always())
            .times(1)
            .returning(move |_, _, _, _| Ok(updated_clone.clone()));

        let uc = UpdateTaskUseCase::new(Arc::new(mock));
        let input = UpdateTask {
            title: Some("Updated".to_string()),
            description: None,
            priority: None,
            assignee_id: None,
            due_date: None,
            labels: None,
            // 楽観ロック用バージョン番号（テストでは version = 1 を期待する）
            expected_version: 1,
        };
        let result = uc.execute("system", id, &input, "user1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Updated");
    }

    /// 異常系：空タイトルのバリデーションエラーを確認する
    #[tokio::test]
    async fn test_update_task_empty_title_rejected() {
        let mock = MockTaskRepository::new();
        let uc = UpdateTaskUseCase::new(Arc::new(mock));
        let input = UpdateTask {
            title: Some("".to_string()),
            description: None,
            priority: None,
            assignee_id: None,
            due_date: None,
            labels: None,
            // 楽観ロック用バージョン番号（バリデーションエラーのため DB には届かない）
            expected_version: 1,
        };
        let result = uc.execute("system", Uuid::new_v4(), &input, "user1").await;
        assert!(result.is_err());
    }
}
