use std::sync::Arc;

use crate::domain::entity::team::Team;
use crate::domain::repository::TeamRepository;

/// ListTeamsUseCase はチーム一覧取得ユースケース。
pub struct ListTeamsUseCase {
    team_repo: Arc<dyn TeamRepository>,
}

impl ListTeamsUseCase {
    pub fn new(team_repo: Arc<dyn TeamRepository>) -> Self {
        Self { team_repo }
    }

    pub async fn execute(&self) -> anyhow::Result<Vec<Team>> {
        self.team_repo.list().await
    }
}
