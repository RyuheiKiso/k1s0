use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateSessionInput {
    pub user_id: String,
    pub ttl_seconds: Option<i64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateSessionOutput {
    pub session: Session,
}

pub struct CreateSessionUseCase {
    repo: Arc<dyn SessionRepository>,
    default_ttl: i64,
    max_ttl: i64,
}

impl CreateSessionUseCase {
    pub fn new(repo: Arc<dyn SessionRepository>, default_ttl: i64, max_ttl: i64) -> Self {
        Self {
            repo,
            default_ttl,
            max_ttl,
        }
    }

    pub async fn execute(&self, input: &CreateSessionInput) -> Result<CreateSessionOutput, SessionError> {
        let ttl = input.ttl_seconds.unwrap_or(self.default_ttl);
        if ttl <= 0 || ttl > self.max_ttl {
            return Err(SessionError::InvalidInput(format!(
                "ttl_seconds must be between 1 and {}",
                self.max_ttl
            )));
        }

        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id.clone(),
            token: Uuid::new_v4().to_string(),
            expires_at: now + Duration::seconds(ttl),
            created_at: now,
            revoked: false,
            metadata: input.metadata.clone().unwrap_or_default(),
        };

        self.repo
            .save(&session)
            .await
            .map_err(|e| SessionError::Internal(e.to_string()))?;

        Ok(CreateSessionOutput { session })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::session_repository::MockSessionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_save().returning(|_| Ok(()));

        let uc = CreateSessionUseCase::new(Arc::new(mock), 3600, 86400);
        let input = CreateSessionInput {
            user_id: "user-1".to_string(),
            ttl_seconds: Some(7200),
            metadata: Some(HashMap::from([("ip".to_string(), "127.0.0.1".to_string())])),
        };
        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.session.user_id, "user-1");
        assert!(!result.session.id.is_empty());
        assert!(!result.session.token.is_empty());
        assert!(!result.session.revoked);
        assert_eq!(result.session.metadata.get("ip").unwrap(), "127.0.0.1");
    }

    #[tokio::test]
    async fn default_ttl() {
        let mut mock = MockSessionRepository::new();
        mock.expect_save().returning(|_| Ok(()));

        let uc = CreateSessionUseCase::new(Arc::new(mock), 3600, 86400);
        let input = CreateSessionInput {
            user_id: "user-2".to_string(),
            ttl_seconds: None,
            metadata: None,
        };
        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.session.user_id, "user-2");
    }

    #[tokio::test]
    async fn invalid_ttl() {
        let mock = MockSessionRepository::new();
        let uc = CreateSessionUseCase::new(Arc::new(mock), 3600, 86400);
        let input = CreateSessionInput {
            user_id: "user-3".to_string(),
            ttl_seconds: Some(100000),
            metadata: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn repo_error() {
        let mut mock = MockSessionRepository::new();
        mock.expect_save()
            .returning(|_| Err(SessionError::Internal("db error".to_string())));

        let uc = CreateSessionUseCase::new(Arc::new(mock), 3600, 86400);
        let input = CreateSessionInput {
            user_id: "user-4".to_string(),
            ttl_seconds: Some(3600),
            metadata: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(SessionError::Internal(_))));
    }
}
