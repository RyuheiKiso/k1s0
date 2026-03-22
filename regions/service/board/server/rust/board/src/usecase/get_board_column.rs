// ボードカラム取得ユースケース。
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
    pub async fn execute(&self, id: Uuid) -> anyhow::Result<Option<BoardColumn>> {
        self.repo.find_by_id(id).await
    }
}
