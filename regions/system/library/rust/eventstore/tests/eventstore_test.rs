use k1s0_eventstore::{
    EventEnvelope, EventStore, InMemoryEventStore, InMemorySnapshotStore, Snapshot, SnapshotStore,
    StreamId,
};

fn make_event(stream_id: &StreamId, event_type: &str) -> EventEnvelope {
    EventEnvelope::new(
        stream_id,
        0,
        event_type,
        serde_json::json!({"key": "value"}),
    )
}

#[tokio::test]
async fn test_append_and_load_events() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-123");

    let events = vec![
        make_event(&stream_id, "OrderCreated"),
        make_event(&stream_id, "OrderConfirmed"),
    ];

    store.append(&stream_id, events, None).await.unwrap();

    let loaded = store.load(&stream_id).await.unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].event_type, "OrderCreated");
    assert_eq!(loaded[1].event_type, "OrderConfirmed");
    assert_eq!(loaded[0].version, 1);
    assert_eq!(loaded[1].version, 2);
}

#[tokio::test]
async fn test_exists_false_before_append() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-999");

    assert!(!store.exists(&stream_id).await.unwrap());
}

#[tokio::test]
async fn test_exists_true_after_append() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-100");

    let events = vec![make_event(&stream_id, "OrderCreated")];
    store.append(&stream_id, events, None).await.unwrap();

    assert!(store.exists(&stream_id).await.unwrap());
}

#[tokio::test]
async fn test_load_from_version() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-200");

    let events = vec![
        make_event(&stream_id, "OrderCreated"),
        make_event(&stream_id, "OrderConfirmed"),
        make_event(&stream_id, "OrderShipped"),
    ];
    store.append(&stream_id, events, None).await.unwrap();

    let loaded = store.load_from(&stream_id, 2).await.unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].event_type, "OrderConfirmed");
    assert_eq!(loaded[0].version, 2);
    assert_eq!(loaded[1].event_type, "OrderShipped");
    assert_eq!(loaded[1].version, 3);
}

#[tokio::test]
async fn test_version_conflict_on_wrong_expected() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-300");

    let events = vec![make_event(&stream_id, "OrderCreated")];
    store.append(&stream_id, events, None).await.unwrap();

    let events2 = vec![make_event(&stream_id, "OrderConfirmed")];
    let result = store.append(&stream_id, events2, Some(0)).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, k1s0_eventstore::EventStoreError::VersionConflict { expected: 0, actual: 1 })
    );
}

#[tokio::test]
async fn test_version_conflict_not_raised_with_correct_version() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-400");

    let events = vec![make_event(&stream_id, "OrderCreated")];
    store.append(&stream_id, events, None).await.unwrap();

    let events2 = vec![make_event(&stream_id, "OrderConfirmed")];
    let result = store.append(&stream_id, events2, Some(1)).await;

    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version, 2);
}

#[tokio::test]
async fn test_current_version_after_appends() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-500");

    assert_eq!(store.current_version(&stream_id).await.unwrap(), 0);

    let events = vec![make_event(&stream_id, "OrderCreated")];
    store.append(&stream_id, events, None).await.unwrap();
    assert_eq!(store.current_version(&stream_id).await.unwrap(), 1);

    let events2 = vec![
        make_event(&stream_id, "OrderConfirmed"),
        make_event(&stream_id, "OrderShipped"),
    ];
    store.append(&stream_id, events2, Some(1)).await.unwrap();
    assert_eq!(store.current_version(&stream_id).await.unwrap(), 3);
}

#[tokio::test]
async fn test_snapshot_save_and_load() {
    let store = InMemorySnapshotStore::new();
    let stream_id = StreamId::new("order-600");

    let snapshot = Snapshot {
        stream_id: stream_id.to_string(),
        version: 5,
        state: serde_json::json!({"status": "shipped"}),
        created_at: chrono::Utc::now(),
    };

    store.save_snapshot(snapshot.clone()).await.unwrap();

    let loaded = store.load_snapshot(&stream_id).await.unwrap();
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.version, 5);
    assert_eq!(loaded.state, serde_json::json!({"status": "shipped"}));
}

#[tokio::test]
async fn test_load_nonexistent_stream_returns_empty() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("nonexistent-stream");

    let events = store.load(&stream_id).await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn test_append_multiple_events_increments_version() {
    let store = InMemoryEventStore::new();
    let stream_id = StreamId::new("order-700");

    let events = vec![
        make_event(&stream_id, "OrderCreated"),
        make_event(&stream_id, "ItemAdded"),
        make_event(&stream_id, "ItemAdded"),
        make_event(&stream_id, "OrderConfirmed"),
    ];

    let final_version = store.append(&stream_id, events, None).await.unwrap();
    assert_eq!(final_version, 4);

    let loaded = store.load(&stream_id).await.unwrap();
    assert_eq!(loaded.len(), 4);
    for (i, event) in loaded.iter().enumerate() {
        assert_eq!(event.version, (i + 1) as u64);
    }
}
