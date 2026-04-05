#![allow(clippy::unwrap_used)]
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use tokio::sync::RwLock;

use k1s0_session_server::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
use k1s0_session_server::domain::entity::session::Session;
use k1s0_session_server::domain::repository::SessionRepository;
use k1s0_session_server::error::SessionError;
use k1s0_session_server::infrastructure::kafka_producer::SessionEventPublisher;
use k1s0_session_server::usecase::create_session::{CreateSessionInput, CreateSessionUseCase};
use k1s0_session_server::usecase::get_session::{GetSessionInput, GetSessionUseCase};
use k1s0_session_server::usecase::list_user_sessions::{
    ListUserSessionsInput, ListUserSessionsUseCase,
};
use k1s0_session_server::usecase::refresh_session::{RefreshSessionInput, RefreshSessionUseCase};
use k1s0_session_server::usecase::revoke_all_sessions::{
    RevokeAllSessionsInput, RevokeAllSessionsUseCase,
};
use k1s0_session_server::usecase::revoke_session::{RevokeSessionInput, RevokeSessionUseCase};

// ---------------------------------------------------------------------------
// In-memory stub: SessionRepository
// ---------------------------------------------------------------------------
struct StubSessionRepository {
    sessions: RwLock<Vec<Session>>,
}

impl StubSessionRepository {
    fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }

    fn with_sessions(sessions: Vec<Session>) -> Self {
        Self {
            sessions: RwLock::new(sessions),
        }
    }
}

#[async_trait]
impl SessionRepository for StubSessionRepository {
    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        if let Some(pos) = sessions.iter().position(|s| s.id == session.id) {
            sessions[pos] = session.clone();
        } else {
            sessions.push(session.clone());
        }
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.iter().find(|s| s.id == id).cloned())
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.iter().find(|s| s.token == token).cloned())
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .iter()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|s| s.id != id);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: SessionEventPublisher
// ---------------------------------------------------------------------------
struct StubSessionEventPublisher {
    events: RwLock<Vec<String>>,
}

impl StubSessionEventPublisher {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl SessionEventPublisher for StubSessionEventPublisher {
    async fn publish_session_created(&self, session: &Session) -> anyhow::Result<()> {
        self.events
            .write()
            .await
            .push(format!("created:{}", session.id));
        Ok(())
    }

    async fn publish_session_revoked(&self, session_id: &str, user_id: &str) -> anyhow::Result<()> {
        self.events
            .write()
            .await
            .push(format!("revoked:{}:{}", session_id, user_id));
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper: build a Session
// ---------------------------------------------------------------------------
/// テスト用セッション生成ヘルパー。tenant_id は "tenant-a" 固定。
fn make_session(id: &str, user_id: &str, revoked: bool) -> Session {
    Session {
        id: id.to_string(),
        user_id: user_id.to_string(),
        tenant_id: "tenant-a".to_string(),
        device_id: format!("device-{}", id),
        device_name: Some("Test Device".to_string()),
        device_type: Some("desktop".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        token: format!("tok-{}", id),
        expires_at: Utc::now() + Duration::hours(1),
        created_at: Utc::now(),
        last_accessed_at: Some(Utc::now()),
        revoked,
        metadata: HashMap::new(),
    }
}

/// テスト用期限切れセッション生成ヘルパー。tenant_id は "tenant-a" 固定。
fn make_expired_session(id: &str, user_id: &str) -> Session {
    Session {
        id: id.to_string(),
        user_id: user_id.to_string(),
        tenant_id: "tenant-a".to_string(),
        device_id: format!("device-{}", id),
        device_name: Some("Test Device".to_string()),
        device_type: Some("desktop".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        token: format!("tok-{}", id),
        expires_at: Utc::now() - Duration::hours(1),
        created_at: Utc::now() - Duration::hours(2),
        last_accessed_at: None,
        revoked: false,
        metadata: HashMap::new(),
    }
}

// ===========================================================================
// CreateSession
// ===========================================================================
mod create_session {
    use super::*;

    #[tokio::test]
    async fn success_creates_new_session() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = CreateSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher.clone(),
            3600,
            86400,
        );

        let input = CreateSessionInput {
            user_id: "user-1".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "device-1".to_string(),
            device_name: Some("My Laptop".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            ttl_seconds: Some(7200),
            max_devices: None,
            metadata: Some(HashMap::from([("os".to_string(), "linux".to_string())])),
        };

        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.session.user_id, "user-1");
        assert_eq!(result.session.device_id, "device-1");
        assert!(!result.session.id.is_empty());
        assert!(!result.session.token.is_empty());
        assert!(!result.session.revoked);
        assert_eq!(result.session.metadata.get("os").unwrap(), "linux");

        // Verify persisted
        let stored = repo.find_by_id(&result.session.id).await.unwrap();
        assert!(stored.is_some());

        // Verify event published
        let events = publisher.events.read().await;
        assert_eq!(events.len(), 1);
        assert!(events[0].starts_with("created:"));
    }

    #[tokio::test]
    async fn uses_default_ttl_when_none() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = CreateSessionUseCase::new(
            repo,
            Arc::new(NoopSessionMetadataRepository),
            publisher,
            3600,
            86400,
        );

        let input = CreateSessionInput {
            user_id: "user-2".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "device-2".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: None,
            max_devices: None,
            metadata: None,
        };

        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.session.user_id, "user-2");
        // Session should be valid (default TTL of 3600s)
        assert!(result.session.is_valid());
    }

