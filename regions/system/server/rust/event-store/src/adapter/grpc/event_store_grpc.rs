use std::sync::Arc;

use crate::domain::entity::event::{EventData, EventMetadata};
use crate::domain::repository::EventStreamRepository;
use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
use crate::proto::k1s0::system::eventstore::v1::{
    AppendEventsRequest as ProtoAppendEventsRequest, AppendEventsResponse as ProtoAppendEventsResponse,
    CreateSnapshotRequest as ProtoCreateSnapshotRequest,
    CreateSnapshotResponse as ProtoCreateSnapshotResponse,
    DeleteStreamRequest as ProtoDeleteStreamRequest,
    DeleteStreamResponse as ProtoDeleteStreamResponse, EventMetadata as ProtoEventMetadata,
    GetLatestSnapshotRequest as ProtoGetLatestSnapshotRequest,
    GetLatestSnapshotResponse as ProtoGetLatestSnapshotResponse,
    ListStreamsRequest as ProtoListStreamsRequest, ListStreamsResponse as ProtoListStreamsResponse,
    ReadEventBySequenceRequest as ProtoReadEventBySequenceRequest,
    ReadEventBySequenceResponse as ProtoReadEventBySequenceResponse,
    ReadEventsRequest as ProtoReadEventsRequest, ReadEventsResponse as ProtoReadEventsResponse,
    Snapshot as ProtoSnapshot, StreamInfo as ProtoStreamInfo, StoredEvent as ProtoStoredEvent,
};
use crate::usecase::append_events::{AppendEventsError, AppendEventsInput, AppendEventsUseCase};
use crate::usecase::create_snapshot::{CreateSnapshotError, CreateSnapshotInput, CreateSnapshotUseCase};
use crate::usecase::delete_stream::{DeleteStreamError, DeleteStreamInput, DeleteStreamUseCase};
use crate::usecase::get_latest_snapshot::{
    GetLatestSnapshotError, GetLatestSnapshotInput, GetLatestSnapshotUseCase,
};
use crate::usecase::read_event_by_sequence::{
    ReadEventBySequenceError, ReadEventBySequenceInput, ReadEventBySequenceUseCase,
};
use crate::usecase::read_events::{ReadEventsError, ReadEventsInput, ReadEventsUseCase};

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("aborted: {0}")]
    Aborted(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub struct EventStoreGrpcService {
    append_events_uc: Arc<AppendEventsUseCase>,
    read_events_uc: Arc<ReadEventsUseCase>,
    read_event_by_sequence_uc: Arc<ReadEventBySequenceUseCase>,
    create_snapshot_uc: Arc<CreateSnapshotUseCase>,
    get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
    delete_stream_uc: Arc<DeleteStreamUseCase>,
    stream_repo: Arc<dyn EventStreamRepository>,
}

impl EventStoreGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        append_events_uc: Arc<AppendEventsUseCase>,
        read_events_uc: Arc<ReadEventsUseCase>,
        read_event_by_sequence_uc: Arc<ReadEventBySequenceUseCase>,
        create_snapshot_uc: Arc<CreateSnapshotUseCase>,
        get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
        delete_stream_uc: Arc<DeleteStreamUseCase>,
        stream_repo: Arc<dyn EventStreamRepository>,
    ) -> Self {
        Self {
            append_events_uc,
            read_events_uc,
            read_event_by_sequence_uc,
            create_snapshot_uc,
            get_latest_snapshot_uc,
            delete_stream_uc,
            stream_repo,
        }
    }

    pub async fn list_streams(
        &self,
        req: ProtoListStreamsRequest,
    ) -> Result<ProtoListStreamsResponse, GrpcError> {
        let pagination = req.pagination.unwrap_or_default();
        let page = if pagination.page <= 0 {
            1
        } else {
            pagination.page as u32
        };
        let page_size = if pagination.page_size <= 0 {
            50
        } else {
            pagination.page_size as u32
        };

        let (streams, total_count) = self
            .stream_repo
            .list_all(page, page_size)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        let has_next = (page as u64) * (page_size as u64) < total_count;

        Ok(ProtoListStreamsResponse {
            streams: streams
                .into_iter()
                .map(|stream| ProtoStreamInfo {
                    id: stream.id,
                    aggregate_type: stream.aggregate_type,
                    current_version: stream.current_version,
                    created_at: stream.created_at.to_rfc3339(),
                    updated_at: stream.updated_at.to_rfc3339(),
                })
                .collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total_count.min(i32::MAX as u64) as i32,
                page: page as i32,
                page_size: page_size as i32,
                has_next,
            }),
        })
    }

    pub async fn append_events(
        &self,
        req: ProtoAppendEventsRequest,
    ) -> Result<ProtoAppendEventsResponse, GrpcError> {
        let events: Vec<EventData> = req
            .events
            .into_iter()
            .map(|e| {
                let payload = serde_json::from_slice(&e.payload).unwrap_or(serde_json::Value::Null);
                let metadata = e.metadata.unwrap_or_default();
                EventData {
                    event_type: e.event_type,
                    payload,
                    metadata: EventMetadata::new(
                        metadata.actor_id,
                        metadata.correlation_id,
                        metadata.causation_id,
                    ),
                }
            })
            .collect();

        let input = AppendEventsInput {
            stream_id: req.stream_id,
            aggregate_type: None,
            events,
            expected_version: req.expected_version,
        };

        match self.append_events_uc.execute(&input).await {
            Ok(output) => Ok(ProtoAppendEventsResponse {
                stream_id: output.stream_id,
                events: output.events.into_iter().map(stored_event_to_proto).collect(),
                current_version: output.current_version,
            }),
            Err(AppendEventsError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(AppendEventsError::StreamAlreadyExists(id)) => {
                Err(GrpcError::AlreadyExists(format!("stream already exists: {}", id)))
            }
            Err(AppendEventsError::VersionConflict {
                stream_id,
                expected,
                actual,
            }) => Err(GrpcError::Aborted(format!(
                "version conflict for stream {}: expected {}, actual {}",
                stream_id, expected, actual
            ))),
            Err(AppendEventsError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn read_events(
        &self,
        req: ProtoReadEventsRequest,
    ) -> Result<ProtoReadEventsResponse, GrpcError> {
        let input = ReadEventsInput {
            stream_id: req.stream_id,
            from_version: req.from_version,
            to_version: req.to_version,
            event_type: None,
            page: req.page,
            page_size: req.page_size,
        };

        match self.read_events_uc.execute(&input).await {
            Ok(output) => Ok(ProtoReadEventsResponse {
                stream_id: output.stream_id,
                events: output.events.into_iter().map(stored_event_to_proto).collect(),
                current_version: output.current_version,
                pagination: Some(ProtoPaginationResult {
                    total_count: output.pagination.total_count.min(i32::MAX as u64) as i32,
                    page: req.page as i32,
                    page_size: req.page_size as i32,
                    has_next: output.pagination.has_next,
                }),
            }),
            Err(ReadEventsError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn read_event_by_sequence(
        &self,
        req: ProtoReadEventBySequenceRequest,
    ) -> Result<ProtoReadEventBySequenceResponse, GrpcError> {
        let input = ReadEventBySequenceInput {
            stream_id: req.stream_id,
            sequence: req.sequence,
        };

        match self.read_event_by_sequence_uc.execute(&input).await {
            Ok(event) => Ok(ProtoReadEventBySequenceResponse {
                event: Some(stored_event_to_proto(event)),
            }),
            Err(ReadEventBySequenceError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(ReadEventBySequenceError::EventNotFound { stream_id, sequence }) => {
                Err(GrpcError::NotFound(format!(
                    "event not found: stream={}, sequence={}",
                    stream_id, sequence
                )))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn create_snapshot(
        &self,
        req: ProtoCreateSnapshotRequest,
    ) -> Result<ProtoCreateSnapshotResponse, GrpcError> {
        let state = serde_json::from_slice(&req.state).unwrap_or(serde_json::Value::Null);

        let input = CreateSnapshotInput {
            stream_id: req.stream_id,
            snapshot_version: req.snapshot_version,
            aggregate_type: req.aggregate_type,
            state,
        };

        match self.create_snapshot_uc.execute(&input).await {
            Ok(output) => Ok(ProtoCreateSnapshotResponse {
                id: output.id,
                stream_id: output.stream_id,
                snapshot_version: output.snapshot_version,
                created_at: output.created_at.to_rfc3339(),
                aggregate_type: output.aggregate_type,
            }),
            Err(CreateSnapshotError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(CreateSnapshotError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_latest_snapshot(
        &self,
        req: ProtoGetLatestSnapshotRequest,
    ) -> Result<ProtoGetLatestSnapshotResponse, GrpcError> {
        let input = GetLatestSnapshotInput {
            stream_id: req.stream_id,
        };

        match self.get_latest_snapshot_uc.execute(&input).await {
            Ok(snapshot) => Ok(ProtoGetLatestSnapshotResponse {
                snapshot: Some(ProtoSnapshot {
                    id: snapshot.id,
                    stream_id: snapshot.stream_id,
                    snapshot_version: snapshot.snapshot_version,
                    aggregate_type: snapshot.aggregate_type,
                    state: serde_json::to_vec(&snapshot.state).unwrap_or_default(),
                    created_at: snapshot.created_at.to_rfc3339(),
                }),
            }),
            Err(GetLatestSnapshotError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(GetLatestSnapshotError::SnapshotNotFound(id)) => {
                Err(GrpcError::NotFound(format!("snapshot not found for stream: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn delete_stream(
        &self,
        req: ProtoDeleteStreamRequest,
    ) -> Result<ProtoDeleteStreamResponse, GrpcError> {
        let out = self
            .delete_stream_uc
            .execute(&DeleteStreamInput {
                stream_id: req.stream_id,
            })
            .await
            .map_err(|e| match e {
                DeleteStreamError::StreamNotFound(id) => {
                    GrpcError::NotFound(format!("stream not found: {}", id))
                }
                DeleteStreamError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ProtoDeleteStreamResponse {
            success: out.success,
            message: out.message,
        })
    }
}

fn stored_event_to_proto(e: crate::domain::entity::event::StoredEvent) -> ProtoStoredEvent {
    ProtoStoredEvent {
        stream_id: e.stream_id,
        sequence: e.sequence,
        event_type: e.event_type,
        version: e.version,
        payload: serde_json::to_vec(&e.payload).unwrap_or_default(),
        metadata: Some(ProtoEventMetadata {
            actor_id: e.metadata.actor_id,
            correlation_id: e.metadata.correlation_id,
            causation_id: e.metadata.causation_id,
        }),
        occurred_at: e.occurred_at.to_rfc3339(),
        stored_at: e.stored_at.to_rfc3339(),
    }
}
