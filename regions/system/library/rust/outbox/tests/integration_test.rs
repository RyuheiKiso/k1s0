use std::sync::Arc;

use k1s0_outbox::{OutboxMessage, OutboxStatus, OutboxProcessor, OutboxStore, OutboxError};
use k1s0_outbox::processor::OutboxPublisher;

#[test]
fn test_outbox_message_serialization_roundtrip() {
    let msg = OutboxMessage::new(
        "k1s0.service.order.created.v1",
        "ord-001",
        serde_json::json!({"order_id": "ord-001", "amount": 100}),
    );

    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: OutboxMessage = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.topic, msg.topic);
    assert_eq!(deserialized.partition_key, msg.partition_key);
    assert_eq!(deserialized.status, OutboxStatus::Pending);
    assert_eq!(deserialized.retry_count, 0);
    assert_eq!(deserialized.id, msg.id);
}

#[test]
fn test_message_state_pending_to_processing() {
    let mut msg = OutboxMessage::new(
        "k1s0.service.order.created.v1",
        "ord-001",
        serde_json::json!({}),
    );
    assert_eq!(msg.status, OutboxStatus::Pending);

    msg.mark_processing();
    assert_eq!(msg.status, OutboxStatus::Processing);
}

#[test]
fn test_message_state_processing_to_delivered() {
    let mut msg = OutboxMessage::new(
        "k1s0.service.order.created.v1",
        "ord-001",
        serde_json::json!({}),
    );
    msg.mark_processing();
    assert_eq!(msg.status, OutboxStatus::Processing);

    msg.mark_delivered();
    assert_eq!(msg.status, OutboxStatus::Delivered);
    assert!(!msg.is_processable());
}

#[test]
fn test_message_state_processing_to_failed() {
    let mut msg = OutboxMessage::new(
        "k1s0.service.order.created.v1",
        "ord-001",
        serde_json::json!({}),
    );
    msg.mark_processing();
    msg.mark_failed("kafka connection timeout");

    assert_eq!(msg.status, OutboxStatus::Failed);
    assert_eq!(msg.last_error.as_deref(), Some("kafka connection timeout"));
    assert_eq!(msg.retry_count, 1);
}

#[test]
fn test_message_retry_count_increments() {
    let mut msg = OutboxMessage::new(
        "k1s0.service.order.created.v1",
        "ord-001",
        serde_json::json!({}),
    );
    assert_eq!(msg.retry_count, 0);

    msg.mark_failed("error 1");
    assert_eq!(msg.retry_count, 1);
    assert_eq!(msg.status, OutboxStatus::Failed);

    msg.mark_failed("error 2");
    assert_eq!(msg.retry_count, 2);
    assert_eq!(msg.status, OutboxStatus::Failed);

    // max_retries はデフォルトで 3 なので、3回目で DeadLetter になる
    msg.mark_failed("error 3");
    assert_eq!(msg.retry_count, 3);
    assert_eq!(msg.status, OutboxStatus::DeadLetter);
}

#[test]
fn test_message_with_empty_payload() {
    let msg = OutboxMessage::new(
        "k1s0.service.notification.v1",
        "key-empty",
        serde_json::json!({}),
    );
    assert_eq!(msg.payload, serde_json::json!({}));
    assert_eq!(msg.status, OutboxStatus::Pending);
    assert_eq!(msg.topic, "k1s0.service.notification.v1");
    assert_eq!(msg.partition_key, "key-empty");
}

// --- Processor integration tests using mock store ---

use mockall::mock;

mock! {
    TestStore {}

    #[async_trait::async_trait]
    impl OutboxStore for TestStore {
        async fn save(&self, message: &OutboxMessage) -> Result<(), OutboxError>;
        async fn fetch_pending(&self, limit: u32) -> Result<Vec<OutboxMessage>, OutboxError>;
        async fn update(&self, message: &OutboxMessage) -> Result<(), OutboxError>;
        async fn delete_delivered(&self, older_than_days: u32) -> Result<u64, OutboxError>;
    }
}

struct AlwaysSuccessPublisher;

#[async_trait::async_trait]
impl OutboxPublisher for AlwaysSuccessPublisher {
    async fn publish(&self, _message: &OutboxMessage) -> Result<(), OutboxError> {
        Ok(())
    }
}

struct SelectivePublisher {
    /// publish が失敗するメッセージのインデックス（呼び出し順）
    fail_indices: Vec<usize>,
}

impl SelectivePublisher {
    fn new(fail_indices: Vec<usize>) -> Self {
        Self { fail_indices }
    }
}

#[async_trait::async_trait]
impl OutboxPublisher for SelectivePublisher {
    async fn publish(&self, _message: &OutboxMessage) -> Result<(), OutboxError> {
        // SelectivePublisher は process_batch の中で順番に呼ばれるため、
        // 呼び出しカウンタとしては使えない。
        // 代わりに、partition_key に "fail" が含まれるものを失敗させる。
        if _message.partition_key.contains("fail") {
            Err(OutboxError::PublishError("publish failed".to_string()))
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_processor_batch_size_respected() {
    let mut store = MockTestStore::new();

    // batch_size=5 が fetch_pending に渡されることを確認
    store
        .expect_fetch_pending()
        .withf(|&limit| limit == 5)
        .returning(|_| Ok(vec![]));

    let processor = OutboxProcessor::new(
        Arc::new(store),
        Arc::new(AlwaysSuccessPublisher),
        5,
    );

    let count = processor.process_batch().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_processor_partial_success_batch() {
    let mut store = MockTestStore::new();

    // 5件のメッセージ: 3件成功、2件失敗
    let messages: Vec<OutboxMessage> = vec![
        OutboxMessage::new("topic.v1", "ok-1", serde_json::json!({})),
        OutboxMessage::new("topic.v1", "fail-1", serde_json::json!({})),
        OutboxMessage::new("topic.v1", "ok-2", serde_json::json!({})),
        OutboxMessage::new("topic.v1", "fail-2", serde_json::json!({})),
        OutboxMessage::new("topic.v1", "ok-3", serde_json::json!({})),
    ];
    let messages_clone = messages.clone();

    store
        .expect_fetch_pending()
        .returning(move |_| Ok(messages_clone.clone()));

    // 各メッセージで processing + (delivered or failed) = 2回ずつ update が呼ばれる
    // 5メッセージ * 2回 = 10回
    store
        .expect_update()
        .times(10)
        .returning(|_| Ok(()));

    let publisher = SelectivePublisher::new(vec![1, 3]);
    let processor = OutboxProcessor::new(
        Arc::new(store),
        Arc::new(publisher),
        10,
    );

    let count = processor.process_batch().await.unwrap();
    // "ok-1", "ok-2", "ok-3" の3件が成功
    assert_eq!(count, 3);
}
