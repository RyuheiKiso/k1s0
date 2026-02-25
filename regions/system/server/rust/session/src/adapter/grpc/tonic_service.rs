//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の SessionService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の SessionGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::session::v1::{
    session_service_server::SessionService,
    CreateSessionRequest as ProtoCreateSessionRequest,
    CreateSessionResponse as ProtoCreateSessionResponse,
    GetSessionRequest as ProtoGetSessionRequest,
    GetSessionResponse as ProtoGetSessionResponse,
    ListUserSessionsRequest as ProtoListUserSessionsRequest,
    ListUserSessionsResponse as ProtoListUserSessionsResponse,
    RefreshSessionRequest as ProtoRefreshSessionRequest,
    RefreshSessionResponse as ProtoRefreshSessionResponse,
    RevokeAllSessionsRequest as ProtoRevokeAllSessionsRequest,
    RevokeAllSessionsResponse as ProtoRevokeAllSessionsResponse,
    RevokeSessionRequest as ProtoRevokeSessionRequest,
    RevokeSessionResponse as ProtoRevokeSessionResponse,
    Session as ProtoSession,
};

use super::session_grpc::{
    CreateSessionRequest, GetSessionRequest, GrpcError, ListUserSessionsRequest,
    RefreshSessionRequest, RevokeAllSessionsRequest, RevokeSessionRequest, SessionGrpcService,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- SessionService tonic ラッパー ---

pub struct SessionServiceTonic {
    inner: Arc<SessionGrpcService>,
}

impl SessionServiceTonic {
    pub fn new(inner: Arc<SessionGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SessionService for SessionServiceTonic {
    async fn create_session(
        &self,
        request: Request<ProtoCreateSessionRequest>,
    ) -> Result<Response<ProtoCreateSessionResponse>, Status> {
        let inner = request.into_inner();
        let req = CreateSessionRequest {
            user_id: inner.user_id,
            device_id: inner.device_id,
            device_name: inner.device_name,
            device_type: inner.device_type,
            user_agent: inner.user_agent,
            ip_address: inner.ip_address,
        };
        let resp = self
            .inner
            .create_session(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateSessionResponse {
            session_id: resp.session_id,
            user_id: resp.user_id,
            device_id: resp.device_id,
            expires_at: resp.expires_at,
            created_at: resp.created_at,
        }))
    }

    async fn get_session(
        &self,
        request: Request<ProtoGetSessionRequest>,
    ) -> Result<Response<ProtoGetSessionResponse>, Status> {
        let req = GetSessionRequest {
            session_id: request.into_inner().session_id,
        };
        let resp = self
            .inner
            .get_session(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetSessionResponse {
            session: Some(pb_session_to_proto(&resp.session)),
        }))
    }

    async fn refresh_session(
        &self,
        request: Request<ProtoRefreshSessionRequest>,
    ) -> Result<Response<ProtoRefreshSessionResponse>, Status> {
        let req = RefreshSessionRequest {
            session_id: request.into_inner().session_id,
        };
        let resp = self
            .inner
            .refresh_session(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRefreshSessionResponse {
            session_id: resp.session_id,
            expires_at: resp.expires_at,
        }))
    }

    async fn revoke_session(
        &self,
        request: Request<ProtoRevokeSessionRequest>,
    ) -> Result<Response<ProtoRevokeSessionResponse>, Status> {
        let req = RevokeSessionRequest {
            session_id: request.into_inner().session_id,
        };
        let resp = self
            .inner
            .revoke_session(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRevokeSessionResponse {
            success: resp.success,
        }))
    }

    async fn revoke_all_sessions(
        &self,
        request: Request<ProtoRevokeAllSessionsRequest>,
    ) -> Result<Response<ProtoRevokeAllSessionsResponse>, Status> {
        let req = RevokeAllSessionsRequest {
            user_id: request.into_inner().user_id,
        };
        let resp = self
            .inner
            .revoke_all_sessions(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRevokeAllSessionsResponse {
            revoked_count: resp.revoked_count,
        }))
    }

    async fn list_user_sessions(
        &self,
        request: Request<ProtoListUserSessionsRequest>,
    ) -> Result<Response<ProtoListUserSessionsResponse>, Status> {
        let req = ListUserSessionsRequest {
            user_id: request.into_inner().user_id,
        };
        let resp = self
            .inner
            .list_user_sessions(req)
            .await
            .map_err(Into::<Status>::into)?;

        let sessions = resp.sessions.iter().map(pb_session_to_proto).collect();

        Ok(Response::new(ProtoListUserSessionsResponse {
            sessions,
            total_count: resp.total_count,
        }))
    }
}

// --- 変換ヘルパー ---

fn pb_session_to_proto(s: &super::session_grpc::PbSession) -> ProtoSession {
    ProtoSession {
        session_id: s.session_id.clone(),
        user_id: s.user_id.clone(),
        device_id: s.device_id.clone(),
        device_name: s.device_name.clone(),
        device_type: s.device_type.clone(),
        user_agent: s.user_agent.clone(),
        ip_address: s.ip_address.clone(),
        status: s.status.clone(),
        expires_at: s.expires_at.clone(),
        created_at: s.created_at.clone(),
        last_accessed_at: s.last_accessed_at.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("session not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("session not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid input".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("internal error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
