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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::team_repository::MockTeamRepository;
    use chrono::Utc;

    fn sample_team(name: &str) -> Team {
        Team {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            description: None,
            contact_email: None,
            slack_channel: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// チームが存在する場合は一覧を返す
    #[tokio::test]
    async fn returns_teams() {
        let mut mock = MockTeamRepository::new();
        mock.expect_list().returning(|| {
            Ok(vec![sample_team("platform"), sample_team("backend")])
        });

        let uc = ListTeamsUseCase::new(Arc::new(mock));
        let result = uc.execute().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "platform");
    }

    /// チームが存在しない場合は空の一覧を返す
    #[tokio::test]
    async fn returns_empty_when_no_teams() {
        let mut mock = MockTeamRepository::new();
        mock.expect_list().returning(|| Ok(vec![]));

        let uc = ListTeamsUseCase::new(Arc::new(mock));
        let result = uc.execute().await.unwrap();
        assert!(result.is_empty());
    }
}
