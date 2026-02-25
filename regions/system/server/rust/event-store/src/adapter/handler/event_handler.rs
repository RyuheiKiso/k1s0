use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use super::error::EventStoreError;
use super::AppState;
use crate::domain::entity::event::{EventData, EventMetadata};
use crate::usecase::append_events::{AppendEventsError, AppendEventsInput};
use crate::usecase::create_snapshot::{CreateSnapshotError, CreateSnapshotInput};
use crate::usecase::get_latest_snapshot::{GetLatestSnapshotError, GetLatestSnapshotInput};
use crate::usecase::read_events::{ReadEventsError, ReadEventsInput};

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AppendEventsRequest {
    pub stream_id: String,
    pub expected_version: i64,
    pub events: Vec<EventDataRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EventDataRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub metadata: MetadataRequest,
}

#[derive(Debug, Default, Deserialize, utoipa::ToSchema)]
pub struct MetadataRequest {
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AppendEventsResponse {
    pub stream_id: String,
    pub current_version: i64,
    pub events: Vec<StoredEventResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StoredEventResponse {
    pub stream_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub version: i64,
    pub payload: serde_json::Value,
    pub metadata: MetadataResponse,
    pub occurred_at: String,
    pub stored_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MetadataResponse {
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadEventsQuery {
    #[serde(default = "default_from_version")]
    pub from_version: i64,
    pub to_version: Option<i64>,
    pub event_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_from_version() -> i64 {
    1
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadEventsResponse {
    pub stream_id: String,
    pub events: Vec<StoredEventResponse>,
    pub current_version: i64,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationResponse {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub event_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListEventsResponse {
    pub events: Vec<StoredEventResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Deserialize)]
pub struct ListStreamsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StreamResponse {
    pub id: String,
    pub aggregate_type: String,
    pub current_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListStreamsResponse {
    pub streams: Vec<StreamResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSnapshotRequest {
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state: serde_json::Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnapshotResponse {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state: serde_json::Value,
    pub created_at: String,
}

// --- Helpers ---

fn to_stored_event_response(event: &crate::domain::entity::event::StoredEvent) -> StoredEventResponse {
    StoredEventResponse {
        stream_id: event.stream_id.clone(),
        sequence: event.sequence,
        event_type: event.event_type.clone(),
        version: event.version,
        payload: event.payload.clone(),
        metadata: MetadataResponse {
            actor_id: event.metadata.actor_id.clone(),
            correlation_id: event.metadata.correlation_id.clone(),
            causation_id: event.metadata.causation_id.clone(),
        },
        occurred_at: event.occurred_at.to_rfc3339(),
        stored_at: event.stored_at.to_rfc3339(),
    }
}

fn to_stream_response(stream: &crate::domain::entity::event::EventStream) -> StreamResponse {
    StreamResponse {
        id: stream.id.clone(),
        aggregate_type: stream.aggregate_type.clone(),
        current_version: stream.current_version,
        created_at: stream.created_at.to_rfc3339(),
        updated_at: stream.updated_at.to_rfc3339(),
    }
}

// --- Handlers ---

/// POST /api/v1/events - Append events to a stream
#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = AppendEventsRequest,
    responses(
        (status = 201, description = "Events appended", body = AppendEventsResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Stream not found"),
        (status = 409, description = "Version conflict"),
    ),
)]
pub async fn append_events(
    State(state): State<AppState>,
    Json(req): Json<AppendEventsRequest>,
) -> Result<(axum::http::StatusCode, Json<AppendEventsResponse>), EventStoreError> {
    let events: Vec<EventData> = req
        .events
        .into_iter()
        .map(|e| EventData {
            event_type: e.event_type,
            payload: e.payload,
            metadata: EventMetadata::new(
                e.metadata.actor_id,
                e.metadata.correlation_id,
                e.metadata.causation_id,
            ),
        })
        .collect();

    let input = AppendEventsInput {
        stream_id: req.stream_id,
        events,
        expected_version: req.expected_version,
    };

    let output = state.append_events_uc.execute(&input).await.map_err(|e| match e {
        AppendEventsError::StreamNotFound(id) => EventStoreError::NotFound(format!("stream not found: {}", id)),
        AppendEventsError::StreamAlreadyExists(id) => {
            EventStoreError::Conflict(format!("stream already exists: {}", id))
        }
        AppendEventsError::VersionConflict {
            stream_id,
            expected,
            actual,
        } => EventStoreError::Conflict(format!(
            "version conflict for stream {}: expected {}, actual {}",
            stream_id, expected, actual
        )),
        AppendEventsError::Validation(msg) => EventStoreError::Validation(msg),
        AppendEventsError::Internal(msg) => EventStoreError::Internal(msg),
    })?;

    // Publish events to Kafka (best-effort)
    if let Err(e) = state
        .event_publisher
        .publish_events(&output.stream_id, &output.events)
        .await
    {
        tracing::warn!(error = %e, "failed to publish events to kafka");
    }

    let event_responses: Vec<StoredEventResponse> =
        output.events.iter().map(to_stored_event_response).collect();

    Ok((
        axum::http::StatusCode::CREATED,
        Json(AppendEventsResponse {
            stream_id: output.stream_id,
            current_version: output.current_version,
            events: event_responses,
        }),
    ))
}

/// GET /api/v1/events/:stream_id - Read events from a stream
#[utoipa::path(
    get,
    path = "/api/v1/events/{stream_id}",
    params(
        ("stream_id" = String, Path, description = "Stream ID"),
        ("from_version" = Option<i64>, Query, description = "Start version"),
        ("to_version" = Option<i64>, Query, description = "End version"),
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Events found", body = ReadEventsResponse),
        (status = 404, description = "Stream not found"),
    ),
)]
pub async fn read_events(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
    Query(query): Query<ReadEventsQuery>,
) -> Result<Json<ReadEventsResponse>, EventStoreError> {
    let input = ReadEventsInput {
        stream_id,
        from_version: query.from_version,
        to_version: query.to_version,
        event_type: query.event_type,
        page: query.page,
        page_size: query.page_size,
    };

    let output = state.read_events_uc.execute(&input).await.map_err(|e| match e {
        ReadEventsError::StreamNotFound(id) => {
            EventStoreError::NotFound(format!("stream not found: {}", id))
        }
        ReadEventsError::Internal(msg) => EventStoreError::Internal(msg),
    })?;

    let event_responses: Vec<StoredEventResponse> =
        output.events.iter().map(to_stored_event_response).collect();

    Ok(Json(ReadEventsResponse {
        stream_id: output.stream_id,
        events: event_responses,
        current_version: output.current_version,
        pagination: PaginationResponse {
            total_count: output.pagination.total_count,
            page: output.pagination.page,
            page_size: output.pagination.page_size,
            has_next: output.pagination.has_next,
        },
    }))
}

/// GET /api/v1/events - List/query events with pagination
#[utoipa::path(
    get,
    path = "/api/v1/events",
    params(
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Events list", body = ListEventsResponse),
    ),
)]
pub async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<ListEventsResponse>, EventStoreError> {
    let page = query.page.max(1);
    let page_size = query.page_size.max(1).min(200);

    let (events, total_count) = state
        .event_repo
        .find_all(query.event_type, page, page_size)
        .await
        .map_err(|e| EventStoreError::Internal(e.to_string()))?;

    let has_next = (page as u64) * (page_size as u64) < total_count;

    let event_responses: Vec<StoredEventResponse> =
        events.iter().map(to_stored_event_response).collect();

    Ok(Json(ListEventsResponse {
        events: event_responses,
        pagination: PaginationResponse {
            total_count,
            page,
            page_size,
            has_next,
        },
    }))
}

/// GET /api/v1/streams - List streams
#[utoipa::path(
    get,
    path = "/api/v1/streams",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Streams list", body = ListStreamsResponse),
    ),
)]
pub async fn list_streams(
    State(state): State<AppState>,
    Query(query): Query<ListStreamsQuery>,
) -> Result<Json<ListStreamsResponse>, EventStoreError> {
    let page = query.page.max(1);
    let page_size = query.page_size.max(1).min(200);

    let (streams, total_count) = state
        .stream_repo
        .list_all(page, page_size)
        .await
        .map_err(|e| EventStoreError::Internal(e.to_string()))?;

    let has_next = (page as u64) * (page_size as u64) < total_count;

    let stream_responses: Vec<StreamResponse> =
        streams.iter().map(to_stream_response).collect();

    Ok(Json(ListStreamsResponse {
        streams: stream_responses,
        pagination: PaginationResponse {
            total_count,
            page,
            page_size,
            has_next,
        },
    }))
}

/// GET /api/v1/streams/:stream_id/snapshot - Get stream snapshot
#[utoipa::path(
    get,
    path = "/api/v1/streams/{stream_id}/snapshot",
    params(("stream_id" = String, Path, description = "Stream ID")),
    responses(
        (status = 200, description = "Snapshot found", body = SnapshotResponse),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn get_snapshot(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
) -> Result<Json<SnapshotResponse>, EventStoreError> {
    let input = GetLatestSnapshotInput {
        stream_id,
    };

    let snapshot = state.get_latest_snapshot_uc.execute(&input).await.map_err(|e| match e {
        GetLatestSnapshotError::StreamNotFound(id) => {
            EventStoreError::NotFound(format!("stream not found: {}", id))
        }
        GetLatestSnapshotError::SnapshotNotFound(id) => {
            EventStoreError::NotFound(format!("snapshot not found for stream: {}", id))
        }
        GetLatestSnapshotError::Internal(msg) => EventStoreError::Internal(msg),
    })?;

    Ok(Json(SnapshotResponse {
        id: snapshot.id,
        stream_id: snapshot.stream_id,
        snapshot_version: snapshot.snapshot_version,
        aggregate_type: snapshot.aggregate_type,
        state: snapshot.state,
        created_at: snapshot.created_at.to_rfc3339(),
    }))
}

/// POST /api/v1/streams/:stream_id/snapshot - Create snapshot
#[utoipa::path(
    post,
    path = "/api/v1/streams/{stream_id}/snapshot",
    params(("stream_id" = String, Path, description = "Stream ID")),
    request_body = CreateSnapshotRequest,
    responses(
        (status = 201, description = "Snapshot created", body = SnapshotResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Stream not found"),
    ),
)]
pub async fn create_snapshot(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
    Json(req): Json<CreateSnapshotRequest>,
) -> Result<(axum::http::StatusCode, Json<SnapshotResponse>), EventStoreError> {
    let input = CreateSnapshotInput {
        stream_id,
        snapshot_version: req.snapshot_version,
        aggregate_type: req.aggregate_type,
        state: req.state,
    };

    let output = state
        .create_snapshot_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            CreateSnapshotError::StreamNotFound(id) => {
                EventStoreError::NotFound(format!("stream not found: {}", id))
            }
            CreateSnapshotError::Validation(msg) => EventStoreError::Validation(msg),
            CreateSnapshotError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(SnapshotResponse {
            id: output.id,
            stream_id: output.stream_id,
            snapshot_version: output.snapshot_version,
            aggregate_type: output.aggregate_type,
            state: serde_json::Value::Null, // state is not returned from use case output
            created_at: output.created_at.to_rfc3339(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler;
    use crate::domain::entity::event::{
        EventMetadata as DomainMeta, EventStream, Snapshot, StoredEvent,
    };
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository, MockSnapshotRepository,
    };
    use crate::infrastructure::kafka::MockEventPublisher;
    use crate::usecase::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn make_test_state(
        stream_repo: MockEventStreamRepository,
        event_repo: MockEventRepository,
        snapshot_repo: MockSnapshotRepository,
    ) -> AppState {
        let stream: Arc<dyn crate::domain::repository::EventStreamRepository> =
            Arc::new(stream_repo);
        let event: Arc<dyn crate::domain::repository::EventRepository> = Arc::new(event_repo);
        let snap: Arc<dyn crate::domain::repository::SnapshotRepository> = Arc::new(snapshot_repo);

        let mut publisher = MockEventPublisher::new();
        publisher.expect_publish_events().returning(|_, _| Ok(()));
        let publisher: Arc<dyn crate::infrastructure::kafka::EventPublisher> =
            Arc::new(publisher);

        AppState {
            append_events_uc: Arc::new(AppendEventsUseCase::new(stream.clone(), event.clone())),
            read_events_uc: Arc::new(ReadEventsUseCase::new(stream.clone(), event.clone())),
            create_snapshot_uc: Arc::new(CreateSnapshotUseCase::new(
                stream.clone(),
                snap.clone(),
            )),
            get_latest_snapshot_uc: Arc::new(GetLatestSnapshotUseCase::new(
                stream.clone(),
                snap.clone(),
            )),
            stream_repo: stream,
            event_repo: event,
            event_publisher: publisher,
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-event-store-server-test")),
        }
    }

    fn make_stream() -> EventStream {
        EventStream {
            id: "order-001".to_string(),
            aggregate_type: "Order".to_string(),
            current_version: 3,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_event(seq: u64) -> StoredEvent {
        StoredEvent::new(
            "order-001".to_string(),
            seq,
            "OrderPlaced".to_string(),
            seq as i64,
            serde_json::json!({}),
            DomainMeta::new(None, None, None),
        )
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_test_state(
            MockEventStreamRepository::new(),
            MockEventRepository::new(),
            MockSnapshotRepository::new(),
        );
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readyz() {
        let state = make_test_state(
            MockEventStreamRepository::new(),
            MockEventRepository::new(),
            MockSnapshotRepository::new(),
        );
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_append_events_new_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_| Ok(None));
        stream_repo.expect_create().returning(|_| Ok(()));
        stream_repo
            .expect_update_version()
            .returning(|_, _| Ok(()));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));
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
        event_repo
            .expect_find_all()
            .returning(|_, _, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let body = serde_json::json!({
            "stream_id": "order-001",
            "expected_version": -1,
            "events": [{
                "event_type": "OrderPlaced",
                "payload": {"order_id": "o-1"},
                "metadata": {"actor_id": "user-1"}
            }]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_read_events_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));
        event_repo
            .expect_find_by_stream()
            .returning(|_, _, _, _, _, _| Ok((vec![make_event(1), make_event(2)], 2)));
        event_repo
            .expect_find_all()
            .returning(|_, _, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events/order-001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_read_events_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events/order-999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_events() {
        let stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        event_repo
            .expect_find_all()
            .returning(|_, _, _| Ok((vec![make_event(1)], 1)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_streams() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![make_stream()], 1)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/streams")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_snapshot_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));
        snapshot_repo.expect_find_latest().returning(|_| {
            Ok(Some(Snapshot::new(
                "snap_001".to_string(),
                "order-001".to_string(),
                3,
                "Order".to_string(),
                serde_json::json!({"status": "shipped"}),
            )))
        });

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/streams/order-001/snapshot")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_snapshot_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));
        snapshot_repo
            .expect_find_latest()
            .returning(|_| Ok(None));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/streams/order-001/snapshot")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_snapshot_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));
        snapshot_repo.expect_create().returning(|_| Ok(()));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let body = serde_json::json!({
            "snapshot_version": 2,
            "aggregate_type": "Order",
            "state": {"status": "shipped"}
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/streams/order-001/snapshot")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_snapshot_stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));
        stream_repo
            .expect_list_all()
            .returning(|_, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let body = serde_json::json!({
            "snapshot_version": 2,
            "aggregate_type": "Order",
            "state": {"status": "shipped"}
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/streams/order-999/snapshot")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
