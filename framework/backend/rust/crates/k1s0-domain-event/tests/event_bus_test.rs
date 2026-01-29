use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;

use k1s0_domain_event::{
    DomainEvent, EventEnvelope, EventHandler, EventPublisher, EventSubscriber, HandlerError,
};
use k1s0_domain_event::bus::InMemoryEventBus;

#[derive(Debug, Serialize)]
struct TestEvent {
    message: String,
}

impl DomainEvent for TestEvent {
    fn event_type(&self) -> &str {
        "test.event"
    }
}

struct CountingHandler {
    count: Arc<AtomicU32>,
}

#[async_trait]
impl EventHandler for CountingHandler {
    fn event_type(&self) -> &str {
        "test.event"
    }

    async fn handle(&self, _envelope: &EventEnvelope) -> Result<(), HandlerError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_publish_and_subscribe() {
    let bus = InMemoryEventBus::new(16);
    let count = Arc::new(AtomicU32::new(0));

    let handler = CountingHandler {
        count: Arc::clone(&count),
    };
    let _handle = bus.subscribe(Box::new(handler)).await.unwrap();

    let event = TestEvent {
        message: "hello".into(),
    };
    let envelope = EventEnvelope::from_event(&event, "test-service").unwrap();
    bus.publish(envelope).await.unwrap();

    // ハンドラがイベントを処理するのを待つ
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_unmatched_event_type_ignored() {
    let bus = InMemoryEventBus::new(16);
    let count = Arc::new(AtomicU32::new(0));

    let handler = CountingHandler {
        count: Arc::clone(&count),
    };
    let _handle = bus.subscribe(Box::new(handler)).await.unwrap();

    // 異なるイベント型を発行
    let envelope = EventEnvelope {
        event_type: "other.event".into(),
        metadata: k1s0_domain_event::EventMetadata::new("test"),
        payload: serde_json::json!({}),
    };
    bus.publish(envelope).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_cancel_subscription() {
    let bus = InMemoryEventBus::new(16);
    let count = Arc::new(AtomicU32::new(0));

    let handler = CountingHandler {
        count: Arc::clone(&count),
    };
    let handle = bus.subscribe(Box::new(handler)).await.unwrap();

    // キャンセル後はイベントを処理しない
    handle.cancel();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let event = TestEvent {
        message: "after cancel".into(),
    };
    let envelope = EventEnvelope::from_event(&event, "test-service").unwrap();
    // レシーバが全てなくなった場合は Send エラーになるため、エラーも許容する
    let _ = bus.publish(envelope).await;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[test]
fn test_envelope_from_event() {
    let event = TestEvent {
        message: "test".into(),
    };
    let envelope = EventEnvelope::from_event(&event, "my-service").unwrap();

    assert_eq!(envelope.event_type, "test.event");
    assert_eq!(envelope.metadata.source, "my-service");
    assert!(envelope.metadata.correlation_id.is_none());
}

#[test]
fn test_metadata_builder() {
    let metadata = k1s0_domain_event::EventMetadata::new("svc")
        .with_correlation_id("corr-123")
        .with_causation_id("cause-456");

    assert_eq!(metadata.source, "svc");
    assert_eq!(metadata.correlation_id.as_deref(), Some("corr-123"));
    assert_eq!(metadata.causation_id.as_deref(), Some("cause-456"));
}
