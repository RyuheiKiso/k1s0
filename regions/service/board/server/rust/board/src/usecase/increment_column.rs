// カラムタスク数増加ユースケース（在庫予約に相当）。
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
    pub async fn execute(&self, req: &IncrementColumnRequest) -> anyhow::Result<BoardColumn> {
        self.repo.increment(req).await
    }
}

#[cfg(test)]
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
            .returning(move |_| Ok(col_clone.clone()));

        let uc = IncrementColumnUseCase::new(Arc::new(mock));
        let req = IncrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: col.project_id,
            status_code: "open".to_string(),
        };
        let result = uc.execute(&req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().task_count, 1);
    }
}
