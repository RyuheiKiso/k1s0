//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の EventStoreService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の EventStoreGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::eventstore::v1::{
    event_store_service_server::EventStoreService,
    AppendEventsRequest as ProtoAppendEventsRequest,
    AppendEventsResponse as ProtoAppendEventsResponse,
    CreateSnapshotRequest as ProtoCreateSnapshotRequest,
    CreateSnapshotResponse as ProtoCreateSnapshotResponse, EventData as ProtoEventData,
    EventMetadata as ProtoEventMetadata,
    GetLatestSnapshotRequest as ProtoGetLatestSnapshotRequest,
    GetLatestSnapshotResponse as ProtoGetLatestSnapshotResponse,
    ReadEventBySequenceRequest as ProtoReadEventBySequenceRequest,
    ReadEventBySequenceResponse as ProtoReadEventBySequenceResponse,
    ReadEventsRequest as ProtoReadEventsRequest, ReadEventsResponse as ProtoReadEventsResponse,
    Snapshot as ProtoSnapshot, StoredEvent as ProtoStoredEvent,
};

use super::event_store_grpc::{
    AppendEventsRequest, CreateSnapshotRequest, EventStoreGrpcService, GetLatestSnapshotRequest,
    GrpcError, PbEventData, PbEventMetadata, PbStoredEvent, ReadEventBySequenceRequest,
    ReadEventsRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- 変換ヘルパー ---

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

// --- EventStoreService tonic ラッパー ---

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
    async fn append_events(
        &self,
        request: Request<ProtoAppendEventsRequest>,
    ) -> Result<Response<ProtoAppendEventsResponse>, Status> {
        let inner = request.into_inner();
        let req = AppendEventsRequest {
            stream_id: inner.stream_id,
            events: inner.events.into_iter().map(proto_event_data_to_pb).collect(),
            expected_version: inner.expected_version,
        };
        let resp = self
            .inner
            .append_events(req)
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
        let resp = self
            .inner
            .read_events(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoReadEventsResponse {
            stream_id: resp.stream_id,
            events: resp.events.into_iter().map(pb_stored_event_to_proto).collect(),
            current_version: resp.current_version,
            total_count: resp.total_count,
            has_next: resp.has_next,
        }))
    }

    async fn read_event_by_sequence(
        &self,
        request: Request<ProtoReadEventBySequenceRequest>,
    ) -> Result<Response<ProtoReadEventBySequenceResponse>, Status> {
        let inner = request.into_inner();
        let req = ReadEventBySequenceRequest {
            stream_id: inner.stream_id,
            sequence: inner.sequence,
        };
        let resp = self
            .inner
            .read_event_by_sequence(req)
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
        let req = CreateSnapshotRequest {
            stream_id: inner.stream_id,
            snapshot_version: inner.snapshot_version,
            aggregate_type: inner.aggregate_type,
            state_json: inner.state_json,
        };
        let resp = self
            .inner
            .create_snapshot(req)
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
        let req = GetLatestSnapshotRequest {
            stream_id: inner.stream_id,
        };
        let resp = self
            .inner
            .get_latest_snapshot(req)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("stream not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("stream not found"));
    }

    #[test]
    fn test_grpc_error_already_exists_to_status() {
        let err = GrpcError::AlreadyExists("stream exists".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::AlreadyExists);
        assert!(status.message().contains("stream exists"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("version conflict".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("version conflict"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }
}
