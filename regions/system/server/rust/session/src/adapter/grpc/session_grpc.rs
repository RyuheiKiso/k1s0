use std::sync::Arc;

use crate::error::SessionError;
use crate::usecase::create_session::{CreateSessionInput, CreateSessionUseCase};
use crate::usecase::get_session::{GetSessionInput, GetSessionUseCase};
use crate::usecase::list_user_sessions::{ListUserSessionsInput, ListUserSessionsUseCase};
use crate::usecase::refresh_session::{RefreshSessionInput, RefreshSessionUseCase};
use crate::usecase::revoke_all_sessions::{RevokeAllSessionsInput, RevokeAllSessionsUseCase};
use crate::usecase::revoke_session::{RevokeSessionInput, RevokeSessionUseCase};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct GetSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct GetSessionResponse {
    pub session: PbSession,
}

#[derive(Debug, Clone)]
pub struct RefreshSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct RefreshSessionResponse {
    pub session_id: String,
    pub expires_at: String,
}

#[derive(Debug, Clone)]
pub struct RevokeSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct RevokeSessionResponse {
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct RevokeAllSessionsRequest {
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct RevokeAllSessionsResponse {
    pub revoked_count: u32,
}

#[derive(Debug, Clone)]
pub struct ListUserSessionsRequest {
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct ListUserSessionsResponse {
    pub sessions: Vec<PbSession>,
    pub total_count: u32,
}

#[derive(Debug, Clone)]
pub struct PbSession {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub status: String,
    pub expires_at: String,
    pub created_at: String,
    pub last_accessed_at: Option<String>,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- SessionGrpcService ---

pub struct SessionGrpcService {
    create_uc: Arc<CreateSessionUseCase>,
    get_uc: Arc<GetSessionUseCase>,
    refresh_uc: Arc<RefreshSessionUseCase>,
    revoke_uc: Arc<RevokeSessionUseCase>,
    revoke_all_uc: Arc<RevokeAllSessionsUseCase>,
    list_uc: Arc<ListUserSessionsUseCase>,
    default_ttl: i64,
}

impl SessionGrpcService {
    pub fn new(
        create_uc: Arc<CreateSessionUseCase>,
        get_uc: Arc<GetSessionUseCase>,
        refresh_uc: Arc<RefreshSessionUseCase>,
        revoke_uc: Arc<RevokeSessionUseCase>,
        revoke_all_uc: Arc<RevokeAllSessionsUseCase>,
        list_uc: Arc<ListUserSessionsUseCase>,
        default_ttl: i64,
    ) -> Self {
        Self {
            create_uc,
            get_uc,
            refresh_uc,
            revoke_uc,
            revoke_all_uc,
            list_uc,
            default_ttl,
        }
    }

    pub async fn create_session(
        &self,
        req: CreateSessionRequest,
    ) -> Result<CreateSessionResponse, GrpcError> {
        let mut metadata = std::collections::HashMap::new();
        if let Some(ref device_id) = Some(req.device_id.clone()) {
            if !device_id.is_empty() {
                metadata.insert("device_id".to_string(), device_id.clone());
            }
        }
        if let Some(ref name) = req.device_name {
            metadata.insert("device_name".to_string(), name.clone());
        }
        if let Some(ref dt) = req.device_type {
            metadata.insert("device_type".to_string(), dt.clone());
        }
        if let Some(ref ua) = req.user_agent {
            metadata.insert("user_agent".to_string(), ua.clone());
        }
        if let Some(ref ip) = req.ip_address {
            metadata.insert("ip_address".to_string(), ip.clone());
        }

        let input = CreateSessionInput {
            user_id: req.user_id,
            ttl_seconds: Some(self.default_ttl),
            metadata: Some(metadata),
        };

        match self.create_uc.execute(&input).await {
            Ok(output) => Ok(CreateSessionResponse {
                session_id: output.session.id,
                user_id: output.session.user_id,
                device_id: req.device_id,
                expires_at: output.session.expires_at.to_rfc3339(),
                created_at: output.session.created_at.to_rfc3339(),
            }),
            Err(SessionError::InvalidInput(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_session(
        &self,
        req: GetSessionRequest,
    ) -> Result<GetSessionResponse, GrpcError> {
        let input = GetSessionInput {
            id: Some(req.session_id),
            token: None,
        };

        match self.get_uc.execute(&input).await {
            Ok(output) => {
                let s = output.session;
                let status = if s.revoked {
                    "revoked"
                } else if s.is_expired() {
                    "expired"
                } else {
                    "active"
                };
                Ok(GetSessionResponse {
                    session: PbSession {
                        session_id: s.id,
                        user_id: s.user_id,
                        device_id: s.metadata.get("device_id").cloned().unwrap_or_default(),
                        device_name: s.metadata.get("device_name").cloned(),
                        device_type: s.metadata.get("device_type").cloned(),
                        user_agent: s.metadata.get("user_agent").cloned(),
                        ip_address: s.metadata.get("ip_address").cloned(),
                        status: status.to_string(),
                        expires_at: s.expires_at.to_rfc3339(),
                        created_at: s.created_at.to_rfc3339(),
                        last_accessed_at: None,
                    },
                })
            }
            Err(SessionError::NotFound(msg)) => Err(GrpcError::NotFound(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn refresh_session(
        &self,
        req: RefreshSessionRequest,
    ) -> Result<RefreshSessionResponse, GrpcError> {
        let input = RefreshSessionInput {
            id: req.session_id,
            ttl_seconds: self.default_ttl,
        };

        match self.refresh_uc.execute(&input).await {
            Ok(output) => Ok(RefreshSessionResponse {
                session_id: output.session.id,
                expires_at: output.session.expires_at.to_rfc3339(),
            }),
            Err(SessionError::NotFound(msg)) => Err(GrpcError::NotFound(msg)),
            Err(SessionError::Revoked(msg)) => Err(GrpcError::InvalidArgument(format!("session revoked: {}", msg))),
            Err(SessionError::InvalidInput(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn revoke_session(
        &self,
        req: RevokeSessionRequest,
    ) -> Result<RevokeSessionResponse, GrpcError> {
        let input = RevokeSessionInput {
            id: req.session_id,
        };

        match self.revoke_uc.execute(&input).await {
            Ok(()) => Ok(RevokeSessionResponse { success: true }),
            Err(SessionError::NotFound(msg)) => Err(GrpcError::NotFound(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn revoke_all_sessions(
        &self,
        req: RevokeAllSessionsRequest,
    ) -> Result<RevokeAllSessionsResponse, GrpcError> {
        let input = RevokeAllSessionsInput {
            user_id: req.user_id,
        };

        match self.revoke_all_uc.execute(&input).await {
            Ok(output) => Ok(RevokeAllSessionsResponse {
                revoked_count: output.count,
            }),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn list_user_sessions(
        &self,
        req: ListUserSessionsRequest,
    ) -> Result<ListUserSessionsResponse, GrpcError> {
        let input = ListUserSessionsInput {
            user_id: req.user_id,
        };

        match self.list_uc.execute(&input).await {
            Ok(output) => {
                let total = output.sessions.len() as u32;
                let sessions = output
                    .sessions
                    .into_iter()
                    .map(|s| {
                        let status = if s.revoked {
                            "revoked"
                        } else if s.is_expired() {
                            "expired"
                        } else {
                            "active"
                        };
                        PbSession {
                            session_id: s.id,
                            user_id: s.user_id,
                            device_id: s.metadata.get("device_id").cloned().unwrap_or_default(),
                            device_name: s.metadata.get("device_name").cloned(),
                            device_type: s.metadata.get("device_type").cloned(),
                            user_agent: s.metadata.get("user_agent").cloned(),
                            ip_address: s.metadata.get("ip_address").cloned(),
                            status: status.to_string(),
                            expires_at: s.expires_at.to_rfc3339(),
                            created_at: s.created_at.to_rfc3339(),
                            last_accessed_at: None,
                        }
                    })
                    .collect();

                Ok(ListUserSessionsResponse {
                    sessions,
                    total_count: total,
                })
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::session::Session;
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

    fn make_service(mock: MockSessionRepository) -> SessionGrpcService {
        let repo = Arc::new(mock);
        SessionGrpcService::new(
            Arc::new(CreateSessionUseCase::new(repo.clone(), 3600, 86400)),
            Arc::new(GetSessionUseCase::new(repo.clone())),
            Arc::new(RefreshSessionUseCase::new(repo.clone(), 86400)),
            Arc::new(RevokeSessionUseCase::new(repo.clone())),
            Arc::new(RevokeAllSessionsUseCase::new(repo.clone())),
            Arc::new(ListUserSessionsUseCase::new(repo)),
            3600,
        )
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_save().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = CreateSessionRequest {
            user_id: "user-1".to_string(),
            device_id: "device-1".to_string(),
            device_name: Some("My Phone".to_string()),
            device_type: Some("mobile".to_string()),
            user_agent: None,
            ip_address: Some("127.0.0.1".to_string()),
        };
        let resp = svc.create_session(req).await.unwrap();

        assert_eq!(resp.user_id, "user-1");
        assert!(!resp.session_id.is_empty());
        assert!(!resp.expires_at.is_empty());
    }

    #[tokio::test]
    async fn test_get_session_success() {
        let mut mock = MockSessionRepository::new();
        let session = make_session("sess-1");
        let return_session = session.clone();
        mock.expect_find_by_id()
            .withf(|id| id == "sess-1")
            .returning(move |_| Ok(Some(return_session.clone())));

        let svc = make_service(mock);
        let req = GetSessionRequest {
            session_id: "sess-1".to_string(),
        };
        let resp = svc.get_session(req).await.unwrap();

        assert_eq!(resp.session.session_id, "sess-1");
        assert_eq!(resp.session.user_id, "user-1");
        assert_eq!(resp.session.status, "active");
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let svc = make_service(mock);
        let req = GetSessionRequest {
            session_id: "missing".to_string(),
        };
        let result = svc.get_session(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_refresh_session_success() {
        let mut mock = MockSessionRepository::new();
        let session = make_session("sess-1");
        let return_session = session.clone();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(return_session.clone())));
        mock.expect_save().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = RefreshSessionRequest {
            session_id: "sess-1".to_string(),
        };
        let resp = svc.refresh_session(req).await.unwrap();

        assert_eq!(resp.session_id, "sess-1");
        assert!(!resp.expires_at.is_empty());
    }

    #[tokio::test]
    async fn test_revoke_session_success() {
        let mut mock = MockSessionRepository::new();
        let session = make_session("sess-1");
        let return_session = session.clone();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(return_session.clone())));
        mock.expect_save().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = RevokeSessionRequest {
            session_id: "sess-1".to_string(),
        };
        let resp = svc.revoke_session(req).await.unwrap();

        assert!(resp.success);
    }

    #[tokio::test]
    async fn test_revoke_all_sessions_success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| {
            Ok(vec![make_session("s1"), make_session("s2")])
        });
        mock.expect_save().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = RevokeAllSessionsRequest {
            user_id: "user-1".to_string(),
        };
        let resp = svc.revoke_all_sessions(req).await.unwrap();

        assert_eq!(resp.revoked_count, 2);
    }

    #[tokio::test]
    async fn test_list_user_sessions_success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id()
            .returning(|_| Ok(vec![make_session("s1"), make_session("s2")]));

        let svc = make_service(mock);
        let req = ListUserSessionsRequest {
            user_id: "user-1".to_string(),
        };
        let resp = svc.list_user_sessions(req).await.unwrap();

        assert_eq!(resp.total_count, 2);
        assert_eq!(resp.sessions.len(), 2);
    }
}
