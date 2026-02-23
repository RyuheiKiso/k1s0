use std::sync::Arc;

use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RevokeSessionInput {
    pub id: String,
}

pub struct RevokeSessionUseCase {
    repo: Arc<dyn SessionRepository>,
}

impl RevokeSessionUseCase {
    pub fn new(repo: Arc<dyn SessionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &RevokeSessionInput) -> Result<(), SessionError> {
        let mut session = self
            .repo
            .find_by_id(&input.id)
            .await?
            .ok_or_else(|| SessionError::NotFound(input.id.clone()))?;

        session.revoke();
        self.repo.save(&session).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session() -> Session {
        Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_session())));
        mock.expect_save().returning(|_| Ok(()));

        let uc = RevokeSessionUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = RevokeSessionUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&RevokeSessionInput {
                id: "missing".to_string(),
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }
}