    #[tokio::test]
    async fn rejects_ttl_exceeding_max() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = CreateSessionUseCase::new(
            repo,
            Arc::new(NoopSessionMetadataRepository),
            publisher,
            3600,
            86400,
        );

        let input = CreateSessionInput {
            user_id: "user-3".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "device-3".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(100000), // exceeds max_ttl of 86400
            max_devices: None,
            metadata: None,
        };

        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn rejects_empty_device_id() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = CreateSessionUseCase::new(
            repo,
            Arc::new(NoopSessionMetadataRepository),
            publisher,
            3600,
            86400,
        );

        let input = CreateSessionInput {
            user_id: "user-4".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "  ".to_string(), // blank
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(3600),
            max_devices: None,
            metadata: None,
        };

        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn revokes_oldest_sessions_when_over_max_devices() {
        // Pre-populate with 3 valid sessions, max_devices = 3
        let sessions = vec![
            make_session("s1", "user-busy", false),
            make_session("s2", "user-busy", false),
            make_session("s3", "user-busy", false),
        ];
        let repo = Arc::new(StubSessionRepository::with_sessions(sessions));
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = CreateSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher,
            3600,
            86400,
        );

        let input = CreateSessionInput {
            user_id: "user-busy".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "device-new".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(3600),
            max_devices: Some(3), // limit is 3, already have 3, so oldest gets revoked
            metadata: None,
        };

        let result = uc.execute(&input).await.unwrap();
        assert!(!result.session.revoked);

        // s1 should be revoked (oldest), but the new session should be valid
        let s1 = repo.find_by_id("s1").await.unwrap().unwrap();
        assert!(s1.revoked);
    }

    #[tokio::test]
    async fn expired_sessions_not_counted_toward_limit() {
        // 2 valid + 1 expired, max_devices = 3
        let sessions = vec![
            make_session("s1", "user-x", false),
            make_session("s2", "user-x", false),
            make_expired_session("s3", "user-x"),
        ];
        let repo = Arc::new(StubSessionRepository::with_sessions(sessions));
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = CreateSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher,
            3600,
            86400,
        );

        let input = CreateSessionInput {
            user_id: "user-x".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: "device-new".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(3600),
            max_devices: Some(3),
            metadata: None,
        };

        let result = uc.execute(&input).await.unwrap();
        assert!(!result.session.revoked);

        // No existing session should be revoked (only 2 valid, limit is 3)
        let s1 = repo.find_by_id("s1").await.unwrap().unwrap();
        let s2 = repo.find_by_id("s2").await.unwrap().unwrap();
        assert!(!s1.revoked);
        assert!(!s2.revoked);
    }
}

// ===========================================================================
// GetSession
// ===========================================================================
mod get_session {
    use super::*;

    #[tokio::test]
    async fn get_by_id() {
        let session = make_session("sess-1", "user-1", false);
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let uc = GetSessionUseCase::new(repo);

        let result = uc
            .execute(&GetSessionInput {
                id: Some("sess-1".to_string()),
                token: None,
            })
            .await
            .unwrap();
        assert_eq!(result.session.id, "sess-1");
        assert_eq!(result.session.user_id, "user-1");
    }

