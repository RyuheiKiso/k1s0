use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::team::Team;
use crate::domain::repository::TeamRepository;

/// CreateTeamInput はチーム作成の入力パラメータ。
pub struct CreateTeamInput {
    pub name: String,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub slack_channel: Option<String>,
}

/// CreateTeamError はチーム作成に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum CreateTeamError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// CreateTeamUseCase はチーム作成ユースケース。
pub struct CreateTeamUseCase {
    team_repo: Arc<dyn TeamRepository>,
}

impl CreateTeamUseCase {
    pub fn new(team_repo: Arc<dyn TeamRepository>) -> Self {
        Self { team_repo }
    }

    pub async fn execute(&self, input: CreateTeamInput) -> Result<Team, CreateTeamError> {
        if input.name.trim().is_empty() {
            return Err(CreateTeamError::Validation(
                "name must not be empty".to_string(),
            ));
        }

        let now = Utc::now();
        let team = Team {
            id: Uuid::new_v4(),
            name: input.name,
            description: input.description,
            contact_email: input.contact_email,
            slack_channel: input.slack_channel,
            created_at: now,
            updated_at: now,
        };

        self.team_repo
            .create(&team)
            .await
            .map_err(|e| CreateTeamError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::team_repository::MockTeamRepository;

    #[tokio::test]
    async fn test_create_team_success() {
        let mut mock = MockTeamRepository::new();
        mock.expect_create().returning(|t| Ok(t.clone()));

        let uc = CreateTeamUseCase::new(Arc::new(mock));
        let result = uc
            .execute(CreateTeamInput {
                name: "platform".to_string(),
                description: Some("Platform team".to_string()),
                contact_email: None,
                slack_channel: None,
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "platform");
    }

    #[tokio::test]
    async fn test_create_team_empty_name() {
        let mock = MockTeamRepository::new();
        let uc = CreateTeamUseCase::new(Arc::new(mock));
        let result = uc
            .execute(CreateTeamInput {
                name: "".to_string(),
                description: None,
                contact_email: None,
                slack_channel: None,
            })
            .await;
        assert!(matches!(result, Err(CreateTeamError::Validation(_))));
    }

    #[tokio::test]
    async fn test_create_team_repo_error() {
        let mut mock = MockTeamRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateTeamUseCase::new(Arc::new(mock));
        let result = uc
            .execute(CreateTeamInput {
                name: "platform".to_string(),
                description: None,
                contact_email: None,
                slack_channel: None,
            })
            .await;
        assert!(matches!(result, Err(CreateTeamError::Internal(_))));
    }
}
