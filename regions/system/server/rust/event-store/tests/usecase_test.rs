use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use k1s0_event_store_server::domain::entity::event::{
    EventData, EventMetadata, EventStream, Snapshot, StoredEvent,
};
use k1s0_event_store_server::domain::repository::{
    EventRepository, EventStreamRepository, SnapshotRepository,
};
use k1s0_event_store_server::usecase::append_events::{
    AppendEventsError, AppendEventsInput, AppendEventsUseCase,
};
use k1s0_event_store_server::usecase::create_snapshot::{
    CreateSnapshotError, CreateSnapshotInput, CreateSnapshotUseCase,
};
use k1s0_event_store_server::usecase::delete_stream::{
    DeleteStreamError, DeleteStreamInput, DeleteStreamUseCase,
};
use k1s0_event_store_server::usecase::get_latest_snapshot::{
    GetLatestSnapshotError, GetLatestSnapshotInput, GetLatestSnapshotUseCase,
};
use k1s0_event_store_server::usecase::list_events::{
    ListEventsError, ListEventsInput, ListEventsUseCase,
};
use k1s0_event_store_server::usecase::list_streams::{
    ListStreamsError, ListStreamsInput, ListStreamsUseCase,
};
use k1s0_event_store_server::usecase::read_event_by_sequence::{
    ReadEventBySequenceError, ReadEventBySequenceInput, ReadEventBySequenceUseCase,
};
use k1s0_event_store_server::usecase::read_events::{
    ReadEventsError, ReadEventsInput, ReadEventsUseCase,
};

// ============================================================================
// Test Stub: In-Memory EventStreamRepository
// ============================================================================

struct StubEventStreamRepository {
    streams: RwLock<Vec<EventStream>>,
    should_fail: bool,
}

impl StubEventStreamRepository {
    fn new() -> Self {
        Self {
            streams: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            streams: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    fn with_streams(streams: Vec<EventStream>) -> Self {
        Self {
            streams: RwLock::new(streams),
            should_fail: false,
        }
    }
}

#[async_trait]
impl EventStreamRepository for StubEventStreamRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<EventStream>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let streams = self.streams.read().await;
        Ok(streams.iter().find(|s| s.id == id).cloned())
    }

    async fn list_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<EventStream>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let streams = self.streams.read().await;
        let total = streams.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let result: Vec<EventStream> = streams
            .iter()
            .skip(start)
            .take(page_size as usize)
            .cloned()
            .collect();
        Ok((result, total))
    }

    async fn create(&self, stream: &EventStream) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut streams = self.streams.write().await;
        streams.push(stream.clone());
        Ok(())
    }

    async fn update_version(&self, id: &str, new_version: i64) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.iter_mut().find(|s| s.id == id) {
            stream.current_version = new_version;
            stream.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("stream not found: {}", id))
        }
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut streams = self.streams.write().await;
        let before = streams.len();
        streams.retain(|s| s.id != id);
        Ok(streams.len() < before)
    }
}

// ============================================================================
// Test Stub: In-Memory EventRepository
// ============================================================================

struct StubEventRepository {
    events: RwLock<Vec<StoredEvent>>,
    should_fail: bool,
    next_sequence: RwLock<u64>,
}

impl StubEventRepository {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            should_fail: false,
            next_sequence: RwLock::new(1),
        }
    }

    fn with_error() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            should_fail: true,
            next_sequence: RwLock::new(1),
        }
    }

    fn with_events(events: Vec<StoredEvent>) -> Self {
        let next_seq = events.iter().map(|e| e.sequence).max().unwrap_or(0) + 1;
        Self {
            events: RwLock::new(events),
            should_fail: false,
            next_sequence: RwLock::new(next_seq),
        }
    }
}

#[async_trait]
impl EventRepository for StubEventRepository {
    async fn append(
        &self,
        stream_id: &str,
        events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut store = self.events.write().await;
        let mut seq = self.next_sequence.write().await;
        let mut result = Vec::new();
        for mut event in events {
            event.stream_id = stream_id.to_string();
            event.sequence = *seq;
            *seq += 1;
            result.push(event.clone());
            store.push(event);
        }
        Ok(result)
    }

