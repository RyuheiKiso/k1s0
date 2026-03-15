use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};
use crate::domain::repository::{ServiceRepository, TeamRepository};

/// RegisterServiceError はサービス登録に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum RegisterServiceError {
    #[error("team not found: {0}")]
    TeamNotFound(Uuid),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// RegisterServiceInput は新規サービス登録の入力データ。
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterServiceInput {
    pub name: String,
    pub description: Option<String>,
    pub team_id: Uuid,
    pub tier: ServiceTier,
    pub lifecycle: ServiceLifecycle,
    pub repository_url: Option<String>,
    pub api_endpoint: Option<String>,
    pub healthcheck_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

fn default_metadata() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

/// RegisterServiceUseCase はサービス登録ユースケース。
pub struct RegisterServiceUseCase {
    service_repo: Arc<dyn ServiceRepository>,
    team_repo: Arc<dyn TeamRepository>,
}

impl RegisterServiceUseCase {
    pub fn new(
        service_repo: Arc<dyn ServiceRepository>,
        team_repo: Arc<dyn TeamRepository>,
    ) -> Self {
        Self {
            service_repo,
            team_repo,
        }
    }

    pub async fn execute(
        &self,
        input: RegisterServiceInput,
    ) -> Result<Service, RegisterServiceError> {
        // Validate name
        if input.name.trim().is_empty() {
            return Err(RegisterServiceError::InvalidInput(
                "service name must not be empty".to_string(),
            ));
        }

        // Validate team exists
        match self.team_repo.find_by_id(input.team_id).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(RegisterServiceError::TeamNotFound(input.team_id)),
            Err(e) => return Err(RegisterServiceError::Internal(e.to_string())),
        }

        let now = Utc::now();
        let service = Service {
            id: Uuid::new_v4(),
            name: input.name,
            description: input.description,
            team_id: input.team_id,
            tier: input.tier,
            lifecycle: input.lifecycle,
            repository_url: input.repository_url,
            api_endpoint: input.api_endpoint,
            healthcheck_url: input.healthcheck_url,
            tags: input.tags,
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
        };

        self.service_repo
            .create(&service)
            .await
            .map_err(|e| RegisterServiceError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::team::Team;
    use crate::domain::repository::service_repository::MockServiceRepository;
    use crate::domain::repository::team_repository::MockTeamRepository;

    #[tokio::test]
    async fn test_register_service_team_not_found() {
        let mock_svc = MockServiceRepository::new();
        let mut mock_team = MockTeamRepository::new();
        mock_team.expect_find_by_id().returning(|_| Ok(None));

        let uc = RegisterServiceUseCase::new(Arc::new(mock_svc), Arc::new(mock_team));
        let input = RegisterServiceInput {
            name: "test".to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Development,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec![],
            metadata: serde_json::json!({}),
        };
        let result = uc.execute(input).await;
        assert!(matches!(result, Err(RegisterServiceError::TeamNotFound(_))));
    }

    #[tokio::test]
    async fn test_register_service_empty_name() {
        let mock_svc = MockServiceRepository::new();
        let mock_team = MockTeamRepository::new();

        let uc = RegisterServiceUseCase::new(Arc::new(mock_svc), Arc::new(mock_team));
        let input = RegisterServiceInput {
            name: "  ".to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Development,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec![],
            metadata: serde_json::json!({}),
        };
        let result = uc.execute(input).await;
        assert!(matches!(result, Err(RegisterServiceError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_register_service_success() {
        let mut mock_svc = MockServiceRepository::new();
        mock_svc.expect_create().returning(|s| Ok(s.clone()));

        let team_id = Uuid::new_v4();
        let mut mock_team = MockTeamRepository::new();
        mock_team.expect_find_by_id().returning(move |_| {
            Ok(Some(Team {
                id: team_id,
                name: "Test Team".to_string(),
                description: None,
                contact_email: None,
                slack_channel: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

        let uc = RegisterServiceUseCase::new(Arc::new(mock_svc), Arc::new(mock_team));
        let input = RegisterServiceInput {
            name: "my-service".to_string(),
            description: Some("A test service".to_string()),
            team_id,
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Development,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec!["test".to_string()],
            metadata: serde_json::json!({}),
        };
        let result = uc.execute(input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "my-service");
    }
}
