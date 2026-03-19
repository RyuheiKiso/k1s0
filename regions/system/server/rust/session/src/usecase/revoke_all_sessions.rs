use std::sync::Arc;

use crate::adapter::repository::session_metadata_postgres::SessionMetadataRepository;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RevokeAllSessionsInput {
    pub user_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RevokeAllSessionsOutput {
    pub count: u32,
}

pub struct RevokeAllSessionsUseCase {
    repo: Arc<dyn SessionRepository>,
    metadata_repo: Arc<dyn SessionMetadataRepository>,
}

impl RevokeAllSessionsUseCase {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        metadata_repo: Arc<dyn SessionMetadataRepository>,
    ) -> Self {
        Self {
            repo,
            metadata_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &RevokeAllSessionsInput,
    ) -> Result<RevokeAllSessionsOutput, SessionError> {
        let sessions = self.repo.find_by_user_id(&input.user_id).await?;
        let mut count = 0u32;

        for mut session in sessions {
            if !session.revoked {
                session.revoke();
                self.repo.save(&session).await?;

                // Mark metadata as revoked
                if let Ok(session_uuid) = uuid::Uuid::parse_str(&session.id) {
                    let _ = self.metadata_repo.mark_revoked(session_uuid).await;
                }

                count += 1;
            }
        }

        Ok(RevokeAllSessionsOutput { count })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session(id: &str, revoked: bool) -> Session {
        Session {
            id: id.to_string(),
            user_id: "user-1".to_string(),
            device_id: format!("device-{}", id),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: format!("tok-{}", id),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
            revoked,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| {
            Ok(vec![
                make_session("s1", false),
                make_session("s2", false),
                make_session("s3", true),
            ])
        });
        mock.expect_save().returning(|_| Ok(()));

        let uc =
            RevokeAllSessionsUseCase::new(Arc::new(mock), Arc::new(NoopSessionMetadataRepository));
        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 2);
    }

    #[tokio::test]
    async fn no_sessions() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));

        let uc =
            RevokeAllSessionsUseCase::new(Arc::new(mock), Arc::new(NoopSessionMetadataRepository));
        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-2".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 0);
    }
}