    async fn find_by_stream(
        &self,
        stream_id: &str,
        from_version: i64,
        to_version: Option<i64>,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.events.read().await;
        let filtered: Vec<StoredEvent> = store
            .iter()
            .filter(|e| e.stream_id == stream_id)
            .filter(|e| e.version >= from_version)
            .filter(|e| to_version.is_none_or(|to| e.version <= to))
            .filter(|e| event_type.as_ref().is_none_or(|et| &e.event_type == et))
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let result: Vec<StoredEvent> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((result, total))
    }

    async fn find_all(
        &self,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.events.read().await;
        let filtered: Vec<StoredEvent> = store
            .iter()
            .filter(|e| event_type.as_ref().is_none_or(|et| &e.event_type == et))
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let result: Vec<StoredEvent> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((result, total))
    }

    async fn find_by_sequence(
        &self,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.events.read().await;
        Ok(store
            .iter()
            .find(|e| e.stream_id == stream_id && e.sequence == sequence)
            .cloned())
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut store = self.events.write().await;
        let before = store.len();
        store.retain(|e| e.stream_id != stream_id);
        Ok((before - store.len()) as u64)
    }
}

// ============================================================================
// Test Stub: In-Memory SnapshotRepository
// ============================================================================

struct StubSnapshotRepository {
    snapshots: RwLock<Vec<Snapshot>>,
    should_fail: bool,
}

impl StubSnapshotRepository {
    fn new() -> Self {
        Self {
            snapshots: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_snapshots(snapshots: Vec<Snapshot>) -> Self {
        Self {
            snapshots: RwLock::new(snapshots),
            should_fail: false,
        }
    }
}

#[async_trait]
impl SnapshotRepository for StubSnapshotRepository {
    async fn create(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut store = self.snapshots.write().await;
        store.push(snapshot.clone());
        Ok(())
    }

    async fn find_latest(&self, stream_id: &str) -> anyhow::Result<Option<Snapshot>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.snapshots.read().await;
        Ok(store
            .iter()
            .filter(|s| s.stream_id == stream_id)
            .max_by_key(|s| s.snapshot_version)
            .cloned())
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut store = self.snapshots.write().await;
        let before = store.len();
        store.retain(|s| s.stream_id != stream_id);
        Ok((before - store.len()) as u64)
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn make_stream(id: &str, aggregate_type: &str, version: i64) -> EventStream {
    let now = chrono::Utc::now();
    EventStream {
        id: id.to_string(),
        aggregate_type: aggregate_type.to_string(),
        current_version: version,
        created_at: now,
        updated_at: now,
    }
}

fn make_event(stream_id: &str, sequence: u64, version: i64, event_type: &str) -> StoredEvent {
    StoredEvent::new(
        stream_id.to_string(),
        sequence,
        event_type.to_string(),
        version,
        serde_json::json!({"key": "value"}),
        EventMetadata::new(Some("user-001".to_string()), None, None),
    )
}

fn make_event_data(event_type: &str) -> EventData {
    EventData {
        event_type: event_type.to_string(),
        payload: serde_json::json!({"key": "value"}),
        metadata: EventMetadata::new(Some("user-001".to_string()), None, None),
    }
}

fn make_snapshot(stream_id: &str, version: i64) -> Snapshot {
    Snapshot::new(
        format!("snap_{}", version),
        stream_id.to_string(),
        version,
        "Order".to_string(),
        serde_json::json!({"status": "active"}),
    )
}

// ============================================================================
// AppendEventsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_append_events_creates_new_stream() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: -1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok(), "expected success, got: {:?}", result.err());

    let output = result.unwrap();
    assert_eq!(output.stream_id, "order-001");
    assert_eq!(output.current_version, 1);
    assert_eq!(output.events.len(), 1);
    assert_eq!(output.events[0].event_type, "OrderPlaced");

    // Verify stream was created
    let streams = stream_repo.streams.read().await;
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "order-001");
    assert_eq!(streams[0].current_version, 1);
}

#[tokio::test]
async fn test_append_events_to_existing_stream() {
    let stream = make_stream("order-001", "Order", 2);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![
            make_event_data("OrderShipped"),
            make_event_data("OrderDelivered"),
        ],
        expected_version: 2,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.current_version, 4);
    assert_eq!(output.events.len(), 2);

