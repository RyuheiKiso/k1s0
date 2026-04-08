//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の `SessionService` トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の `SessionGrpcService` に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::Timestamp as ProtoTimestamp;
use crate::proto::k1s0::system::session::v1::{
    session_service_server::SessionService, CreateSessionRequest as ProtoCreateSessionRequest,
    CreateSessionResponse as ProtoCreateSessionResponse,
    GetSessionRequest as ProtoGetSessionRequest, GetSessionResponse as ProtoGetSessionResponse,
    ListUserSessionsRequest as ProtoListUserSessionsRequest,
    ListUserSessionsResponse as ProtoListUserSessionsResponse,
    RefreshSessionRequest as ProtoRefreshSessionRequest,
    RefreshSessionResponse as ProtoRefreshSessionResponse,
    RevokeAllSessionsRequest as ProtoRevokeAllSessionsRequest,
    RevokeAllSessionsResponse as ProtoRevokeAllSessionsResponse,
    RevokeSessionRequest as ProtoRevokeSessionRequest,
    RevokeSessionResponse as ProtoRevokeSessionResponse, Session as ProtoSession,
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
    #[must_use] 
    pub fn new(inner: Arc<SessionGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SessionService for SessionServiceTonic {
    /// gRPC `CreateSession` ハンドラー。
    /// tonic Request Extensions から Claims `を取得し、tenant_id` を `CreateSessionRequest` に設定する。
    /// gRPC 認証レイヤー（GrpcAuthLayer）が Claims を Extensions に注入しているため、
    /// 認証済みリクエストでは必ず Claims が存在する。未認証時は "system" フォールバックを使用する。
    async fn create_session(
        &self,
        request: Request<ProtoCreateSessionRequest>,
    ) -> Result<Response<ProtoCreateSessionResponse>, Status> {
        // gRPC Extensions から Claims を取得して tenant_id を抽出する
        let tenant_id = request
            .extensions()
            .get::<k1s0_auth::Claims>().map_or_else(|| "system".to_string(), |c| c.tenant_id().to_string());

        let inner = request.into_inner();
        let req = CreateSessionRequest {
            user_id: inner.user_id,
            tenant_id,
            device_id: inner.device_id,
            device_name: inner.device_name,
            device_type: inner.device_type,
            user_agent: inner.user_agent,
            ip_address: inner.ip_address,
            ttl_seconds: inner.ttl_seconds,
            max_devices: inner.max_devices.and_then(|v| {
                if v > 0 {
                    u32::try_from(v).ok()
                } else {
                    None
                }
            }),
            metadata: if inner.metadata.is_empty() {
                None
            } else {
                Some(inner.metadata)
            },
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
            expires_at: parse_rfc3339_to_proto_timestamp(&resp.expires_at),
            created_at: parse_rfc3339_to_proto_timestamp(&resp.created_at),
            token: resp.token,
            metadata: resp.metadata,
            device_name: resp.device_name,
            device_type: resp.device_type,
            user_agent: resp.user_agent,
            ip_address: resp.ip_address,
            // ドメイン層の文字列状態を proto enum の i32 値に変換する
            status: status_str_to_proto(&resp.status),
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
        let inner = request.into_inner();
        let req = RefreshSessionRequest {
            session_id: inner.session_id,
            ttl_seconds: inner.ttl_seconds,
        };
        let resp = self
            .inner
            .refresh_session(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRefreshSessionResponse {
            session_id: resp.session_id,
            expires_at: parse_rfc3339_to_proto_timestamp(&resp.expires_at),
            user_id: resp.user_id,
            token: resp.token,
            device_id: resp.device_id,
            device_name: resp.device_name,
            device_type: resp.device_type,
            user_agent: resp.user_agent,
            ip_address: resp.ip_address,
            metadata: resp.metadata,
            created_at: parse_rfc3339_to_proto_timestamp(&resp.created_at),
            last_accessed_at: resp
                .last_accessed_at
                .as_ref()
                .and_then(|v| parse_rfc3339_to_proto_timestamp(v)),
            // ドメイン層の文字列状態を proto enum の i32 値に変換する
            status: status_str_to_proto(&resp.status),
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

/// セッション状態文字列を proto `SessionStatus` enum の i32 値に変換する。
/// ドメイン層の文字列表現（"active", "revoked"）を protobuf の整数 enum 値にマッピングする。
fn status_str_to_proto(s: &str) -> i32 {
    use crate::proto::k1s0::system::session::v1::SessionStatus;
    match s {
        // アクティブ状態: SESSION_STATUS_ACTIVE = 1
        "active" => SessionStatus::Active as i32,
        // 無効化状態: SESSION_STATUS_REVOKED = 2
        "revoked" => SessionStatus::Revoked as i32,
        // 不明な値はデフォルト値（UNSPECIFIED = 0）を返す
        _ => SessionStatus::Unspecified as i32,
    }
}

fn pb_session_to_proto(s: &super::session_grpc::PbSession) -> ProtoSession {
    ProtoSession {
        session_id: s.session_id.clone(),
        user_id: s.user_id.clone(),
        device_id: s.device_id.clone(),
        device_name: s.device_name.clone(),
        device_type: s.device_type.clone(),
        user_agent: s.user_agent.clone(),
        ip_address: s.ip_address.clone(),
        // ドメイン層の文字列状態を proto enum の i32 値に変換する
        status: status_str_to_proto(&s.status),
        token: s.token.clone(),
        expires_at: parse_rfc3339_to_proto_timestamp(&s.expires_at),
        created_at: parse_rfc3339_to_proto_timestamp(&s.created_at),
        last_accessed_at: s
            .last_accessed_at
            .as_ref()
            .and_then(|v| parse_rfc3339_to_proto_timestamp(v)),
    }
}

fn parse_rfc3339_to_proto_timestamp(v: &str) -> Option<ProtoTimestamp> {
    chrono::DateTime::parse_from_rfc3339(v)
        .ok()
        .map(|dt| ProtoTimestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
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

    /// status_str_to_proto 関数のユニットテスト。
    /// ドメイン層の文字列が proto enum の正しい i32 値にマッピングされることを検証する。
    #[test]
    fn test_status_str_to_proto_active() {
        // "active" は SessionStatus::Active (= 1) にマッピングされること
        assert_eq!(status_str_to_proto("active"), 1);
    }

    #[test]
    fn test_status_str_to_proto_revoked() {
        // "revoked" は SessionStatus::Revoked (= 2) にマッピングされること
        assert_eq!(status_str_to_proto("revoked"), 2);
    }

    #[test]
    fn test_status_str_to_proto_unknown() {
        // 未知の値は SessionStatus::Unspecified (= 0) にマッピングされること
        assert_eq!(status_str_to_proto("unknown"), 0);
        assert_eq!(status_str_to_proto(""), 0);
    }
}
