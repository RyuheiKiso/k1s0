// ボードカラム取得ユースケース。
// テナント分離のため tenant_id を受け取りリポジトリに渡す。
use crate::domain::entity::board_column::BoardColumn;
use crate::domain::repository::board_column_repository::BoardColumnRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetBoardColumnUseCase {
    repo: Arc<dyn BoardColumnRepository>,
}

impl GetBoardColumnUseCase {
    pub fn new(repo: Arc<dyn BoardColumnRepository>) -> Self {
        Self { repo }
    }

    // ボードカラム取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<BoardColumn>> {
        self.repo.find_by_id(tenant_id, id).await
    }
}

#[cfg(test)]
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::board_column_repository::MockBoardColumnRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用のサンプルカラムを生成するヘルパー関数
    fn sample_column() -> BoardColumn {
        BoardColumn {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "open".to_string(),
            wip_limit: 3,
            task_count: 1,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 正常系: 指定した ID のカラムが取得できることを確認する
    #[tokio::test]
    async fn test_get_board_column_success() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column();
        let col_id = col.id;
        let col_clone = col.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_, _| Ok(Some(col_clone.clone())));

        let uc = GetBoardColumnUseCase::new(Arc::new(mock));
        let result = uc.execute("test-tenant", col_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, col_id);
    }

    // 異常系: 指定した ID のカラムが存在しない場合（NotFound）に None が返ることを確認する
    #[tokio::test]
    async fn test_get_board_column_not_found() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = GetBoardColumnUseCase::new(Arc::new(mock));
        let result = uc.execute("test-tenant", Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // 異常系: リポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_get_board_column_repository_error() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database error")));

        let uc = GetBoardColumnUseCase::new(Arc::new(mock));
        let result = uc.execute("test-tenant", Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database error"));
    }
}
