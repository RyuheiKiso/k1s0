// カラムタスク数減少ユースケース（在庫解放に相当）。
// テナント分離のため tenant_id を受け取りリポジトリに渡す。
use crate::domain::entity::board_column::{BoardColumn, DecrementColumnRequest};
use crate::domain::repository::board_column_repository::BoardColumnRepository;
use std::sync::Arc;

pub struct DecrementColumnUseCase {
    repo: Arc<dyn BoardColumnRepository>,
}

impl DecrementColumnUseCase {
    pub fn new(repo: Arc<dyn BoardColumnRepository>) -> Self {
        Self { repo }
    }

    // カラムタスク数減少の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, req: &DecrementColumnRequest) -> anyhow::Result<BoardColumn> {
        // BoardError を anyhow::Error に変換して戻り値の型を合わせる
        self.repo.decrement(tenant_id, req).await.map_err(anyhow::Error::from)
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
            status_code: "in_progress".to_string(),
            wip_limit: 5,
            task_count: 2,
            version: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 正常系: カラムタスク数のデクリメントが成功することを確認する
    #[tokio::test]
    async fn test_decrement_success() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column();
        let col_clone = col.clone();

        mock.expect_decrement()
            .times(1)
            .returning(move |_, _| Ok(col_clone.clone()));

        let uc = DecrementColumnUseCase::new(Arc::new(mock));
        let req = DecrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: col.project_id,
            status_code: "in_progress".to_string(),
            reason: None,
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().task_count, 2);
    }

    // 異常系: リポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_decrement_repository_error() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_decrement()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database connection failed").into()));

        let uc = DecrementColumnUseCase::new(Arc::new(mock));
        let req = DecrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "in_progress".to_string(),
            reason: None,
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database connection failed"));
    }

    // 異常系: 対象カラムが存在しない場合（NotFound）にエラーが返ることを確認する
    #[tokio::test]
    async fn test_decrement_not_found() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_decrement()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("column not found").into()));

        let uc = DecrementColumnUseCase::new(Arc::new(mock));
        let req = DecrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "unknown".to_string(),
            reason: Some("task cancelled".to_string()),
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_err());
    }

    // 正常系: reason フィールドを指定したデクリメントが成功することを確認する
    #[tokio::test]
    async fn test_decrement_with_reason() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column();
        let col_clone = col.clone();

        mock.expect_decrement()
            .times(1)
            .returning(move |_, _| Ok(col_clone.clone()));

        let uc = DecrementColumnUseCase::new(Arc::new(mock));
        let req = DecrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: col.project_id,
            status_code: "in_progress".to_string(),
            reason: Some("task completed".to_string()),
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_ok());
    }
}