    // Verify stream version updated
    let streams = stream_repo.streams.read().await;
    assert_eq!(streams[0].current_version, 4);
}

#[tokio::test]
async fn test_append_events_version_conflict() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo, event_repo);

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: 2,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        AppendEventsError::VersionConflict {
            stream_id,
            expected,
            actual,
        } => {
            assert_eq!(stream_id, "order-001");
            assert_eq!(expected, 2);
            assert_eq!(actual, 5);
        }
        e => panic!("expected VersionConflict, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_append_events_stream_not_found() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo, event_repo);

    let input = AppendEventsInput {
        stream_id: "nonexistent".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: 0,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AppendEventsError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_append_events_stream_already_exists() {
    let stream = make_stream("order-001", "Order", 0);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo, event_repo);

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: -1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AppendEventsError::StreamAlreadyExists(_)
    ));
}

#[tokio::test]
async fn test_append_events_validation_empty_events() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo, event_repo);

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![],
        expected_version: -1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AppendEventsError::Validation(_)
    ));
}

#[tokio::test]
async fn test_append_events_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo, event_repo);

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: 0,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppendEventsError::Internal(msg) => {
            assert!(msg.contains("unavailable"), "unexpected msg: {}", msg);
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_append_multiple_events_increments_versions_correctly() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());

    let input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![
            make_event_data("OrderPlaced"),
            make_event_data("OrderConfirmed"),
            make_event_data("OrderShipped"),
        ],
        expected_version: -1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.current_version, 3);
    assert_eq!(output.events.len(), 3);
    assert_eq!(output.events[0].version, 1);
    assert_eq!(output.events[1].version, 2);
    assert_eq!(output.events[2].version, 3);
}

// ============================================================================
// ReadEventsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_read_events_success() {
    let stream = make_stream("order-001", "Order", 3);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-001", 2, 2, "OrderConfirmed"),
        make_event("order-001", 3, 3, "OrderShipped"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ReadEventsUseCase::new(stream_repo, event_repo);

    let input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 1,
        to_version: None,
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.stream_id, "order-001");
    assert_eq!(output.events.len(), 3);
    assert_eq!(output.current_version, 3);
    assert_eq!(output.pagination.total_count, 3);
    assert!(!output.pagination.has_next);
}

#[tokio::test]
async fn test_read_events_with_version_range() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-001", 2, 2, "OrderConfirmed"),
        make_event("order-001", 3, 3, "OrderShipped"),
        make_event("order-001", 4, 4, "OrderDelivered"),
        make_event("order-001", 5, 5, "OrderCompleted"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ReadEventsUseCase::new(stream_repo, event_repo);

    let input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 2,
        to_version: Some(4),
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.events.len(), 3);
    assert_eq!(output.events[0].version, 2);
    assert_eq!(output.events[2].version, 4);
}

#[tokio::test]
async fn test_read_events_with_event_type_filter() {
    let stream = make_stream("order-001", "Order", 3);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-001", 2, 2, "OrderConfirmed"),
        make_event("order-001", 3, 3, "OrderPlaced"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ReadEventsUseCase::new(stream_repo, event_repo);

    let input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 1,
        to_version: None,
        event_type: Some("OrderPlaced".to_string()),
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.events.len(), 2);
    assert!(output.events.iter().all(|e| e.event_type == "OrderPlaced"));
}

