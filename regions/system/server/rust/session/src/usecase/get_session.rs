use std::sync::Arc;

use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GetSessionInput {
    pub id: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GetSessionOutput {
    pub session: Session,
}

pub struct GetSessionUseCase {
    repo: Arc<dyn SessionRepository>,
}

impl GetSessionUseCase {
    pub fn new(repo: Arc<dyn SessionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &GetSessionInput) -> Result<GetSessionOutput, SessionError> {
        let session = if let Some(ref id) = input.id {
            self.repo.find_by_id(id).await?
        } else if let Some(ref token) = input.token {
            self.repo.find_by_token(token).await?
        } else {
            return Err(SessionError::InvalidInput(
                "either id or token is required".to_string(),
            ));
        };

        match session {
            Some(s) => Ok(GetSessionOutput { session: s }),
            None => Err(SessionError::NotFound("session not found".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn get_by_id() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_session())));

        let uc = GetSessionUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&GetSessionInput {
                id: Some("sess-1".to_string()),
                token: None,
            })
            .await
            .unwrap();
        assert_eq!(result.session.id, "sess-1");
    }

    #[tokio::test]
    async fn get_by_token() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_token()
            .returning(|_| Ok(Some(make_session())));

        let uc = GetSessionUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&GetSessionInput {
                id: None,
                token: Some("tok-1".to_string()),
            })
            .await
            .unwrap();
        assert_eq!(result.session.token, "tok-1");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetSessionUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&GetSessionInput {
                id: Some("missing".to_string()),
                token: None,
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn no_input() {
        let mock = MockSessionRepository::new();
        let uc = GetSessionUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&GetSessionInput {
                id: None,
                token: None,
            })
            .await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }
}
