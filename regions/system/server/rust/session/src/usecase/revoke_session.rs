use std::sync::Arc;

use crate::domain::repository::SessionRepository;
use crate::error::SessionError;
use crate::infrastructure::kafka_producer::SessionEventPublisher;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RevokeSessionInput {
    pub id: String,
}

pub struct RevokeSessionUseCase {
    repo: Arc<dyn SessionRepository>,
    event_publisher: Arc<dyn SessionEventPublisher>,
}

impl RevokeSessionUseCase {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        event_publisher: Arc<dyn SessionEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &RevokeSessionInput) -> Result<(), SessionError> {
        let mut session = self
            .repo
            .find_by_id(&input.id)
            .await?
            .ok_or_else(|| SessionError::NotFound(input.id.clone()))?;

        if session.revoked {
            return Err(SessionError::AlreadyRevoked(input.id.clone()));
        }

        session.revoke();
        self.repo.save(&session).await?;
        self.event_publisher
            .publish_session_revoked(&session.id, &session.user_id)
            .await
            .map_err(|e| SessionError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use crate::infrastructure::kafka_producer::MockSessionEventPublisher;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session() -> Session {
        Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            device_id: "device-1".to_string(),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
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
        let mut mock_publisher = MockSessionEventPublisher::new();
        mock_publisher
            .expect_publish_session_revoked()
            .withf(|session_id, user_id| session_id == "sess-1" && user_id == "user-1")
            .returning(|_, _| Ok(()));

        let uc = RevokeSessionUseCase::new(Arc::new(mock), Arc::new(mock_publisher));
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

        let uc = RevokeSessionUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
        );
        let result = uc
            .execute(&RevokeSessionInput {
                id: "missing".to_string(),
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn already_revoked() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| {
            let mut s = make_session();
            s.revoked = true;
            Ok(Some(s))
        });

        let uc = RevokeSessionUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
        );
        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
            })
            .await;
        assert!(matches!(result, Err(SessionError::AlreadyRevoked(_))));
    }
}