#[tokio::test]
async fn test_read_events_stream_not_found() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = ReadEventsUseCase::new(stream_repo, event_repo);

    let input = ReadEventsInput {
        stream_id: "nonexistent".to_string(),
        from_version: 1,
        to_version: None,
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ReadEventsError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_read_events_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = ReadEventsUseCase::new(stream_repo, event_repo);

    let input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 1,
        to_version: None,
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ReadEventsError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_read_events_pagination_has_next() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let events: Vec<StoredEvent> = (1..=5)
        .map(|i| make_event("order-001", i, i as i64, "OrderPlaced"))
        .collect();
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ReadEventsUseCase::new(stream_repo, event_repo);

    let input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 1,
        to_version: None,
        event_type: None,
        page: 1,
        page_size: 2,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.events.len(), 2);
    assert!(output.pagination.has_next);
    assert_eq!(output.pagination.total_count, 5);
}

// ============================================================================
// ReadEventBySequenceUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_read_event_by_sequence_success() {
    let stream = make_stream("order-001", "Order", 3);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-001", 2, 2, "OrderConfirmed"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ReadEventBySequenceUseCase::new(stream_repo, event_repo);

    let input = ReadEventBySequenceInput {
        stream_id: "order-001".to_string(),
        sequence: 1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.stream_id, "order-001");
    assert_eq!(event.sequence, 1);
    assert_eq!(event.event_type, "OrderPlaced");
}

#[tokio::test]
async fn test_read_event_by_sequence_stream_not_found() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = ReadEventBySequenceUseCase::new(stream_repo, event_repo);

    let input = ReadEventBySequenceInput {
        stream_id: "nonexistent".to_string(),
        sequence: 1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ReadEventBySequenceError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_read_event_by_sequence_event_not_found() {
    let stream = make_stream("order-001", "Order", 3);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = ReadEventBySequenceUseCase::new(stream_repo, event_repo);

    let input = ReadEventBySequenceInput {
        stream_id: "order-001".to_string(),
        sequence: 999,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ReadEventBySequenceError::EventNotFound { .. }
    ));
}

#[tokio::test]
async fn test_read_event_by_sequence_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = ReadEventBySequenceUseCase::new(stream_repo, event_repo);

    let input = ReadEventBySequenceInput {
        stream_id: "order-001".to_string(),
        sequence: 1,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ReadEventBySequenceError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// CreateSnapshotUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_create_snapshot_success() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = CreateSnapshotUseCase::new(stream_repo, snapshot_repo.clone());

    let input = CreateSnapshotInput {
        stream_id: "order-001".to_string(),
        snapshot_version: 3,
        aggregate_type: "Order".to_string(),
        state: serde_json::json!({"status": "shipped"}),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.stream_id, "order-001");
    assert_eq!(output.snapshot_version, 3);
    assert!(output.id.starts_with("snap_"));

    // Verify snapshot was persisted
    let snapshots = snapshot_repo.snapshots.read().await;
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].snapshot_version, 3);
}

