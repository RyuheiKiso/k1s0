// ボードカラム一覧ユースケース。
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
        filter: &BoardColumnFilter,
    ) -> anyhow::Result<(Vec<BoardColumn>, i64)> {
        let cols = self.repo.find_all(filter).await?;
        let count = self.repo.count(filter).await?;
        Ok((cols, count))
    }
}
