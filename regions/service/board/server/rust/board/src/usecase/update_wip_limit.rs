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

    pub async fn execute(&self, req: &UpdateWipLimitRequest) -> anyhow::Result<BoardColumn> {
        BoardService::validate_wip_limit(req.wip_limit)?;
        self.repo.update_wip_limit(req).await
    }
}
