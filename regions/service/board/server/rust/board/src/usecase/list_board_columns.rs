// ボードカラム一覧ユースケース。
// テナント分離のため tenant_id を受け取りリポジトリに渡す。
use crate::domain::entity::board_column::{BoardColumn, BoardColumnFilter};
use crate::domain::repository::board_column_repository::BoardColumnRepository;
use std::sync::Arc;

pub struct ListBoardColumnsUseCase {
    repo: Arc<dyn BoardColumnRepository>,
}

impl ListBoardColumnsUseCase {
    pub fn new(repo: Arc<dyn BoardColumnRepository>) -> Self {
        Self { repo }
    }

    // ボードカラム一覧取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(
        &self,
        tenant_id: &str,
        filter: &BoardColumnFilter,
    ) -> anyhow::Result<(Vec<BoardColumn>, i64)> {
        let cols = self.repo.find_all(tenant_id, filter).await?;
        let count = self.repo.count(tenant_id, filter).await?;
        Ok((cols, count))
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
    fn sample_column(project_id: Uuid) -> BoardColumn {
        BoardColumn {
            id: Uuid::new_v4(),
            project_id,
            status_code: "open".to_string(),
            wip_limit: 5,
            task_count: 0,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 正常系: ボード ID を指定して全カラムが取得できることを確認する
    #[tokio::test]
    async fn test_list_board_columns_success() {
        let mut mock = MockBoardColumnRepository::new();
        let project_id = Uuid::new_v4();
        let col1 = sample_column(project_id);
        let col2 = sample_column(project_id);
        let cols = vec![col1, col2];
        let cols_clone = cols.clone();

        mock.expect_find_all()
            .times(1)
            .returning(move |_, _| Ok(cols_clone.clone()));
        mock.expect_count()
            .times(1)
            .returning(|_, _| Ok(2));

        let uc = ListBoardColumnsUseCase::new(Arc::new(mock));
        let filter = BoardColumnFilter {
            project_id: Some(project_id),
            ..Default::default()
        };
        let result = uc.execute("test-tenant", &filter).await;
        assert!(result.is_ok());
        let (items, count) = result.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(count, 2);
    }

    // 正常系: 対象ボードにカラムが存在しない場合に空リストが返ることを確認する
    #[tokio::test]
    async fn test_list_board_columns_empty() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_find_all()
            .times(1)
            .returning(|_, _| Ok(vec![]));
        mock.expect_count()
            .times(1)
            .returning(|_, _| Ok(0));

        let uc = ListBoardColumnsUseCase::new(Arc::new(mock));
        let filter = BoardColumnFilter {
            project_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let result = uc.execute("test-tenant", &filter).await;
        assert!(result.is_ok());
        let (items, count) = result.unwrap();
        assert!(items.is_empty());
        assert_eq!(count, 0);
    }

    // 異常系: find_all でリポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_list_board_columns_find_all_error() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_find_all()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database connection failed")));

        let uc = ListBoardColumnsUseCase::new(Arc::new(mock));
        let filter = BoardColumnFilter::default();
        let result = uc.execute("test-tenant", &filter).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database connection failed"));
    }

    // 異常系: count でリポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_list_board_columns_count_error() {
        let mut mock = MockBoardColumnRepository::new();
        let project_id = Uuid::new_v4();
        let cols = vec![sample_column(project_id)];

        mock.expect_find_all()
            .times(1)
            .returning(move |_, _| Ok(cols.clone()));
        mock.expect_count()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("count query failed")));

        let uc = ListBoardColumnsUseCase::new(Arc::new(mock));
        let filter = BoardColumnFilter {
            project_id: Some(project_id),
            ..Default::default()
        };
        let result = uc.execute("test-tenant", &filter).await;
        assert!(result.is_err());
    }
}
