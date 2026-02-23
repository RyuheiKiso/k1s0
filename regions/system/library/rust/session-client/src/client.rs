use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::SessionError;
use crate::model::{CreateSessionRequest, RefreshSessionRequest, Session};
use chrono::Utc;
use uuid::Uuid;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait SessionClient: Send + Sync {
    async fn create(&self, req: CreateSessionRequest) -> Result<Session, SessionError>;
    async fn get(&self, id: &str) -> Result<Option<Session>, SessionError>;
    async fn refresh(&self, req: RefreshSessionRequest) -> Result<Session, SessionError>;
    async fn revoke(&self, id: &str) -> Result<(), SessionError>;
    async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<Session>, SessionError>;
    async fn revoke_all(&self, user_id: &str) -> Result<u32, SessionError>;
}

pub struct InMemorySessionClient {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl InMemorySessionClient {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySessionClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionClient for InMemorySessionClient {
    async fn create(&self, req: CreateSessionRequest) -> Result<Session, SessionError> {
        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: req.user_id,
            token: Uuid::new_v4().to_string(),
            expires_at: now + chrono::Duration::seconds(req.ttl_seconds),
            created_at: now,
            revoked: false,
            metadata: req.metadata,
        };
        self.sessions
            .write()
            .await
            .insert(session.id.clone(), session.clone());
        Ok(session)
    }

    async fn get(&self, id: &str) -> Result<Option<Session>, SessionError> {
        Ok(self.sessions.read().await.get(id).cloned())
    }

    async fn refresh(&self, req: RefreshSessionRequest) -> Result<Session, SessionError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&req.id)
            .ok_or_else(|| SessionError::NotFound(req.id.clone()))?;
        session.expires_at = Utc::now() + chrono::Duration::seconds(req.ttl_seconds);
        Ok(session.clone())
    }

    async fn revoke(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| SessionError::NotFound(id.to_string()))?;
        session.revoked = true;
        Ok(())
    }

    async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        Ok(self
            .sessions
            .read()
            .await
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn revoke_all(&self, user_id: &str) -> Result<u32, SessionError> {
        let mut sessions = self.sessions.write().await;
        let mut count = 0u32;
        for session in sessions
            .values_mut()
            .filter(|s| s.user_id == user_id && !s.revoked)
        {
            session.revoked = true;
            count += 1;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create() {
        let client = InMemorySessionClient::new();
        let session = client
            .create(CreateSessionRequest {
                user_id: "user-1".to_string(),
                ttl_seconds: 3600,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        assert_eq!(session.user_id, "user-1");
        assert!(!session.revoked);
        assert!(!session.id.is_empty());
        assert!(!session.token.is_empty());

        let fetched = client.get(&session.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let client = InMemorySessionClient::new();
        let result = client.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_refresh() {
        let client = InMemorySessionClient::new();
        let session = client
            .create(CreateSessionRequest {
                user_id: "user-1".to_string(),
                ttl_seconds: 100,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        let refreshed = client
            .refresh(RefreshSessionRequest {
                id: session.id.clone(),
                ttl_seconds: 7200,
            })
            .await
            .unwrap();

        assert!(refreshed.expires_at > session.expires_at);
    }

    #[tokio::test]
    async fn test_refresh_not_found() {
        let client = InMemorySessionClient::new();
        let result = client
            .refresh(RefreshSessionRequest {
                id: "missing".to_string(),
                ttl_seconds: 3600,
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_revoke() {
        let client = InMemorySessionClient::new();
        let session = client
            .create(CreateSessionRequest {
                user_id: "user-1".to_string(),
                ttl_seconds: 3600,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        client.revoke(&session.id).await.unwrap();

        let fetched = client.get(&session.id).await.unwrap().unwrap();
        assert!(fetched.revoked);
    }

    #[tokio::test]
    async fn test_revoke_not_found() {
        let client = InMemorySessionClient::new();
        let result = client.revoke("missing").await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_user_sessions() {
        let client = InMemorySessionClient::new();
        for _ in 0..2 {
            client
                .create(CreateSessionRequest {
                    user_id: "user-1".to_string(),
                    ttl_seconds: 3600,
                    metadata: HashMap::new(),
                })
                .await
                .unwrap();
        }
        client
            .create(CreateSessionRequest {
                user_id: "user-2".to_string(),
                ttl_seconds: 3600,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        let sessions = client.list_user_sessions("user-1").await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_revoke_all() {
        let client = InMemorySessionClient::new();
        for _ in 0..2 {
            client
                .create(CreateSessionRequest {
                    user_id: "user-1".to_string(),
                    ttl_seconds: 3600,
                    metadata: HashMap::new(),
                })
                .await
                .unwrap();
        }

        let count = client.revoke_all("user-1").await.unwrap();
        assert_eq!(count, 2);

        let sessions = client.list_user_sessions("user-1").await.unwrap();
        assert!(sessions.iter().all(|s| s.revoked));
    }

    #[test]
    fn test_default() {
        let client = InMemorySessionClient::default();
        assert!(Arc::strong_count(&client.sessions) == 1);
    }
}
