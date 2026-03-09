use k1s0_audit_client::{AuditClient, AuditError, AuditEvent, BufferedAuditClient};
use serde_json::json;

// ============================================================
// AuditEvent construction
// ============================================================

#[test]
fn audit_event_new_sets_all_fields() {
    let event = AuditEvent::new(
        "tenant-1",
        "user-42",
        "create",
        "document",
        "doc-123",
        json!({"key": "value"}),
    );
    assert_eq!(event.tenant_id, "tenant-1");
    assert_eq!(event.actor_id, "user-42");
    assert_eq!(event.action, "create");
    assert_eq!(event.resource_type, "document");
    assert_eq!(event.resource_id, "doc-123");
    assert_eq!(event.metadata["key"], "value");
}

#[test]
fn audit_event_generates_unique_ids() {
    let e1 = AuditEvent::new("t", "a", "act", "res", "r1", json!({}));
    let e2 = AuditEvent::new("t", "a", "act", "res", "r2", json!({}));
    assert_ne!(e1.id, e2.id);
}

#[test]
fn audit_event_timestamp_is_set() {
    let event = AuditEvent::new("t", "a", "act", "res", "r1", json!({}));
    // Timestamp should be recent (within last 5 seconds)
    let now = chrono::Utc::now();
    let diff = now - event.timestamp;
    assert!(diff.num_seconds() < 5);
}

#[test]
fn audit_event_accepts_string_owned_args() {
    let event = AuditEvent::new(
        String::from("tenant"),
        String::from("actor"),
        String::from("delete"),
        String::from("user"),
        String::from("usr-1"),
        json!(null),
    );
    assert_eq!(event.tenant_id, "tenant");
    assert_eq!(event.action, "delete");
}

#[test]
fn audit_event_with_complex_metadata() {
    let metadata = json!({
        "ip": "192.168.1.1",
        "user_agent": "Mozilla/5.0",
        "changes": {
            "field": "name",
            "old": "Alice",
            "new": "Bob"
        },
        "tags": ["important", "security"]
    });
    let event = AuditEvent::new("t", "a", "update", "user", "u1", metadata.clone());
    assert_eq!(event.metadata, metadata);
}

#[test]
fn audit_event_with_empty_metadata() {
    let event = AuditEvent::new("t", "a", "act", "res", "r1", json!({}));
    assert!(event.metadata.is_object());
    assert_eq!(event.metadata.as_object().unwrap().len(), 0);
}

#[test]
fn audit_event_with_null_metadata() {
    let event = AuditEvent::new("t", "a", "act", "res", "r1", json!(null));
    assert!(event.metadata.is_null());
}

#[test]
fn audit_event_clone() {
    let event = AuditEvent::new("t", "a", "act", "res", "r1", json!({"x": 1}));
    let cloned = event.clone();
    assert_eq!(event.id, cloned.id);
    assert_eq!(event.tenant_id, cloned.tenant_id);
    assert_eq!(event.metadata, cloned.metadata);
}

#[test]
fn audit_event_serialization_roundtrip() {
    let event = AuditEvent::new("t-1", "a-1", "read", "file", "f-1", json!({"size": 42}));
    let json_str = serde_json::to_string(&event).unwrap();
    let deserialized: AuditEvent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(event.id, deserialized.id);
    assert_eq!(event.tenant_id, deserialized.tenant_id);
    assert_eq!(event.action, deserialized.action);
    assert_eq!(event.resource_type, deserialized.resource_type);
    assert_eq!(event.resource_id, deserialized.resource_id);
    assert_eq!(event.metadata, deserialized.metadata);
}

// ============================================================
// BufferedAuditClient: record single event
// ============================================================

#[tokio::test]
async fn record_single_event() {
    let client = BufferedAuditClient::new();
    let event = AuditEvent::new("t", "a", "create", "doc", "d1", json!({}));
    let event_id = event.id;

    client.record(event).await.unwrap();
    let flushed = client.flush().await.unwrap();

    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].id, event_id);
}

// ============================================================
// BufferedAuditClient: events buffered until flush
// ============================================================

#[tokio::test]
async fn events_buffered_until_flush() {
    let client = BufferedAuditClient::new();

    for i in 0..3 {
        let event = AuditEvent::new("t", "a", format!("action-{}", i), "res", "r", json!({}));
        client.record(event).await.unwrap();
    }

    // Events should accumulate
    let flushed = client.flush().await.unwrap();
    assert_eq!(flushed.len(), 3);
    assert_eq!(flushed[0].action, "action-0");
    assert_eq!(flushed[1].action, "action-1");
    assert_eq!(flushed[2].action, "action-2");
}

