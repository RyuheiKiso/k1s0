use std::sync::Arc;

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
}

impl RevokeAllSessionsUseCase {
    pub fn new(repo: Arc<dyn SessionRepository>) -> Self {
        Self { repo }
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
                count += 1;
            }
        }

        Ok(RevokeAllSessionsOutput { count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session(id: &str, revoked: bool) -> Session {
        Session {
            id: id.to_string(),
            user_id: "user-1".to_string(),
            token: format!("tok-{}", id),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
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

        let uc = RevokeAllSessionsUseCase::new(Arc::new(mock));
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

        let uc = RevokeAllSessionsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-2".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 0);
    }
}
