// カラムタスク数増加ユースケース（在庫予約に相当）。
// テナント分離のため tenant_id を受け取りリポジトリに渡す。
use crate::domain::entity::board_column::{BoardColumn, IncrementColumnRequest};
use crate::domain::repository::board_column_repository::BoardColumnRepository;
use std::sync::Arc;

pub struct IncrementColumnUseCase {
    repo: Arc<dyn BoardColumnRepository>,
}

impl IncrementColumnUseCase {
    pub fn new(repo: Arc<dyn BoardColumnRepository>) -> Self {
        Self { repo }
    }

    // カラムタスク数増加の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, tenant_id: &str, req: &IncrementColumnRequest) -> anyhow::Result<BoardColumn> {
        // BoardError を anyhow::Error に変換して戻り値の型を合わせる
        self.repo.increment(tenant_id, req).await.map_err(anyhow::Error::from)
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

    fn sample_column() -> BoardColumn {
        BoardColumn {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "open".to_string(),
            wip_limit: 5,
            task_count: 1,
            version: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_increment_success() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column();
        let col_clone = col.clone();

        mock.expect_increment()
            .times(1)
            .returning(move |_, _| Ok(col_clone.clone()));

        let uc = IncrementColumnUseCase::new(Arc::new(mock));
        let req = IncrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: col.project_id,
            status_code: "open".to_string(),
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().task_count, 1);
    }

    // 異常系: リポジトリがエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_increment_repository_error() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_increment()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("database error").into()));

        let uc = IncrementColumnUseCase::new(Arc::new(mock));
        let req = IncrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "open".to_string(),
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database error"));
    }

    // 異常系: WIP 制限超過エラーがリポジトリから返された場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_increment_wip_limit_exceeded() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_increment()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("WIP limit exceeded").into()));

        let uc = IncrementColumnUseCase::new(Arc::new(mock));
        let req = IncrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "in_progress".to_string(),
        };
        let result = uc.execute("test-tenant", &req).await;
        assert!(result.is_err());
    }

    // 正常系: 異なる tenant_id でインクリメントが実行されることを確認する（テナント分離）
    #[tokio::test]
    async fn test_increment_tenant_isolation() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column();
        let col_clone = col.clone();

        mock.expect_increment()
            .times(1)
            .returning(move |tenant_id, _| {
                // テナントIDが正しく渡されていることを検証する
                assert_eq!(tenant_id, "tenant-xyz");
                Ok(col_clone.clone())
            });

        let uc = IncrementColumnUseCase::new(Arc::new(mock));
        let req = IncrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: col.project_id,
            status_code: "open".to_string(),
        };
        let result = uc.execute("tenant-xyz", &req).await;
        assert!(result.is_ok());
    }
}
