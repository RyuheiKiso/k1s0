use async_trait::async_trait;

use crate::domain::entity::session::Session;
use crate::error::SessionError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &Session) -> Result<(), SessionError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError>;
    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError>;
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError>;
    async fn delete(&self, id: &str) -> Result<(), SessionError>;
}
