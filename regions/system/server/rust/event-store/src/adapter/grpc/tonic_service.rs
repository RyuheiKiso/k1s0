use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
use crate::proto::k1s0::system::eventstore::v1::{
    event_store_service_server::EventStoreService, AppendEventsRequest as ProtoAppendEventsRequest,
    AppendEventsResponse as ProtoAppendEventsResponse, CreateSnapshotRequest as ProtoCreateSnapshotRequest,
    CreateSnapshotResponse as ProtoCreateSnapshotResponse, DeleteStreamRequest as ProtoDeleteStreamRequest,
    DeleteStreamResponse as ProtoDeleteStreamResponse, EventData as ProtoEventData,
    EventMetadata as ProtoEventMetadata, GetLatestSnapshotRequest as ProtoGetLatestSnapshotRequest,
    GetLatestSnapshotResponse as ProtoGetLatestSnapshotResponse, ListStreamsRequest as ProtoListStreamsRequest,
    ListStreamsResponse as ProtoListStreamsResponse, ReadEventBySequenceRequest as ProtoReadEventBySequenceRequest,
    ReadEventBySequenceResponse as ProtoReadEventBySequenceResponse, ReadEventsRequest as ProtoReadEventsRequest,
    ReadEventsResponse as ProtoReadEventsResponse, Snapshot as ProtoSnapshot, StreamInfo as ProtoStreamInfo,
    StoredEvent as ProtoStoredEvent,
};

use super::event_store_grpc::{
    AppendEventsRequest, CreateSnapshotRequest, DeleteStreamRequest, EventStoreGrpcService,
    GetLatestSnapshotRequest, GrpcError, ListStreamsRequest, PbEventData, PbEventMetadata,
    PbStoredEvent, ReadEventBySequenceRequest, ReadEventsRequest,
};

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

fn pb_stored_event_to_proto(e: PbStoredEvent) -> ProtoStoredEvent {
    ProtoStoredEvent {
        stream_id: e.stream_id,
        sequence: e.sequence,
        event_type: e.event_type,
        version: e.version,
        payload_json: e.payload_json,
        metadata: Some(ProtoEventMetadata {
            actor_id: e.metadata.actor_id,
            correlation_id: e.metadata.correlation_id,
            causation_id: e.metadata.causation_id,
        }),
        occurred_at: e.occurred_at,
        stored_at: e.stored_at,
    }
}

fn proto_event_data_to_pb(e: ProtoEventData) -> PbEventData {
    let meta = e.metadata.unwrap_or_default();
    PbEventData {
        event_type: e.event_type,
        payload_json: e.payload_json,
        metadata: PbEventMetadata {
            actor_id: meta.actor_id,
            correlation_id: meta.correlation_id,
            causation_id: meta.causation_id,
        },
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
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map(|p| (p.page, p.page_size)).unwrap_or((1, 20));
        let resp = self
            .inner
            .list_streams(ListStreamsRequest { page, page_size })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListStreamsResponse {
            streams: resp
                .streams
                .into_iter()
                .map(|s| ProtoStreamInfo {
                    id: s.id,
                    aggregate_type: s.aggregate_type,
                    current_version: s.current_version,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                })
                .collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count.min(i32::MAX as u64) as i32,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn append_events(
        &self,
        request: Request<ProtoAppendEventsRequest>,
    ) -> Result<Response<ProtoAppendEventsResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .append_events(AppendEventsRequest {
                stream_id: inner.stream_id,
                events: inner.events.into_iter().map(proto_event_data_to_pb).collect(),
                expected_version: inner.expected_version,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoAppendEventsResponse {
            stream_id: resp.stream_id,
            events: resp.events.into_iter().map(pb_stored_event_to_proto).collect(),
            current_version: resp.current_version,
        }))
    }

    async fn read_events(
        &self,
        request: Request<ProtoReadEventsRequest>,
    ) -> Result<Response<ProtoReadEventsResponse>, Status> {
        let inner = request.into_inner();
        let req = ReadEventsRequest {
            stream_id: inner.stream_id,
            from_version: inner.from_version,
            to_version: inner.to_version,
            page: inner.page,
            page_size: inner.page_size,
        };
        let page = req.page as i32;
        let page_size = req.page_size as i32;
        let resp = self.inner.read_events(req).await.map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoReadEventsResponse {
            stream_id: resp.stream_id,
            events: resp.events.into_iter().map(pb_stored_event_to_proto).collect(),
            current_version: resp.current_version,
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count.min(i32::MAX as u64) as i32,
                page,
                page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn read_event_by_sequence(
        &self,
        request: Request<ProtoReadEventBySequenceRequest>,
    ) -> Result<Response<ProtoReadEventBySequenceResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .read_event_by_sequence(ReadEventBySequenceRequest {
                stream_id: inner.stream_id,
                sequence: inner.sequence,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoReadEventBySequenceResponse {
            event: Some(pb_stored_event_to_proto(resp.event)),
        }))
    }

    async fn create_snapshot(
        &self,
        request: Request<ProtoCreateSnapshotRequest>,
    ) -> Result<Response<ProtoCreateSnapshotResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .create_snapshot(CreateSnapshotRequest {
                stream_id: inner.stream_id,
                snapshot_version: inner.snapshot_version,
                aggregate_type: inner.aggregate_type,
                state_json: inner.state_json,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateSnapshotResponse {
            id: resp.id,
            stream_id: resp.stream_id,
            snapshot_version: resp.snapshot_version,
            created_at: resp.created_at,
        }))
    }

    async fn get_latest_snapshot(
        &self,
        request: Request<ProtoGetLatestSnapshotRequest>,
    ) -> Result<Response<ProtoGetLatestSnapshotResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_latest_snapshot(GetLatestSnapshotRequest {
                stream_id: inner.stream_id,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetLatestSnapshotResponse {
            snapshot: Some(ProtoSnapshot {
                id: resp.snapshot.id,
                stream_id: resp.snapshot.stream_id,
                snapshot_version: resp.snapshot.snapshot_version,
                aggregate_type: resp.snapshot.aggregate_type,
                state_json: resp.snapshot.state_json,
                created_at: resp.snapshot.created_at,
            }),
        }))
    }

    async fn delete_stream(
        &self,
        request: Request<ProtoDeleteStreamRequest>,
    ) -> Result<Response<ProtoDeleteStreamResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_stream(DeleteStreamRequest {
                stream_id: inner.stream_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteStreamResponse {
            success: resp.success,
            message: resp.message,
        }))
    }
}
