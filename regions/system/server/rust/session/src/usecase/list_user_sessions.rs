use std::sync::Arc;

use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ListUserSessionsInput {
    pub user_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ListUserSessionsOutput {
    pub sessions: Vec<Session>,
}

pub struct ListUserSessionsUseCase {
    repo: Arc<dyn SessionRepository>,
}

impl ListUserSessionsUseCase {
    pub fn new(repo: Arc<dyn SessionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListUserSessionsInput,
    ) -> Result<ListUserSessionsOutput, SessionError> {
        let sessions = self.repo.find_by_user_id(&input.user_id).await?;
        Ok(ListUserSessionsOutput { sessions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session(id: &str) -> Session {
        Session {
            id: id.to_string(),
            user_id: "user-1".to_string(),
            token: format!("tok-{}", id),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id()
            .returning(|_| Ok(vec![make_session("s1"), make_session("s2")]));

        let uc = ListUserSessionsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&ListUserSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.sessions.len(), 2);
    }

    #[tokio::test]
    async fn empty() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));

        let uc = ListUserSessionsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&ListUserSessionsInput {
                user_id: "user-2".to_string(),
            })
            .await
            .unwrap();
        assert!(result.sessions.is_empty());
    }
}
