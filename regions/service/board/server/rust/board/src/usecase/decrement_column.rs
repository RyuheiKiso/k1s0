// カラムタスク数減少ユースケース（在庫解放に相当）。
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
    pub async fn execute(&self, req: &DecrementColumnRequest) -> anyhow::Result<BoardColumn> {
        self.repo.decrement(req).await
    }
}
