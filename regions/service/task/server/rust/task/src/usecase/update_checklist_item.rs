// チェックリスト項目更新ユースケース。
// 既存チェックリスト項目のタイトル・完了状態・並び順を更新する。
// RLS テナント分離のため tenant_id をリポジトリに渡す。
use crate::domain::entity::task::{TaskChecklistItem, UpdateChecklistItem};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateChecklistItemUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl UpdateChecklistItemUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    /// チェックリスト項目を更新する
    // チェックリスト項目更新の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, task_id: Uuid, item_id: Uuid, input: &UpdateChecklistItem) -> anyhow::Result<TaskChecklistItem> {
        // TaskError を anyhow::Error に変換して戻り値の型を合わせる
        self.task_repo.update_checklist_item(tenant_id, task_id, item_id, input).await.map_err(anyhow::Error::from)
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

    /// 正常系：項目が更新されて TaskChecklistItem が返ることを確認する
    #[tokio::test]
    async fn test_update_checklist_item_success() {
        let mut mock = MockTaskRepository::new();
        let task_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let expected = TaskChecklistItem {
            id: item_id,
            task_id,
            title: "Updated title".to_string(),
            is_completed: true,
            sort_order: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let expected_clone = expected.clone();

        // update_checklist_item が更新済み TaskChecklistItem を返すモックを設定する
        mock.expect_update_checklist_item()
            .with(always(), eq(task_id), eq(item_id), always())
            .times(1)
            .returning(move |_, _, _, _| Ok(expected_clone.clone()));

        let uc = UpdateChecklistItemUseCase::new(Arc::new(mock));
        let input = UpdateChecklistItem {
            title: Some("Updated title".to_string()),
            is_completed: Some(true),
            sort_order: Some(2),
        };
        let result = uc.execute("system", task_id, item_id, &input).await;
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.title, "Updated title");
        assert!(item.is_completed);
    }
}
