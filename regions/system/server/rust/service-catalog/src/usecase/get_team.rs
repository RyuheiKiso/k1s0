use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::team::Team;
use crate::domain::repository::TeamRepository;

/// GetTeamError はチーム取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetTeamError {
    #[error("team not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetTeamUseCase はチーム取得ユースケース。
pub struct GetTeamUseCase {
    team_repo: Arc<dyn TeamRepository>,
}

impl GetTeamUseCase {
    pub fn new(team_repo: Arc<dyn TeamRepository>) -> Self {
        Self { team_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Team, GetTeamError> {
        self.team_repo
            .find_by_id(id)
            .await
            .map_err(|e| GetTeamError::Internal(e.to_string()))?
            .ok_or(GetTeamError::NotFound(id))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::team_repository::MockTeamRepository;
    use chrono::Utc;

    fn make_team(id: Uuid) -> Team {
        Team {
            id,
            name: "platform".to_string(),
            description: Some("Platform team".to_string()),
            contact_email: Some("platform@example.com".to_string()),
            slack_channel: Some("#platform".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_team_success() {
        let id = Uuid::new_v4();
        let mut mock = MockTeamRepository::new();
        let tid = id;
        mock.expect_find_by_id()
            .withf(move |i| *i == tid)
            .returning(move |_| Ok(Some(make_team(id))));

        let uc = GetTeamUseCase::new(Arc::new(mock));
        let result = uc.execute(id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "platform");
    }

    #[tokio::test]
    async fn test_get_team_not_found() {
        let mut mock = MockTeamRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetTeamUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetTeamError::NotFound(_))));
    }
}