// ============================================================
// BufferedAuditClient: flush empties buffer
// ============================================================

#[tokio::test]
async fn flush_empties_buffer() {
    let client = BufferedAuditClient::new();
    let event = AuditEvent::new("t", "a", "act", "res", "r1", json!({}));
    client.record(event).await.unwrap();

    let first_flush = client.flush().await.unwrap();
    assert_eq!(first_flush.len(), 1);

    let second_flush = client.flush().await.unwrap();
    assert!(second_flush.is_empty());
}

// ============================================================
// BufferedAuditClient: flush on empty buffer
// ============================================================

#[tokio::test]
async fn flush_empty_buffer_returns_empty_vec() {
    let client = BufferedAuditClient::new();
    let flushed = client.flush().await.unwrap();
    assert!(flushed.is_empty());
}

// ============================================================
// BufferedAuditClient: large number of events
// ============================================================

#[tokio::test]
async fn buffer_handles_many_events() {
    let client = BufferedAuditClient::new();
    let count = 1000;

    for i in 0..count {
        let event = AuditEvent::new(
            "t",
            "a",
            format!("action-{}", i),
            "res",
            format!("r-{}", i),
            json!({"index": i}),
        );
        client.record(event).await.unwrap();
    }

    let flushed = client.flush().await.unwrap();
    assert_eq!(flushed.len(), count);
    // Verify ordering is preserved
    for (i, event) in flushed.iter().enumerate() {
        assert_eq!(event.action, format!("action-{}", i));
        assert_eq!(event.resource_id, format!("r-{}", i));
    }
}

// ============================================================
// BufferedAuditClient: multiple flush cycles
// ============================================================

#[tokio::test]
async fn multiple_flush_cycles() {
    let client = BufferedAuditClient::new();

    // Cycle 1
    client
        .record(AuditEvent::new("t", "a", "c1-act", "res", "r1", json!({})))
        .await
        .unwrap();
    let batch1 = client.flush().await.unwrap();
    assert_eq!(batch1.len(), 1);
    assert_eq!(batch1[0].action, "c1-act");

    // Cycle 2
    client
        .record(AuditEvent::new("t", "a", "c2-act1", "res", "r2", json!({})))
        .await
        .unwrap();
    client
        .record(AuditEvent::new("t", "a", "c2-act2", "res", "r3", json!({})))
        .await
        .unwrap();
    let batch2 = client.flush().await.unwrap();
    assert_eq!(batch2.len(), 2);
    assert_eq!(batch2[0].action, "c2-act1");
    assert_eq!(batch2[1].action, "c2-act2");
}

// ============================================================
// BufferedAuditClient: Default trait
// ============================================================

#[tokio::test]
async fn default_creates_empty_client() {
    let client = BufferedAuditClient::default();
    let flushed = client.flush().await.unwrap();
    assert!(flushed.is_empty());
}

// ============================================================
// BufferedAuditClient: event data integrity through buffer
// ============================================================

#[tokio::test]
async fn event_data_preserved_through_buffer() {
    let client = BufferedAuditClient::new();
    let metadata = json!({
        "ip": "10.0.0.1",
        "nested": {"deep": true}
    });
    let event = AuditEvent::new("tenant-abc", "actor-xyz", "update", "config", "cfg-99", metadata.clone());
    let original_id = event.id;
    let original_timestamp = event.timestamp;

    client.record(event).await.unwrap();
    let flushed = client.flush().await.unwrap();

    assert_eq!(flushed[0].id, original_id);
    assert_eq!(flushed[0].timestamp, original_timestamp);
    assert_eq!(flushed[0].tenant_id, "tenant-abc");
    assert_eq!(flushed[0].actor_id, "actor-xyz");
    assert_eq!(flushed[0].action, "update");
    assert_eq!(flushed[0].resource_type, "config");
    assert_eq!(flushed[0].resource_id, "cfg-99");
    assert_eq!(flushed[0].metadata, metadata);
}

// ============================================================
// AuditError
// ============================================================

#[test]
fn audit_error_send_error_display() {
    let err = AuditError::SendError("connection refused".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("connection refused"));
}

#[test]
fn audit_error_internal_display() {
    let err = AuditError::Internal("unexpected state".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("unexpected state"));
}

#[test]
fn audit_error_serialization_from_serde() {
    // Create a serde_json error by trying to parse invalid JSON
    let serde_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let audit_err: AuditError = serde_err.into();
    match audit_err {
        AuditError::SerializationError(_) => {}
        other => panic!("expected SerializationError, got: {:?}", other),
    }
}

#[test]
fn audit_error_debug_format() {
    let err = AuditError::SendError("timeout".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("SendError"));
    assert!(debug.contains("timeout"));
}
