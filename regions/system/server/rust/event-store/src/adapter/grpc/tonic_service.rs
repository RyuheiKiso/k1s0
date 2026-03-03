use std::sync::Arc;

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
}

impl EventStoreServiceTonic {
    pub fn new(inner: Arc<EventStoreGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl EventStoreService for EventStoreServiceTonic {
    async fn list_streams(
        &self,
        request: Request<ProtoListStreamsRequest>,
    ) -> Result<Response<ProtoListStreamsResponse>, Status> {
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
        let resp = self
            .inner
            .delete_stream(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }
}

