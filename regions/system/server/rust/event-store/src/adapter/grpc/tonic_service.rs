use std::sync::Arc;

use k1s0_auth::JwksVerifier;
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

    async fn authorize<T>(&self, request: &Request<T>, action: &str) -> Result<(), Status> {
        let Some(auth_state) = &self.auth_state else {
            return Ok(());
        };

        let auth_header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("missing Authorization metadata"))?;
        let auth_header = auth_header
            .to_str()
            .map_err(|_| Status::unauthenticated("invalid Authorization metadata"))?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| Status::unauthenticated("Authorization must be Bearer token"))?;

        let claims = auth_state
            .verifier
            .verify_token(token)
            .await
            .map_err(|_| Status::unauthenticated("token validation failed"))?;

        if check_system_permission(claims.realm_roles(), action) {
            Ok(())
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
        self.authorize(&request, "read").await?;
        let resp = self
            .inner
            .list_streams(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn append_events(
        &self,
        request: Request<ProtoAppendEventsRequest>,
    ) -> Result<Response<ProtoAppendEventsResponse>, Status> {
        self.authorize(&request, "write").await?;
        let resp = self
            .inner
            .append_events(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn read_events(
        &self,
        request: Request<ProtoReadEventsRequest>,
    ) -> Result<Response<ProtoReadEventsResponse>, Status> {
        self.authorize(&request, "read").await?;
        let resp = self
            .inner
            .read_events(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn read_event_by_sequence(
        &self,
        request: Request<ProtoReadEventBySequenceRequest>,
    ) -> Result<Response<ProtoReadEventBySequenceResponse>, Status> {
        self.authorize(&request, "read").await?;
        let resp = self
            .inner
            .read_event_by_sequence(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn create_snapshot(
        &self,
        request: Request<ProtoCreateSnapshotRequest>,
    ) -> Result<Response<ProtoCreateSnapshotResponse>, Status> {
        self.authorize(&request, "write").await?;
        let resp = self
            .inner
            .create_snapshot(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn get_latest_snapshot(
        &self,
        request: Request<ProtoGetLatestSnapshotRequest>,
    ) -> Result<Response<ProtoGetLatestSnapshotResponse>, Status> {
        self.authorize(&request, "read").await?;
        let resp = self
            .inner
            .get_latest_snapshot(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn delete_stream(
        &self,
        request: Request<ProtoDeleteStreamRequest>,
    ) -> Result<Response<ProtoDeleteStreamResponse>, Status> {
        self.authorize(&request, "admin").await?;
        let resp = self
            .inner
            .delete_stream(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }
}
