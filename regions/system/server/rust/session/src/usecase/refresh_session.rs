use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RefreshSessionInput {
    pub id: String,
    pub ttl_seconds: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RefreshSessionOutput {
    pub session: Session,
}

pub struct RefreshSessionUseCase {
    repo: Arc<dyn SessionRepository>,
    max_ttl: i64,
}

impl RefreshSessionUseCase {
    pub fn new(repo: Arc<dyn SessionRepository>, max_ttl: i64) -> Self {
        Self { repo, max_ttl }
    }

    pub async fn execute(
        &self,
        input: &RefreshSessionInput,
    ) -> Result<RefreshSessionOutput, SessionError> {
        if input.ttl_seconds <= 0 || input.ttl_seconds > self.max_ttl {
            return Err(SessionError::InvalidInput(format!(
                "ttl_seconds must be between 1 and {}",
                self.max_ttl
            )));
        }

        let mut session = self
            .repo
            .find_by_id(&input.id)
            .await?
            .ok_or_else(|| SessionError::NotFound(input.id.clone()))?;

        if session.revoked {
            return Err(SessionError::Revoked(input.id.clone()));
        }

        let new_expires_at = Utc::now() + Duration::seconds(input.ttl_seconds);
        session.refresh(new_expires_at);

        self.repo.save(&session).await?;

        Ok(RefreshSessionOutput { session })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use std::collections::HashMap;

    fn make_session(revoked: bool) -> Session {
        Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_session(false))));
        mock.expect_save().returning(|_| Ok(()));

        let uc = RefreshSessionUseCase::new(Arc::new(mock), 86400);
        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 7200,
            })
            .await
            .unwrap();
        assert_eq!(result.session.id, "sess-1");
    }

    #[tokio::test]
    async fn session_not_found() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = RefreshSessionUseCase::new(Arc::new(mock), 86400);
        let result = uc
            .execute(&RefreshSessionInput {
                id: "missing".to_string(),
                ttl_seconds: 3600,
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn session_revoked() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_session(true))));

        let uc = RefreshSessionUseCase::new(Arc::new(mock), 86400);
        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 3600,
            })
            .await;
        assert!(matches!(result, Err(SessionError::Revoked(_))));
    }

    #[tokio::test]
    async fn invalid_ttl() {
        let mock = MockSessionRepository::new();
        let uc = RefreshSessionUseCase::new(Arc::new(mock), 86400);
        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 0,
            })
            .await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }
}