    #[tokio::test]
    async fn get_by_token() {
        let session = make_session("sess-1", "user-1", false);
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let uc = GetSessionUseCase::new(repo);

        let result = uc
            .execute(&GetSessionInput {
                id: None,
                token: Some("tok-sess-1".to_string()),
            })
            .await
            .unwrap();
        assert_eq!(result.session.token, "tok-sess-1");
    }

    #[tokio::test]
    async fn not_found_by_id() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = GetSessionUseCase::new(repo);

        let result = uc
            .execute(&GetSessionInput {
                id: Some("missing".to_string()),
                token: None,
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn not_found_by_token() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = GetSessionUseCase::new(repo);

        let result = uc
            .execute(&GetSessionInput {
                id: None,
                token: Some("bad-token".to_string()),
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn no_input_returns_invalid_input() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = GetSessionUseCase::new(repo);

        let result = uc
            .execute(&GetSessionInput {
                id: None,
                token: None,
            })
            .await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn id_takes_priority_over_token() {
        let session = make_session("sess-1", "user-1", false);
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let uc = GetSessionUseCase::new(repo);

        // Both id and token provided; id should be used
        let result = uc
            .execute(&GetSessionInput {
                id: Some("sess-1".to_string()),
                token: Some("non-existent-token".to_string()),
            })
            .await
            .unwrap();
        assert_eq!(result.session.id, "sess-1");
    }
}

// ===========================================================================
// RefreshSession
// ===========================================================================
mod refresh_session {
    use super::*;

    #[tokio::test]
    async fn success_extends_expiry() {
        let session = make_session("sess-1", "user-1", false);
        let original_expires = session.expires_at;
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let uc = RefreshSessionUseCase::new(repo.clone(), 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 7200,
            })
            .await
            .unwrap();

        assert_eq!(result.session.id, "sess-1");
        assert!(result.session.expires_at > original_expires);

        // Verify persisted
        let stored = repo.find_by_id("sess-1").await.unwrap().unwrap();
        assert_eq!(stored.expires_at, result.session.expires_at);
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = RefreshSessionUseCase::new(repo, 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "missing".to_string(),
                ttl_seconds: 3600,
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn rejects_revoked_session() {
        let session = make_session("sess-1", "user-1", true);
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let uc = RefreshSessionUseCase::new(repo, 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 3600,
            })
            .await;
        assert!(matches!(result, Err(SessionError::Revoked(_))));
    }

    #[tokio::test]
    async fn rejects_zero_ttl() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = RefreshSessionUseCase::new(repo, 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 0,
            })
            .await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn rejects_negative_ttl() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = RefreshSessionUseCase::new(repo, 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: -100,
            })
            .await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn rejects_ttl_exceeding_max() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = RefreshSessionUseCase::new(repo, 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 100000,
            })
            .await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn can_refresh_expired_but_not_revoked() {
        let session = make_expired_session("sess-1", "user-1");
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let uc = RefreshSessionUseCase::new(repo.clone(), 86400);

        let result = uc
            .execute(&RefreshSessionInput {
                id: "sess-1".to_string(),
                ttl_seconds: 3600,
            })
            .await
            .unwrap();

        // After refresh, the session should no longer be expired
        assert!(!result.session.is_expired());
    }
}

// ===========================================================================
// RevokeSession
// ===========================================================================
mod revoke_session {
    use super::*;

    #[tokio::test]
    async fn success() {
        let session = make_session("sess-1", "user-1", false);
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc = RevokeSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher.clone(),
        );

        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
                jwt_jti: None,
                jwt_remaining_secs: None,
            })
            .await;
        assert!(result.is_ok());

        // Verify persisted as revoked
        let stored = repo.find_by_id("sess-1").await.unwrap().unwrap();
        assert!(stored.revoked);

        // Verify event published
        let events = publisher.events.read().await;
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("revoked:sess-1:user-1"));
    }

    #[tokio::test]
    async fn not_found() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc =
            RevokeSessionUseCase::new(repo, Arc::new(NoopSessionMetadataRepository), publisher);

        let result = uc
            .execute(&RevokeSessionInput {
                id: "missing".to_string(),
                jwt_jti: None,
                jwt_remaining_secs: None,
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn already_revoked_returns_error() {
        let session = make_session("sess-1", "user-1", true);
        let repo = Arc::new(StubSessionRepository::with_sessions(vec![session]));
        let publisher = Arc::new(StubSessionEventPublisher::new());
        let uc =
            RevokeSessionUseCase::new(repo, Arc::new(NoopSessionMetadataRepository), publisher);

        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
                jwt_jti: None,
                jwt_remaining_secs: None,
            })
            .await;
        assert!(matches!(result, Err(SessionError::AlreadyRevoked(_))));
    }
}