#[tokio::test]
async fn test_create_snapshot_stream_not_found() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = CreateSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = CreateSnapshotInput {
        stream_id: "nonexistent".to_string(),
        snapshot_version: 1,
        aggregate_type: "Order".to_string(),
        state: serde_json::json!({}),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateSnapshotError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_create_snapshot_version_exceeds_current() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = CreateSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = CreateSnapshotInput {
        stream_id: "order-001".to_string(),
        snapshot_version: 10,
        aggregate_type: "Order".to_string(),
        state: serde_json::json!({}),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CreateSnapshotError::Validation(msg) => {
            assert!(msg.contains("exceeds"));
        }
        e => panic!("expected Validation, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_create_snapshot_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = CreateSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = CreateSnapshotInput {
        stream_id: "order-001".to_string(),
        snapshot_version: 1,
        aggregate_type: "Order".to_string(),
        state: serde_json::json!({}),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CreateSnapshotError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_create_snapshot_at_current_version() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = CreateSnapshotUseCase::new(stream_repo, snapshot_repo.clone());

    let input = CreateSnapshotInput {
        stream_id: "order-001".to_string(),
        snapshot_version: 5,
        aggregate_type: "Order".to_string(),
        state: serde_json::json!({"status": "latest"}),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().snapshot_version, 5);
}

// ============================================================================
// GetLatestSnapshotUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_latest_snapshot_success() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let snapshots = vec![make_snapshot("order-001", 2), make_snapshot("order-001", 4)];
    let snapshot_repo = Arc::new(StubSnapshotRepository::with_snapshots(snapshots));

    let uc = GetLatestSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = GetLatestSnapshotInput {
        stream_id: "order-001".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let snap = result.unwrap();
    assert_eq!(snap.stream_id, "order-001");
    assert_eq!(snap.snapshot_version, 4);
}

#[tokio::test]
async fn test_get_latest_snapshot_stream_not_found() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = GetLatestSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = GetLatestSnapshotInput {
        stream_id: "nonexistent".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GetLatestSnapshotError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_get_latest_snapshot_no_snapshot_exists() {
    let stream = make_stream("order-001", "Order", 5);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = GetLatestSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = GetLatestSnapshotInput {
        stream_id: "order-001".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GetLatestSnapshotError::SnapshotNotFound(_)
    ));
}

#[tokio::test]
async fn test_get_latest_snapshot_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = GetLatestSnapshotUseCase::new(stream_repo, snapshot_repo);

    let input = GetLatestSnapshotInput {
        stream_id: "order-001".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetLatestSnapshotError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// ListEventsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_list_events_success() {
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-002", 2, 1, "OrderPlaced"),
        make_event("order-001", 3, 2, "OrderConfirmed"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ListEventsUseCase::new(event_repo);

    let input = ListEventsInput {
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.events.len(), 3);
    assert_eq!(output.pagination.total_count, 3);
    assert!(!output.pagination.has_next);
}

#[tokio::test]
async fn test_list_events_with_type_filter() {
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-002", 2, 1, "OrderConfirmed"),
        make_event("order-003", 3, 1, "OrderPlaced"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ListEventsUseCase::new(event_repo);

    let input = ListEventsInput {
        event_type: Some("OrderPlaced".to_string()),
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.events.len(), 2);
    assert!(output.events.iter().all(|e| e.event_type == "OrderPlaced"));
}

#[tokio::test]
async fn test_list_events_pagination() {
    let events: Vec<StoredEvent> = (1..=5)
        .map(|i| make_event("order-001", i, i as i64, "OrderPlaced"))
        .collect();
    let event_repo = Arc::new(StubEventRepository::with_events(events));

    let uc = ListEventsUseCase::new(event_repo);

    // First page
    let input = ListEventsInput {
        event_type: None,
        page: 1,
        page_size: 2,
    };
    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.events.len(), 2);
    assert!(output.pagination.has_next);

    // Last page
    let input = ListEventsInput {
        event_type: None,
        page: 3,
        page_size: 2,
    };
    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.events.len(), 1);
    assert!(!output.pagination.has_next);
}

#[tokio::test]
async fn test_list_events_empty() {
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = ListEventsUseCase::new(event_repo);

    let input = ListEventsInput {
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert!(result.unwrap().events.is_empty());
}

#[tokio::test]
async fn test_list_events_internal_error() {
    let event_repo = Arc::new(StubEventRepository::with_error());

    let uc = ListEventsUseCase::new(event_repo);

    let input = ListEventsInput {
        event_type: None,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ListEventsError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// ListStreamsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_list_streams_success() {
    let streams = vec![
        make_stream("order-001", "Order", 3),
        make_stream("payment-001", "Payment", 1),
        make_stream("order-002", "Order", 5),
    ];
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(streams));

    let uc = ListStreamsUseCase::new(stream_repo);

    let input = ListStreamsInput {
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.streams.len(), 3);
    assert_eq!(output.pagination.total_count, 3);
    assert!(!output.pagination.has_next);
}

#[tokio::test]
async fn test_list_streams_pagination() {
    let streams: Vec<EventStream> = (1..=5)
        .map(|i| make_stream(&format!("stream-{:03}", i), "Order", i))
        .collect();
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(streams));

    let uc = ListStreamsUseCase::new(stream_repo);

    let input = ListStreamsInput {
        page: 1,
        page_size: 2,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.streams.len(), 2);
    assert!(output.pagination.has_next);
    assert_eq!(output.pagination.total_count, 5);
}

#[tokio::test]
async fn test_list_streams_empty() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());

    let uc = ListStreamsUseCase::new(stream_repo);

    let input = ListStreamsInput {
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert!(result.unwrap().streams.is_empty());
}

#[tokio::test]
async fn test_list_streams_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());

    let uc = ListStreamsUseCase::new(stream_repo);

    let input = ListStreamsInput {
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ListStreamsError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// DeleteStreamUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_delete_stream_success() {
    let stream = make_stream("order-001", "Order", 3);
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(vec![stream]));
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-001", 2, 2, "OrderConfirmed"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));
    let snapshots = vec![make_snapshot("order-001", 2)];
    let snapshot_repo = Arc::new(StubSnapshotRepository::with_snapshots(snapshots));

    let uc = DeleteStreamUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
        snapshot_repo.clone(),
    );

    let input = DeleteStreamInput {
        stream_id: "order-001".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert!(output.message.contains("order-001"));

    // Verify all data was deleted
    let streams = stream_repo.streams.read().await;
    assert!(streams.is_empty());

    let events = event_repo.events.read().await;
    assert!(events.is_empty());

    let snaps = snapshot_repo.snapshots.read().await;
    assert!(snaps.is_empty());
}

