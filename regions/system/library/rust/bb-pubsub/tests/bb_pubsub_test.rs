// bb-pubsub の外部結合テスト。
// InMemoryPubSub の publish/subscribe ラウンドトリップを検証する。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use k1s0_bb_core::{Component, ComponentStatus};
use k1s0_bb_pubsub::{InMemoryPubSub, Message, MessageHandler, PubSub, PubSubError};

// テスト用のメッセージカウントハンドラー。受信回数をカウントする。
struct CountingHandler {
    count: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl MessageHandler for CountingHandler {
    async fn handle(&self, _message: Message) -> Result<(), PubSubError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

// テスト用のメッセージ記録ハンドラー。受信データを記録する。
struct RecordingHandler {
    received: Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

#[async_trait::async_trait]
impl MessageHandler for RecordingHandler {
    async fn handle(&self, message: Message) -> Result<(), PubSubError> {
        let mut received = self.received.lock().await;
        received.push(message.data);
        Ok(())
    }
}

// --- ライフサイクルテスト ---

// InMemoryPubSub の init/close ライフサイクルが正しく動作することを確認する。
#[tokio::test]
async fn test_pubsub_lifecycle() {
    let pubsub = InMemoryPubSub::new("test-pubsub");

    assert_eq!(pubsub.status().await, ComponentStatus::Uninitialized);

    pubsub.init().await.unwrap();
    assert_eq!(pubsub.status().await, ComponentStatus::Ready);

    pubsub.close().await.unwrap();
    assert_eq!(pubsub.status().await, ComponentStatus::Closed);
}

// InMemoryPubSub のコンポーネントメタデータが正しいことを確認する。
#[test]
fn test_pubsub_component_metadata() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    assert_eq!(pubsub.name(), "test-pubsub");
    assert_eq!(pubsub.component_type(), "pubsub");
    let meta = pubsub.metadata();
    assert_eq!(meta.get("backend").unwrap(), "memory");
}

// --- publish/subscribe ラウンドトリップテスト ---

// publish したメッセージがサブスクライブ済みハンドラーに配信されることを確認する。
#[tokio::test]
async fn test_publish_subscribe_roundtrip() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    pubsub.init().await.unwrap();

    let count = Arc::new(AtomicUsize::new(0));
    let handler = Box::new(CountingHandler {
        count: count.clone(),
    });

    let sub_id = pubsub.subscribe("orders", handler).await.unwrap();
    assert!(!sub_id.is_empty());

    // 同じトピックに publish するとハンドラーが呼ばれる
    pubsub.publish("orders", b"order-1", None).await.unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // 2回目の publish
    pubsub.publish("orders", b"order-2", None).await.unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

// 異なるトピックへの publish はハンドラーに配信されないことを確認する。
#[tokio::test]
async fn test_publish_different_topic() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    pubsub.init().await.unwrap();

    let count = Arc::new(AtomicUsize::new(0));
    let handler = Box::new(CountingHandler {
        count: count.clone(),
    });

    pubsub.subscribe("orders", handler).await.unwrap();
    pubsub.publish("payments", b"pay-1", None).await.unwrap();

    // 異なるトピックなのでカウントは 0 のまま
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

// メタデータ付きで publish できることを確認する。
#[tokio::test]
async fn test_publish_with_metadata() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    pubsub.init().await.unwrap();

    let count = Arc::new(AtomicUsize::new(0));
    let handler = Box::new(CountingHandler {
        count: count.clone(),
    });

    pubsub.subscribe("events", handler).await.unwrap();

    let mut meta = HashMap::new();
    meta.insert("trace_id".to_string(), "trace-123".to_string());
    pubsub
        .publish("events", b"event-data", Some(meta))
        .await
        .unwrap();

    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// publish されたメッセージのデータ内容が正しくハンドラーに渡されることを確認する。
#[tokio::test]
async fn test_publish_data_content() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    pubsub.init().await.unwrap();

    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let handler = Box::new(RecordingHandler {
        received: received.clone(),
    });

    pubsub.subscribe("data-topic", handler).await.unwrap();
    pubsub
        .publish("data-topic", b"hello world", None)
        .await
        .unwrap();

    let messages = received.lock().await;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0], b"hello world");
}

// --- unsubscribe テスト ---

// unsubscribe 後にメッセージが配信されないことを確認する。
#[tokio::test]
async fn test_unsubscribe_stops_delivery() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    pubsub.init().await.unwrap();

    let count = Arc::new(AtomicUsize::new(0));
    let handler = Box::new(CountingHandler {
        count: count.clone(),
    });

    let sub_id = pubsub.subscribe("topic", handler).await.unwrap();
    pubsub.unsubscribe(&sub_id).await.unwrap();

    pubsub.publish("topic", b"data", None).await.unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

// 存在しないサブスクリプション ID で unsubscribe するとエラーが返ることを確認する。
#[tokio::test]
async fn test_unsubscribe_not_found() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    let result = pubsub.unsubscribe("nonexistent-id").await;
    assert!(result.is_err());
}

// 同じトピックに複数のサブスクライバーが登録された場合に全てに配信されることを確認する。
#[tokio::test]
async fn test_multiple_subscribers_same_topic() {
    let pubsub = InMemoryPubSub::new("test-pubsub");
    pubsub.init().await.unwrap();

    let count1 = Arc::new(AtomicUsize::new(0));
    let count2 = Arc::new(AtomicUsize::new(0));

    let handler1 = Box::new(CountingHandler {
        count: count1.clone(),
    });
    let handler2 = Box::new(CountingHandler {
        count: count2.clone(),
    });

    pubsub.subscribe("shared-topic", handler1).await.unwrap();
    pubsub.subscribe("shared-topic", handler2).await.unwrap();

    pubsub
        .publish("shared-topic", b"broadcast", None)
        .await
        .unwrap();

    assert_eq!(count1.load(Ordering::SeqCst), 1);
    assert_eq!(count2.load(Ordering::SeqCst), 1);
}
