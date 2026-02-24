use std::sync::Arc;

use crate::usecase::append_events::{
    AppendEventsError, AppendEventsInput, AppendEventsUseCase,
};
use crate::usecase::create_snapshot::{
    CreateSnapshotError, CreateSnapshotInput, CreateSnapshotUseCase,
};
use crate::usecase::get_latest_snapshot::{
    GetLatestSnapshotError, GetLatestSnapshotInput, GetLatestSnapshotUseCase,
};
use crate::usecase::read_event_by_sequence::{
    ReadEventBySequenceError, ReadEventBySequenceInput, ReadEventBySequenceUseCase,
};
use crate::usecase::read_events::{ReadEventsError, ReadEventsInput, ReadEventsUseCase};

use crate::domain::entity::event::{EventData, EventMetadata};

// --- gRPC Request/Response Types ---

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
pub struct PbSnapshot {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state_json: Vec<u8>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct GetLatestSnapshotResponse {
    pub snapshot: PbSnapshot,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- EventStoreGrpcService ---

pub struct EventStoreGrpcService {
    append_events_uc: Arc<AppendEventsUseCase>,
    read_events_uc: Arc<ReadEventsUseCase>,
    read_event_by_sequence_uc: Arc<ReadEventBySequenceUseCase>,
    create_snapshot_uc: Arc<CreateSnapshotUseCase>,
    get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
}

impl EventStoreGrpcService {
    pub fn new(
        append_events_uc: Arc<AppendEventsUseCase>,
        read_events_uc: Arc<ReadEventsUseCase>,
        read_event_by_sequence_uc: Arc<ReadEventBySequenceUseCase>,
        create_snapshot_uc: Arc<CreateSnapshotUseCase>,
        get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
    ) -> Self {
        Self {
            append_events_uc,
            read_events_uc,
            read_event_by_sequence_uc,
            create_snapshot_uc,
            get_latest_snapshot_uc,
        }
    }