#[tokio::test]
async fn test_delete_stream_not_found() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = DeleteStreamUseCase::new(stream_repo, event_repo, snapshot_repo);

    let input = DeleteStreamInput {
        stream_id: "nonexistent".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeleteStreamError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_delete_stream_internal_error() {
    let stream_repo = Arc::new(StubEventStreamRepository::with_error());
    let event_repo = Arc::new(StubEventRepository::new());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = DeleteStreamUseCase::new(stream_repo, event_repo, snapshot_repo);

    let input = DeleteStreamInput {
        stream_id: "order-001".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DeleteStreamError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_delete_stream_preserves_other_streams() {
    let streams = vec![
        make_stream("order-001", "Order", 3),
        make_stream("order-002", "Order", 1),
    ];
    let stream_repo = Arc::new(StubEventStreamRepository::with_streams(streams));
    let events = vec![
        make_event("order-001", 1, 1, "OrderPlaced"),
        make_event("order-002", 2, 1, "OrderPlaced"),
    ];
    let event_repo = Arc::new(StubEventRepository::with_events(events));
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let uc = DeleteStreamUseCase::new(stream_repo.clone(), event_repo.clone(), snapshot_repo);

    let input = DeleteStreamInput {
        stream_id: "order-001".to_string(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    // order-002 should still exist
    let streams = stream_repo.streams.read().await;
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].id, "order-002");

    let events = event_repo.events.read().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].stream_id, "order-002");
}

// ============================================================================
// Cross-cutting / Integration-style Usecase Tests
// ============================================================================

#[tokio::test]
async fn test_append_then_read_events_roundtrip() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let append_uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());
    let read_uc = ReadEventsUseCase::new(stream_repo.clone(), event_repo.clone());

    // Append events to new stream
    let append_input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![
            make_event_data("OrderPlaced"),
            make_event_data("OrderConfirmed"),
        ],
        expected_version: -1,
    };
    let append_result = append_uc.execute(&append_input).await;
    assert!(append_result.is_ok());

    // Read events back
    let read_input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 1,
        to_version: None,
        event_type: None,
        page: 1,
        page_size: 50,
    };
    let read_result = read_uc.execute(&read_input).await;
    assert!(read_result.is_ok());

    let output = read_result.unwrap();
    assert_eq!(output.events.len(), 2);
    assert_eq!(output.events[0].event_type, "OrderPlaced");
    assert_eq!(output.events[1].event_type, "OrderConfirmed");
    assert_eq!(output.current_version, 2);
}

