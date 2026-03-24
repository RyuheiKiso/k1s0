// チェックリスト項目追加ユースケース。
// 既存タスクに新しいチェックリスト項目を追加する。
// RLS テナント分離のため tenant_id をリポジトリに渡す。
use crate::domain::entity::task::{AddChecklistItem, TaskChecklistItem};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateChecklistItemUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl CreateChecklistItemUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    /// チェックリスト項目を追加する
    // チェックリスト項目追加の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, task_id: Uuid, input: &AddChecklistItem) -> anyhow::Result<TaskChecklistItem> {
        // TaskError を anyhow::Error に変換して戻り値の型を合わせる
        self.task_repo.add_checklist_item(tenant_id, task_id, input).await.map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::task_repository::MockTaskRepository;
    use chrono::Utc;
    use mockall::predicate::*;

    /// 正常系：項目が追加されて TaskChecklistItem が返ることを確認する
    #[tokio::test]
    async fn test_create_checklist_item_success() {
        let mut mock = MockTaskRepository::new();
        let task_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let expected = TaskChecklistItem {
            id: item_id,
            task_id,
            title: "Review PR".to_string(),
            is_completed: false,
            sort_order: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let expected_clone = expected.clone();

        // add_checklist_item が TaskChecklistItem を返すモックを設定する
        mock.expect_add_checklist_item()
            .with(always(), eq(task_id), always())
            .times(1)
            .returning(move |_, _, _| Ok(expected_clone.clone()));

        let uc = CreateChecklistItemUseCase::new(Arc::new(mock));
        let input = AddChecklistItem { title: "Review PR".to_string(), sort_order: 1 };
        let result = uc.execute("system", task_id, &input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Review PR");
    }
}
