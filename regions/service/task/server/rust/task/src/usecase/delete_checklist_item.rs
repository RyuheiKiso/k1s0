// チェックリスト項目削除ユースケース。
// 既存チェックリスト項目を削除する。
// RLS テナント分離のため tenant_id をリポジトリに渡す。
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteChecklistItemUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl DeleteChecklistItemUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    /// チェックリスト項目を削除する
    // チェックリスト項目削除の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, task_id: Uuid, item_id: Uuid) -> anyhow::Result<()> {
        // TaskError を anyhow::Error に変換して戻り値の型を合わせる
        self.task_repo.delete_checklist_item(tenant_id, task_id, item_id).await.map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::task_repository::MockTaskRepository;
    use mockall::predicate::*;

    /// 正常系：項目が削除されて Ok(()) が返ることを確認する
    #[tokio::test]
    async fn test_delete_checklist_item_success() {
        let mut mock = MockTaskRepository::new();
        let task_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();

        // delete_checklist_item が Ok(()) を返すモックを設定する
        mock.expect_delete_checklist_item()
            .with(always(), eq(task_id), eq(item_id))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let uc = DeleteChecklistItemUseCase::new(Arc::new(mock));
        let result = uc.execute("system", task_id, item_id).await;
        assert!(result.is_ok());
    }

    /// 異常系：存在しない項目を削除しようとするとエラーが返ることを確認する
    #[tokio::test]
    async fn test_delete_checklist_item_not_found() {
        let mut mock = MockTaskRepository::new();
        let task_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();

        // delete_checklist_item が NotFound エラーを返すモックを設定する
        mock.expect_delete_checklist_item()
            .with(always(), eq(task_id), eq(item_id))
            .times(1)
            .returning(move |_, _, id| Err(crate::domain::error::TaskError::NotFound(format!("Checklist item '{}' not found", id))));

        let uc = DeleteChecklistItemUseCase::new(Arc::new(mock));
        let result = uc.execute("system", task_id, item_id).await;
        assert!(result.is_err());
    }
}
