use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::team::Team;
use crate::domain::repository::TeamRepository;

/// `UpdateTeamInput` はチーム更新の入力パラメータ。
pub struct UpdateTeamInput {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub slack_channel: Option<String>,
}

/// `UpdateTeamError` はチーム更新に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum UpdateTeamError {
    #[error("team not found: {0}")]
    NotFound(Uuid),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `UpdateTeamUseCase` はチーム更新ユースケース。
pub struct UpdateTeamUseCase {
    team_repo: Arc<dyn TeamRepository>,
}

impl UpdateTeamUseCase {
    pub fn new(team_repo: Arc<dyn TeamRepository>) -> Self {
        Self { team_repo }
    }

    pub async fn execute(&self, input: UpdateTeamInput) -> Result<Team, UpdateTeamError> {
        if input.name.trim().is_empty() {
            return Err(UpdateTeamError::Validation(
                "name must not be empty".to_string(),
            ));
        }

        let existing = self
            .team_repo
            .find_by_id(input.id)
            .await
            .map_err(|e| UpdateTeamError::Internal(e.to_string()))?
            .ok_or(UpdateTeamError::NotFound(input.id))?;

        let team = Team {
            id: existing.id,
            name: input.name,
            description: input.description,
            contact_email: input.contact_email,
            slack_channel: input.slack_channel,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };

        self.team_repo
            .update(&team)
            .await
            .map_err(|e| UpdateTeamError::Internal(e.to_string()))
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
            name: "old-name".to_string(),
            description: None,
            contact_email: None,
            slack_channel: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_update_team_success() {
        let id = Uuid::new_v4();
        let mut mock = MockTeamRepository::new();
        let tid = id;
        mock.expect_find_by_id()
            .withf(move |i| *i == tid)
            .returning(move |_| Ok(Some(make_team(id))));
        mock.expect_update().returning(|t| Ok(t.clone()));

        let uc = UpdateTeamUseCase::new(Arc::new(mock));
        let result = uc
            .execute(UpdateTeamInput {
                id,
                name: "new-name".to_string(),
                description: Some("Updated".to_string()),
                contact_email: None,
                slack_channel: None,
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "new-name");
    }

    #[tokio::test]
    async fn test_update_team_not_found() {
        let mut mock = MockTeamRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateTeamUseCase::new(Arc::new(mock));
        let result = uc
            .execute(UpdateTeamInput {
                id: Uuid::new_v4(),
                name: "name".to_string(),
                description: None,
                contact_email: None,
                slack_channel: None,
            })
            .await;
        assert!(matches!(result, Err(UpdateTeamError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_team_empty_name() {
        let mock = MockTeamRepository::new();
        let uc = UpdateTeamUseCase::new(Arc::new(mock));
        let result = uc
            .execute(UpdateTeamInput {
                id: Uuid::new_v4(),
                name: "".to_string(),
                description: None,
                contact_email: None,
                slack_channel: None,
            })
            .await;
        assert!(matches!(result, Err(UpdateTeamError::Validation(_))));
    }
}
