use std::sync::Arc;

use crate::domain::entity::event::{EventData, EventMetadata, EventStream};
use crate::domain::repository::EventStreamRepository;
use crate::usecase::append_events::{AppendEventsError, AppendEventsInput, AppendEventsUseCase};
use crate::usecase::create_snapshot::{CreateSnapshotError, CreateSnapshotInput, CreateSnapshotUseCase};
use crate::usecase::delete_stream::{DeleteStreamError, DeleteStreamInput, DeleteStreamUseCase};
use crate::usecase::get_latest_snapshot::{GetLatestSnapshotError, GetLatestSnapshotInput, GetLatestSnapshotUseCase};
use crate::usecase::read_event_by_sequence::{ReadEventBySequenceError, ReadEventBySequenceInput, ReadEventBySequenceUseCase};
use crate::usecase::read_events::{ReadEventsError, ReadEventsInput, ReadEventsUseCase};

#[derive(Debug, Clone)]
pub struct PbEventMetadata {
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PbEventData {
    pub event_type: String,
    pub payload_json: Vec<u8>,
    pub metadata: PbEventMetadata,
}

#[derive(Debug, Clone)]
pub struct PbStoredEvent {
    pub stream_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub version: i64,
    pub payload_json: Vec<u8>,
    pub metadata: PbEventMetadata,
    pub occurred_at: String,
    pub stored_at: String,
}

#[derive(Debug, Clone)]
pub struct PbSnapshot {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state_json: Vec<u8>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct StreamInfoData {
    pub id: String,
    pub aggregate_type: String,
    pub current_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ListStreamsRequest {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListStreamsResponse {
    pub streams: Vec<StreamInfoData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct AppendEventsRequest {
    pub stream_id: String,
    pub events: Vec<PbEventData>,
    pub expected_version: i64,
}

#[derive(Debug, Clone)]
pub struct AppendEventsResponse {
    pub stream_id: String,
    pub events: Vec<PbStoredEvent>,
    pub current_version: i64,
}

#[derive(Debug, Clone)]
pub struct ReadEventsRequest {
    pub stream_id: String,
    pub from_version: i64,
    pub to_version: Option<i64>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ReadEventsResponse {
    pub stream_id: String,
    pub events: Vec<PbStoredEvent>,
    pub current_version: i64,
    pub total_count: u64,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct ReadEventBySequenceRequest {
    pub stream_id: String,
    pub sequence: u64,
}

#[derive(Debug, Clone)]
pub struct ReadEventBySequenceResponse {
    pub event: PbStoredEvent,
}

#[derive(Debug, Clone)]
pub struct CreateSnapshotRequest {
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CreateSnapshotResponse {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct GetLatestSnapshotRequest {
    pub stream_id: String,
}

#[derive(Debug, Clone)]
pub struct GetLatestSnapshotResponse {
    pub snapshot: PbSnapshot,
}

#[derive(Debug, Clone)]
pub struct DeleteStreamRequest {
    pub stream_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteStreamResponse {
    pub success: bool,
    pub message: String,
}

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
        req: ListStreamsRequest,
    ) -> Result<ListStreamsResponse, GrpcError> {
        let page = if req.page <= 0 { 1 } else { req.page as u32 };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as u32
        };

        let (streams, total_count) = self
            .stream_repo
            .list_all(page, page_size)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        let has_next = (page as u64) * (page_size as u64) < total_count;

        Ok(ListStreamsResponse {
            streams: streams.into_iter().map(to_stream_info).collect(),
            total_count,
            page: page as i32,
            page_size: page_size as i32,
            has_next,
        })
    }

    pub async fn append_events(
        &self,
        req: AppendEventsRequest,
    ) -> Result<AppendEventsResponse, GrpcError> {
        let events: Vec<EventData> = req
            .events
            .into_iter()
            .map(|e| {
                let payload = serde_json::from_slice(&e.payload_json).unwrap_or(serde_json::Value::Null);
                EventData {
                    event_type: e.event_type,
                    payload,
                    metadata: EventMetadata::new(
                        e.metadata.actor_id,
                        e.metadata.correlation_id,
                        e.metadata.causation_id,
                    ),
                }
            })
            .collect();

        let input = AppendEventsInput {
            stream_id: req.stream_id,
            events,
            expected_version: req.expected_version,
        };

        match self.append_events_uc.execute(&input).await {
            Ok(output) => Ok(AppendEventsResponse {
                stream_id: output.stream_id,
                events: output.events.into_iter().map(stored_event_to_pb).collect(),
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
        req: ReadEventsRequest,
    ) -> Result<ReadEventsResponse, GrpcError> {
        let input = ReadEventsInput {
            stream_id: req.stream_id,
            from_version: req.from_version,
            to_version: req.to_version,
            event_type: None,
            page: req.page,
            page_size: req.page_size,
        };

        match self.read_events_uc.execute(&input).await {
            Ok(output) => Ok(ReadEventsResponse {
                stream_id: output.stream_id,
                events: output.events.into_iter().map(stored_event_to_pb).collect(),
                current_version: output.current_version,
                total_count: output.pagination.total_count,
                has_next: output.pagination.has_next,
            }),
            Err(ReadEventsError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn read_event_by_sequence(
        &self,
        req: ReadEventBySequenceRequest,
    ) -> Result<ReadEventBySequenceResponse, GrpcError> {
        let input = ReadEventBySequenceInput {
            stream_id: req.stream_id,
            sequence: req.sequence,
        };

        match self.read_event_by_sequence_uc.execute(&input).await {
            Ok(event) => Ok(ReadEventBySequenceResponse {
                event: stored_event_to_pb(event),
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
        req: CreateSnapshotRequest,
    ) -> Result<CreateSnapshotResponse, GrpcError> {
        let state = serde_json::from_slice(&req.state_json).unwrap_or(serde_json::Value::Null);

        let input = CreateSnapshotInput {
            stream_id: req.stream_id,
            snapshot_version: req.snapshot_version,
            aggregate_type: req.aggregate_type,
            state,
        };

        match self.create_snapshot_uc.execute(&input).await {
            Ok(output) => Ok(CreateSnapshotResponse {
                id: output.id,
                stream_id: output.stream_id,
                snapshot_version: output.snapshot_version,
                created_at: output.created_at.to_rfc3339(),
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
        req: GetLatestSnapshotRequest,
    ) -> Result<GetLatestSnapshotResponse, GrpcError> {
        let input = GetLatestSnapshotInput {
            stream_id: req.stream_id,
        };

        match self.get_latest_snapshot_uc.execute(&input).await {
            Ok(snap) => Ok(GetLatestSnapshotResponse {
                snapshot: PbSnapshot {
                    id: snap.id,
                    stream_id: snap.stream_id,
                    snapshot_version: snap.snapshot_version,
                    aggregate_type: snap.aggregate_type,
                    state_json: serde_json::to_vec(&snap.state).unwrap_or_default(),
                    created_at: snap.created_at.to_rfc3339(),
                },
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
        req: DeleteStreamRequest,
    ) -> Result<DeleteStreamResponse, GrpcError> {
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

        Ok(DeleteStreamResponse {
            success: out.success,
            message: out.message,
        })
    }
}

fn stored_event_to_pb(e: crate::domain::entity::event::StoredEvent) -> PbStoredEvent {
    PbStoredEvent {
        stream_id: e.stream_id,
        sequence: e.sequence,
        event_type: e.event_type,
        version: e.version,
        payload_json: serde_json::to_vec(&e.payload).unwrap_or_default(),
        metadata: PbEventMetadata {
            actor_id: e.metadata.actor_id,
            correlation_id: e.metadata.correlation_id,
            causation_id: e.metadata.causation_id,
        },
        occurred_at: e.occurred_at.to_rfc3339(),
        stored_at: e.stored_at.to_rfc3339(),
    }
}

fn to_stream_info(stream: EventStream) -> StreamInfoData {
    StreamInfoData {
        id: stream.id,
        aggregate_type: stream.aggregate_type,
        current_version: stream.current_version,
        created_at: stream.created_at.to_rfc3339(),
        updated_at: stream.updated_at.to_rfc3339(),
    }
}