#[tokio::test]
async fn test_append_create_snapshot_then_get_snapshot() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let append_uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());
    let snap_uc = CreateSnapshotUseCase::new(stream_repo.clone(), snapshot_repo.clone());
    let get_snap_uc = GetLatestSnapshotUseCase::new(stream_repo.clone(), snapshot_repo.clone());

    // Create stream with events
    let append_input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![
            make_event_data("OrderPlaced"),
            make_event_data("OrderConfirmed"),
            make_event_data("OrderShipped"),
        ],
        expected_version: -1,
    };
    assert!(append_uc.execute(&append_input).await.is_ok());

    // Create snapshot at version 2
    let snap_input = CreateSnapshotInput {
        stream_id: "order-001".to_string(),
        snapshot_version: 2,
        aggregate_type: "Order".to_string(),
        state: serde_json::json!({"status": "confirmed"}),
    };
    assert!(snap_uc.execute(&snap_input).await.is_ok());

    // Get latest snapshot
    let get_input = GetLatestSnapshotInput {
        stream_id: "order-001".to_string(),
    };
    let result = get_snap_uc.execute(&get_input).await;
    assert!(result.is_ok());

    let snap = result.unwrap();
    assert_eq!(snap.snapshot_version, 2);
    assert_eq!(snap.state["status"], "confirmed");
}

#[tokio::test]
async fn test_append_then_delete_then_read_fails() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());
    let snapshot_repo = Arc::new(StubSnapshotRepository::new());

    let append_uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());
    let delete_uc = DeleteStreamUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
        snapshot_repo.clone(),
    );
    let read_uc = ReadEventsUseCase::new(stream_repo.clone(), event_repo.clone());

    // Append
    let append_input = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: -1,
    };
    assert!(append_uc.execute(&append_input).await.is_ok());

    // Delete
    let delete_input = DeleteStreamInput {
        stream_id: "order-001".to_string(),
    };
    assert!(delete_uc.execute(&delete_input).await.is_ok());

    // Read should fail
    let read_input = ReadEventsInput {
        stream_id: "order-001".to_string(),
        from_version: 1,
        to_version: None,
        event_type: None,
        page: 1,
        page_size: 50,
    };
    let result = read_uc.execute(&read_input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ReadEventsError::StreamNotFound(_)
    ));
}

#[tokio::test]
async fn test_list_streams_after_multiple_appends() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let append_uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());
    let list_uc = ListStreamsUseCase::new(stream_repo.clone());

    for name in &["order-001", "order-002", "payment-001"] {
        let input = AppendEventsInput {
            stream_id: name.to_string(),
            aggregate_type: Some("Order".to_string()),
            events: vec![make_event_data("Created")],
            expected_version: -1,
        };
        assert!(append_uc.execute(&input).await.is_ok());
    }

    let result = list_uc
        .execute(&ListStreamsInput {
            page: 1,
            page_size: 50,
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().streams.len(), 3);
}

#[tokio::test]
async fn test_optimistic_concurrency_sequential_appends() {
    let stream_repo = Arc::new(StubEventStreamRepository::new());
    let event_repo = Arc::new(StubEventRepository::new());

    let uc = AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone());

    // Create stream
    let input1 = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: Some("Order".to_string()),
        events: vec![make_event_data("OrderPlaced")],
        expected_version: -1,
    };
    let result1 = uc.execute(&input1).await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().current_version, 1);

    // Append at version 1
    let input2 = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: None,
        events: vec![make_event_data("OrderConfirmed")],
        expected_version: 1,
    };
    let result2 = uc.execute(&input2).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().current_version, 2);

    // Stale version should fail
    let input3 = AppendEventsInput {
        stream_id: "order-001".to_string(),
        aggregate_type: None,
        events: vec![make_event_data("OrderShipped")],
        expected_version: 1,
    };
    let result3 = uc.execute(&input3).await;
    assert!(result3.is_err());
    assert!(matches!(
        result3.unwrap_err(),
        AppendEventsError::VersionConflict { .. }
    ));
}