    pub async fn append_events(
        &self,
        req: AppendEventsRequest,
    ) -> Result<AppendEventsResponse, GrpcError> {
        let events: Vec<EventData> = req
            .events
            .into_iter()
            .map(|e| {
                let payload = serde_json::from_slice(&e.payload_json)
                    .unwrap_or(serde_json::Value::Null);
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
            Ok(output) => {
                let pb_events = output
                    .events
                    .into_iter()
                    .map(stored_event_to_pb)
                    .collect();
                Ok(AppendEventsResponse {
                    stream_id: output.stream_id,
                    events: pb_events,
                    current_version: output.current_version,
                })
            }
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
            }) => Err(GrpcError::InvalidArgument(format!(
                "version conflict for stream {}: expected {}, actual {}",
                stream_id, expected, actual
            ))),
            Err(AppendEventsError::Validation(msg)) => {
                Err(GrpcError::InvalidArgument(msg))
            }
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
            Ok(output) => {
                let pb_events = output
                    .events
                    .into_iter()
                    .map(stored_event_to_pb)
                    .collect();
                Ok(ReadEventsResponse {
                    stream_id: output.stream_id,
                    events: pb_events,
                    current_version: output.current_version,
                    total_count: output.pagination.total_count,
                    has_next: output.pagination.has_next,
                })
            }
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
        let state = serde_json::from_slice(&req.state_json)
            .unwrap_or(serde_json::Value::Null);

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
            Err(CreateSnapshotError::Validation(msg)) => {
                Err(GrpcError::InvalidArgument(msg))
            }
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
            Ok(snap) => {
                let state_json = serde_json::to_vec(&snap.state)
                    .unwrap_or_default();
                Ok(GetLatestSnapshotResponse {
                    snapshot: PbSnapshot {
                        id: snap.id,
                        stream_id: snap.stream_id,
                        snapshot_version: snap.snapshot_version,
                        aggregate_type: snap.aggregate_type,
                        state_json,
                        created_at: snap.created_at.to_rfc3339(),
                    },
                })
            }
            Err(GetLatestSnapshotError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(GetLatestSnapshotError::SnapshotNotFound(id)) => {
                Err(GrpcError::NotFound(format!("snapshot not found for stream: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

// --- Conversion helpers ---

fn stored_event_to_pb(e: crate::domain::entity::event::StoredEvent) -> PbStoredEvent {
    let payload_json = serde_json::to_vec(&e.payload).unwrap_or_default();
    PbStoredEvent {
        stream_id: e.stream_id,
        sequence: e.sequence,
        event_type: e.event_type,
        version: e.version,
        payload_json,
        metadata: PbEventMetadata {
            actor_id: e.metadata.actor_id,
            correlation_id: e.metadata.correlation_id,
            causation_id: e.metadata.causation_id,
        },
        occurred_at: e.occurred_at.to_rfc3339(),
        stored_at: e.stored_at.to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::{EventMetadata as DomainMeta, EventStream, StoredEvent};
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository, MockSnapshotRepository,
    };

    fn make_service(
        stream_repo: MockEventStreamRepository,
        event_repo: MockEventRepository,
        snapshot_repo: MockSnapshotRepository,
    ) -> EventStoreGrpcService {
        let stream = Arc::new(stream_repo);
        let event = Arc::new(event_repo);
        let snap = Arc::new(snapshot_repo);
        EventStoreGrpcService::new(
            Arc::new(AppendEventsUseCase::new(stream.clone(), event.clone())),
            Arc::new(ReadEventsUseCase::new(stream.clone(), event.clone())),
            Arc::new(ReadEventBySequenceUseCase::new(stream.clone(), event.clone())),
            Arc::new(CreateSnapshotUseCase::new(stream.clone(), snap.clone())),
            Arc::new(GetLatestSnapshotUseCase::new(stream, snap)),
        )
    }

    fn make_stored_event(stream_id: &str, seq: u64) -> StoredEvent {
        StoredEvent::new(
            stream_id.to_string(),
            seq,
            "OrderPlaced".to_string(),
            seq as i64,
            serde_json::json!({}),
            DomainMeta::new(None, None, None),
        )
    }

    #[tokio::test]
    async fn test_append_events_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| Ok(None));
        stream_repo.expect_create().returning(|_| Ok(()));
        stream_repo.expect_update_version().returning(|_, _| Ok(()));
        event_repo.expect_append().returning(|sid, events| {
            Ok(events
                .into_iter()
                .enumerate()
                .map(|(i, mut e)| {
                    e.sequence = (i as u64) + 1;
                    e.stream_id = sid.to_string();
                    e
                })
                .collect())
        });

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = AppendEventsRequest {
            stream_id: "order-001".to_string(),
            events: vec![PbEventData {
                event_type: "OrderPlaced".to_string(),
                payload_json: b"{}".to_vec(),
                metadata: PbEventMetadata {
                    actor_id: Some("user-1".to_string()),
                    correlation_id: None,
                    causation_id: None,
                },
            }],
            expected_version: -1,
        };
        let resp = svc.append_events(req).await.unwrap();
        assert_eq!(resp.stream_id, "order-001");
        assert_eq!(resp.current_version, 1);
        assert_eq!(resp.events.len(), 1);
    }

    #[tokio::test]
    async fn test_append_events_version_conflict() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = AppendEventsRequest {
            stream_id: "order-001".to_string(),
            events: vec![PbEventData {
                event_type: "OrderPlaced".to_string(),
                payload_json: b"{}".to_vec(),
                metadata: PbEventMetadata {
                    actor_id: None,
                    correlation_id: None,
                    causation_id: None,
                },
            }],
            expected_version: 2,
        };
        let result = svc.append_events(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => assert!(msg.contains("version conflict")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_read_events_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 3,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        event_repo
            .expect_find_by_stream()
            .returning(|sid, _, _, _, _, _| {
                Ok((vec![make_stored_event(sid, 1), make_stored_event(sid, 2)], 2))
            });

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = ReadEventsRequest {
            stream_id: "order-001".to_string(),
            from_version: 1,
            to_version: None,
            page: 1,
            page_size: 50,
        };
        let resp = svc.read_events(req).await.unwrap();
        assert_eq!(resp.events.len(), 2);
        assert_eq!(resp.current_version, 3);
        assert_eq!(resp.total_count, 2);
        assert!(!resp.has_next);
    }

    #[tokio::test]
    async fn test_read_event_by_sequence_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 3,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        event_repo
            .expect_find_by_sequence()
            .returning(|sid, seq| Ok(Some(make_stored_event(sid, seq))));

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = ReadEventBySequenceRequest {
            stream_id: "order-001".to_string(),
            sequence: 1,
        };
        let resp = svc.read_event_by_sequence(req).await.unwrap();
        assert_eq!(resp.event.event_type, "OrderPlaced");
    }

    #[tokio::test]
    async fn test_create_snapshot_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        snapshot_repo.expect_create().returning(|_| Ok(()));

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = CreateSnapshotRequest {
            stream_id: "order-001".to_string(),
            snapshot_version: 3,
            aggregate_type: "Order".to_string(),
            state_json: b"{\"status\":\"shipped\"}".to_vec(),
        };
        let resp = svc.create_snapshot(req).await.unwrap();
        assert_eq!(resp.stream_id, "order-001");
        assert_eq!(resp.snapshot_version, 3);
        assert!(resp.id.starts_with("snap_"));
    }

    #[tokio::test]
    async fn test_get_latest_snapshot_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        snapshot_repo.expect_find_latest().returning(|_| {
            Ok(Some(crate::domain::entity::event::Snapshot::new(
                "snap_001".to_string(),
                "order-001".to_string(),
                3,
                "Order".to_string(),
                serde_json::json!({"status": "shipped"}),
            )))
        });

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = GetLatestSnapshotRequest {
            stream_id: "order-001".to_string(),
        };
        let resp = svc.get_latest_snapshot(req).await.unwrap();
        assert_eq!(resp.snapshot.id, "snap_001");
        assert_eq!(resp.snapshot.snapshot_version, 3);
    }

    #[tokio::test]
    async fn test_get_latest_snapshot_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "order-001".to_string(),
                aggregate_type: "Order".to_string(),
                current_version: 5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        snapshot_repo.expect_find_latest().returning(|_| Ok(None));

        let svc = make_service(stream_repo, event_repo, snapshot_repo);
        let req = GetLatestSnapshotRequest {
            stream_id: "order-001".to_string(),
        };
        let result = svc.get_latest_snapshot(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("snapshot not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
