// WIP 制限更新ユースケース。楽観的ロック付き。
use crate::domain::entity::board_column::{BoardColumn, UpdateWipLimitRequest};
use crate::domain::repository::board_column_repository::BoardColumnRepository;
use crate::domain::service::board_service::BoardService;
use std::sync::Arc;

pub struct UpdateWipLimitUseCase {
    repo: Arc<dyn BoardColumnRepository>,
}

impl UpdateWipLimitUseCase {
    pub fn new(repo: Arc<dyn BoardColumnRepository>) -> Self {
        Self { repo }
    }

    // WIP 制限更新の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, req: &UpdateWipLimitRequest) -> anyhow::Result<BoardColumn> {
        BoardService::validate_wip_limit(req.wip_limit)?;
        self.repo.update_wip_limit(req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::board_column_repository::MockBoardColumnRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用のサンプルカラムを生成するヘルパー関数
    fn sample_column(wip_limit: i32) -> BoardColumn {
        BoardColumn {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "in_progress".to_string(),
            wip_limit,
            task_count: 0,
            version: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // 正常系: 有効な WIP リミット値で更新が成功することを確認する
    #[tokio::test]
    async fn test_update_wip_limit_success() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column(10);
        let col_clone = col.clone();

        mock.expect_update_wip_limit()
            .times(1)
            .returning(move |_| Ok(col_clone.clone()));

        let uc = UpdateWipLimitUseCase::new(Arc::new(mock));
        let req = UpdateWipLimitRequest {
            column_id: col.id,
            wip_limit: 10,
            expected_version: 1,
        };
        let result = uc.execute(&req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().wip_limit, 10);
    }

    // 正常系: wip_limit=0（無制限）で更新が成功することを確認する
    #[tokio::test]
    async fn test_update_wip_limit_zero_unlimited() {
        let mut mock = MockBoardColumnRepository::new();
        let col = sample_column(0);
        let col_clone = col.clone();

        mock.expect_update_wip_limit()
            .times(1)
            .returning(move |_| Ok(col_clone.clone()));

        let uc = UpdateWipLimitUseCase::new(Arc::new(mock));
        let req = UpdateWipLimitRequest {
            column_id: col.id,
            wip_limit: 0,
            expected_version: 1,
        };
        let result = uc.execute(&req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().wip_limit, 0);
    }

    // 異常系: wip_limit が負の値の場合にリポジトリを呼ばずにバリデーションエラーが返ることを確認する
    #[tokio::test]
    async fn test_update_wip_limit_invalid_negative() {
        let mock = MockBoardColumnRepository::new();
        // update_wip_limit は呼ばれないはず（バリデーション前にエラー）

        let uc = UpdateWipLimitUseCase::new(Arc::new(mock));
        let req = UpdateWipLimitRequest {
            column_id: Uuid::new_v4(),
            wip_limit: -1,
            expected_version: 1,
        };
        let result = uc.execute(&req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("wip_limit must be >= 0"));
    }

    // 異常系: 対象カラムが存在しない場合（NotFound）にエラーが返ることを確認する
    #[tokio::test]
    async fn test_update_wip_limit_not_found() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_update_wip_limit()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("column not found")));

        let uc = UpdateWipLimitUseCase::new(Arc::new(mock));
        let req = UpdateWipLimitRequest {
            column_id: Uuid::new_v4(),
            wip_limit: 5,
            expected_version: 1,
        };
        let result = uc.execute(&req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("column not found"));
    }

    // 異常系: リポジトリがデータベースエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_update_wip_limit_repository_error() {
        let mut mock = MockBoardColumnRepository::new();

        mock.expect_update_wip_limit()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("database error")));

        let uc = UpdateWipLimitUseCase::new(Arc::new(mock));
        let req = UpdateWipLimitRequest {
            column_id: Uuid::new_v4(),
            wip_limit: 3,
            expected_version: 2,
        };
        let result = uc.execute(&req).await;
        assert!(result.is_err());
    }
}
