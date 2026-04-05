use std::sync::Arc;

use k1s0_auth::{Claims, JwksVerifier};
use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::eventstore::v1::{
    event_store_service_server::EventStoreService, AppendEventsRequest as ProtoAppendEventsRequest,
    AppendEventsResponse as ProtoAppendEventsResponse,
    CreateSnapshotRequest as ProtoCreateSnapshotRequest,
    CreateSnapshotResponse as ProtoCreateSnapshotResponse,
    DeleteStreamRequest as ProtoDeleteStreamRequest,
    DeleteStreamResponse as ProtoDeleteStreamResponse,
    GetLatestSnapshotRequest as ProtoGetLatestSnapshotRequest,
    GetLatestSnapshotResponse as ProtoGetLatestSnapshotResponse,
    ListStreamsRequest as ProtoListStreamsRequest, ListStreamsResponse as ProtoListStreamsResponse,
    ReadEventBySequenceRequest as ProtoReadEventBySequenceRequest,
    ReadEventBySequenceResponse as ProtoReadEventBySequenceResponse,
    ReadEventsRequest as ProtoReadEventsRequest, ReadEventsResponse as ProtoReadEventsResponse,
};

use super::event_store_grpc::{EventStoreGrpcService, GrpcError};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Aborted(msg) => Status::aborted(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

pub struct EventStoreServiceTonic {
    inner: Arc<EventStoreGrpcService>,
    auth_state: Option<EventStoreGrpcAuthState>,
}

#[derive(Clone)]
pub struct EventStoreGrpcAuthState {
    pub verifier: Arc<JwksVerifier>,
}

impl EventStoreServiceTonic {
    pub fn new(
        inner: Arc<EventStoreGrpcService>,
        auth_state: Option<EventStoreGrpcAuthState>,
    ) -> Self {
        Self { inner, auth_state }
    }

    /// リクエストの認証・認可を行い、Claims から tenant_id を取得して返す。
    /// 認証が設定されていない場合は "system" を返す（開発環境用フォールバック）。
    async fn authorize_with_tenant<T>(
        &self,
        request: &Request<T>,
        action: &str,
    ) -> Result<String, Status> {
        let Some(auth_state) = &self.auth_state else {
            // 認証なし（開発環境）: フォールバックのテナント ID を返す
            return Ok("system".to_string());
        };

        let auth_header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("missing Authorization metadata"))?;
        let auth_header = auth_header
            .to_str()
            .map_err(|_| Status::unauthenticated("invalid Authorization metadata"))?;
        // RFC 7235: Authorization スキーム名は大文字小文字を区別しない（RUST-HIGH-001 対応）
        // "Bearer ", "bearer ", "BEARER " いずれも受け入れる
        const BEARER_PREFIX_LEN: usize = 7; // "bearer ".len()
        if auth_header.len() < BEARER_PREFIX_LEN
            || !auth_header[..BEARER_PREFIX_LEN].eq_ignore_ascii_case("bearer ")
        {
            return Err(Status::unauthenticated("Authorization must be Bearer token"));
        }
        let token = &auth_header[BEARER_PREFIX_LEN..];

        let claims: Claims = auth_state
            .verifier
            .verify_token(token)
            .await
            .map_err(|_| Status::unauthenticated("token validation failed"))?;

        if check_system_permission(claims.realm_roles(), action) {
            // テナント ID を Claims から取得して返す
            Ok(claims.tenant_id().to_string())
        } else {
            Err(Status::permission_denied(format!(
                "insufficient permissions for action '{}': required role mapping not satisfied",
                action
            )))
        }
    }
}

fn check_system_permission(roles: &[String], action: &str) -> bool {
    for role in roles {
        match role.as_str() {
            "sys_admin" => return true,
            "sys_operator" => {
                if action == "read" || action == "write" {
                    return true;
                }
            }
            "sys_auditor" => {
                if action == "read" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[async_trait::async_trait]
impl EventStoreService for EventStoreServiceTonic {
    async fn list_streams(
        &self,
        request: Request<ProtoListStreamsRequest>,
    ) -> Result<Response<ProtoListStreamsResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "read").await?;
        let resp = self
            .inner
            .list_streams(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn append_events(
        &self,
        request: Request<ProtoAppendEventsRequest>,
    ) -> Result<Response<ProtoAppendEventsResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "write").await?;
        let resp = self
            .inner
            .append_events(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn read_events(
        &self,
        request: Request<ProtoReadEventsRequest>,
    ) -> Result<Response<ProtoReadEventsResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "read").await?;
        let resp = self
            .inner
            .read_events(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn read_event_by_sequence(
        &self,
        request: Request<ProtoReadEventBySequenceRequest>,
    ) -> Result<Response<ProtoReadEventBySequenceResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "read").await?;
        let resp = self
            .inner
            .read_event_by_sequence(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn create_snapshot(
        &self,
        request: Request<ProtoCreateSnapshotRequest>,
    ) -> Result<Response<ProtoCreateSnapshotResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "write").await?;
        let resp = self
            .inner
            .create_snapshot(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn get_latest_snapshot(
        &self,
        request: Request<ProtoGetLatestSnapshotRequest>,
    ) -> Result<Response<ProtoGetLatestSnapshotResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "read").await?;
        let resp = self
            .inner
            .get_latest_snapshot(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn delete_stream(
        &self,
        request: Request<ProtoDeleteStreamRequest>,
    ) -> Result<Response<ProtoDeleteStreamResponse>, Status> {
        // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
        let tenant_id = self.authorize_with_tenant(&request, "admin").await?;
        let resp = self
            .inner
            .delete_stream(request.into_inner(), &tenant_id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }
}