// ===========================================================================
// RevokeAllSessions
// ===========================================================================
mod revoke_all_sessions {
    use super::*;

    #[tokio::test]
    async fn revokes_only_active_sessions() {
        let sessions = vec![
            make_session("s1", "user-1", false),
            make_session("s2", "user-1", false),
            make_session("s3", "user-1", true), // already revoked
        ];
        let repo = Arc::new(StubSessionRepository::with_sessions(sessions));
        let uc =
            RevokeAllSessionsUseCase::new(repo.clone(), Arc::new(NoopSessionMetadataRepository));

        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.count, 2);

        // Verify all are now revoked
        let all = repo.find_by_user_id("user-1").await.unwrap();
        assert!(all.iter().all(|s| s.revoked));
    }

    #[tokio::test]
    async fn returns_zero_when_no_sessions() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = RevokeAllSessionsUseCase::new(repo, Arc::new(NoopSessionMetadataRepository));

        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-none".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.count, 0);
    }

    #[tokio::test]
    async fn returns_zero_when_all_already_revoked() {
        let sessions = vec![
            make_session("s1", "user-1", true),
            make_session("s2", "user-1", true),
        ];
        let repo = Arc::new(StubSessionRepository::with_sessions(sessions));
        let uc = RevokeAllSessionsUseCase::new(repo, Arc::new(NoopSessionMetadataRepository));

        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.count, 0);
    }

    #[tokio::test]
    async fn does_not_affect_other_users() {
        let sessions = vec![
            make_session("s1", "user-1", false),
            make_session("s2", "user-2", false),
        ];
        let repo = Arc::new(StubSessionRepository::with_sessions(sessions));
        let uc =
            RevokeAllSessionsUseCase::new(repo.clone(), Arc::new(NoopSessionMetadataRepository));

        uc.execute(&RevokeAllSessionsInput {
            user_id: "user-1".to_string(),
        })
        .await
        .unwrap();

        // user-2's session should remain active
        let user2_sessions = repo.find_by_user_id("user-2").await.unwrap();
        assert_eq!(user2_sessions.len(), 1);
        assert!(!user2_sessions[0].revoked);
    }
}

// ===========================================================================
// ListUserSessions
// ===========================================================================
mod list_user_sessions {
    use super::*;

    #[tokio::test]
    async fn returns_all_user_sessions() {
        let sessions = vec![
            make_session("s1", "user-1", false),
            make_session("s2", "user-1", true), // revoked sessions also returned
            make_session("s3", "user-2", false), // different user
        ];
        let repo = Arc::new(StubSessionRepository::with_sessions(sessions));
        let uc = ListUserSessionsUseCase::new(repo);

        let result = uc
            .execute(&ListUserSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.sessions.len(), 2);
        assert!(result.sessions.iter().all(|s| s.user_id == "user-1"));
    }

    #[tokio::test]
    async fn empty_for_unknown_user() {
        let repo = Arc::new(StubSessionRepository::new());
        let uc = ListUserSessionsUseCase::new(repo);

        let result = uc
            .execute(&ListUserSessionsInput {
                user_id: "user-unknown".to_string(),
            })
            .await
            .unwrap();

        assert!(result.sessions.is_empty());
    }
}

// ===========================================================================
// Cross-cutting: full session lifecycle
// ===========================================================================
mod lifecycle {
    use super::*;

    #[tokio::test]
    async fn full_session_lifecycle() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());

        // 1. Create session
        let create_uc = CreateSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher.clone(),
            3600,
            86400,
        );
        let create_result = create_uc
            .execute(&CreateSessionInput {
                user_id: "user-lifecycle".to_string(),
                tenant_id: "tenant-a".to_string(),
                device_id: "device-1".to_string(),
                device_name: Some("Laptop".to_string()),
                device_type: Some("desktop".to_string()),
                user_agent: None,
                ip_address: Some("10.0.0.1".to_string()),
                ttl_seconds: Some(3600),
                max_devices: None,
                metadata: None,
            })
            .await
            .unwrap();
        let session_id = create_result.session.id.clone();
        let token = create_result.session.token.clone();
        assert!(create_result.session.is_valid());

        // 2. Get session by id
        let get_uc = GetSessionUseCase::new(repo.clone());
        let get_result = get_uc
            .execute(&GetSessionInput {
                id: Some(session_id.clone()),
                token: None,
            })
            .await
            .unwrap();
        assert_eq!(get_result.session.user_id, "user-lifecycle");

        // 3. Get session by token
        let get_result = get_uc
            .execute(&GetSessionInput {
                id: None,
                token: Some(token),
            })
            .await
            .unwrap();
        assert_eq!(get_result.session.id, session_id);

        // 4. List sessions
        let list_uc = ListUserSessionsUseCase::new(repo.clone());
        let list_result = list_uc
            .execute(&ListUserSessionsInput {
                user_id: "user-lifecycle".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(list_result.sessions.len(), 1);

        // 5. Refresh session
        let refresh_uc = RefreshSessionUseCase::new(repo.clone(), 86400);
        let refresh_result = refresh_uc
            .execute(&RefreshSessionInput {
                id: session_id.clone(),
                ttl_seconds: 7200,
            })
            .await
            .unwrap();
        assert!(refresh_result.session.is_valid());

        // 6. Revoke session
        let revoke_uc = RevokeSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher.clone(),
        );
        revoke_uc
            .execute(&RevokeSessionInput {
                id: session_id.clone(),
                jwt_jti: None,
                jwt_remaining_secs: None,
            })
            .await
            .unwrap();

        // 7. Verify revoked
        let get_result = get_uc
            .execute(&GetSessionInput {
                id: Some(session_id.clone()),
                token: None,
            })
            .await
            .unwrap();
        assert!(get_result.session.revoked);
        assert!(!get_result.session.is_valid());

        // 8. Cannot revoke again
        let revoke_result = revoke_uc
            .execute(&RevokeSessionInput {
                id: session_id.clone(),
                jwt_jti: None,
                jwt_remaining_secs: None,
            })
            .await;
        assert!(matches!(
            revoke_result,
            Err(SessionError::AlreadyRevoked(_))
        ));

        // 9. Cannot refresh revoked session
        let refresh_result = refresh_uc
            .execute(&RefreshSessionInput {
                id: session_id,
                ttl_seconds: 3600,
            })
            .await;
        assert!(matches!(refresh_result, Err(SessionError::Revoked(_))));

        // Verify events
        let events = publisher.events.read().await;
        assert_eq!(events.len(), 2); // created + revoked
    }

    #[tokio::test]
    async fn revoke_all_then_create_new() {
        let repo = Arc::new(StubSessionRepository::new());
        let publisher = Arc::new(StubSessionEventPublisher::new());

        // Create 3 sessions
        let create_uc = CreateSessionUseCase::new(
            repo.clone(),
            Arc::new(NoopSessionMetadataRepository),
            publisher.clone(),
            3600,
            86400,
        );
        for i in 0..3 {
            create_uc
                .execute(&CreateSessionInput {
                    user_id: "user-bulk".to_string(),
                    tenant_id: "tenant-a".to_string(),
                    device_id: format!("device-{}", i),
                    device_name: None,
                    device_type: None,
                    user_agent: None,
                    ip_address: None,
                    ttl_seconds: Some(3600),
                    max_devices: None,
                    metadata: None,
                })
                .await
                .unwrap();
        }

        // Revoke all
        let revoke_all_uc =
            RevokeAllSessionsUseCase::new(repo.clone(), Arc::new(NoopSessionMetadataRepository));
        let result = revoke_all_uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-bulk".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 3);

        // Create a new session after revoking all
        let new_result = create_uc
            .execute(&CreateSessionInput {
                user_id: "user-bulk".to_string(),
                tenant_id: "tenant-a".to_string(),
                device_id: "device-new".to_string(),
                device_name: None,
                device_type: None,
                user_agent: None,
                ip_address: None,
                ttl_seconds: Some(3600),
                max_devices: None,
                metadata: None,
            })
            .await
            .unwrap();
        assert!(new_result.session.is_valid());

        // Should have 4 sessions total, 3 revoked + 1 active
        let list_uc = ListUserSessionsUseCase::new(repo.clone());
        let all = list_uc
            .execute(&ListUserSessionsInput {
                user_id: "user-bulk".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(all.sessions.len(), 4);
        let active: Vec<_> = all.sessions.iter().filter(|s| !s.revoked).collect();
        assert_eq!(active.len(), 1);
    }
}
