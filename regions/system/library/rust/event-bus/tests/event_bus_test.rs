#![allow(clippy::unwrap_used)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::RwLock;

use k1s0_event_bus::{
    DomainEvent, Event, EventBus, EventBusConfig, EventBusError, EventHandler, InMemoryEventBus,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct CountingHandler {
    event_type: String,
    call_count: Arc<AtomicUsize>,
}

impl CountingHandler {
    fn new(event_type: &str) -> (Self, Arc<AtomicUsize>) {
        let count = Arc::new(AtomicUsize::new(0));
        (
            Self {
                event_type: event_type.to_string(),
                call_count: count.clone(),
            },
            count,
        )
    }
}

#[async_trait]
impl EventHandler for CountingHandler {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    async fn handle(&self, _event: Arc<Event>) -> Result<(), EventBusError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct CapturingHandler {
    event_type: String,
    captured: Arc<RwLock<Vec<Event>>>,
}

impl CapturingHandler {
    fn new(event_type: &str) -> (Self, Arc<RwLock<Vec<Event>>>) {
        let captured = Arc::new(RwLock::new(Vec::new()));
        (
            Self {
                event_type: event_type.to_string(),
                captured: captured.clone(),
            },
            captured,
        )
    }
}

#[async_trait]
impl EventHandler for CapturingHandler {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    async fn handle(&self, event: Arc<Event>) -> Result<(), EventBusError> {
        // Arc の参照先をクローンして格納する（テスト検証用）
        self.captured.write().await.push((*event).clone());
        Ok(())
    }
}

struct FailingHandler {
    event_type: String,
    message: String,
}

impl FailingHandler {
    fn new(event_type: &str, message: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            message: message.to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for FailingHandler {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    async fn handle(&self, _event: Arc<Event>) -> Result<(), EventBusError> {
        Err(EventBusError::HandlerFailed(self.message.clone()))
    }
}

struct SlowHandler {
    event_type: String,
    delay: Duration,
}

#[async_trait]
impl EventHandler for SlowHandler {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    async fn handle(&self, _event: Arc<Event>) -> Result<(), EventBusError> {
        tokio::time::sleep(self.delay).await;
        Ok(())
    }
}

// ===========================================================================
// InMemoryEventBus tests (legacy API)
// ===========================================================================

// レガシー API でイベントを購読・発行する基本フローが動作することを確認する。
#[tokio::test]
async fn legacy_subscribe_and_publish_basic_flow() {
    let bus = InMemoryEventBus::new();
    let (handler, count) = CountingHandler::new("user.created");
    bus.subscribe(Arc::new(handler)).await;

    let event = Event::new("user.created".to_string(), json!({"name": "alice"}));
    bus.publish(event).await.unwrap();

    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// レガシー API でハンドラーが発行イベントのデータを正しく受け取ることを確認する。
#[tokio::test]
async fn legacy_handler_receives_correct_event_data() {
    let bus = InMemoryEventBus::new();
    let (handler, captured) = CapturingHandler::new("order.placed");
    bus.subscribe(Arc::new(handler)).await;

    let payload = json!({"order_id": "ORD-001", "amount": 99.99});
    let event = Event::new("order.placed".to_string(), payload.clone());
    let expected_id = event.id;
    bus.publish(event).await.unwrap();

    let events = captured.read().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, expected_id);
    assert_eq!(events[0].event_type, "order.placed");
    assert_eq!(events[0].payload, payload);
}

// レガシー API で複数の購読者が同一イベントを受け取ることを確認する。
#[tokio::test]
async fn legacy_multiple_subscribers_receive_same_event() {
    let bus = InMemoryEventBus::new();
    let (h1, count1) = CountingHandler::new("item.added");
    let (h2, count2) = CountingHandler::new("item.added");
    let (h3, count3) = CountingHandler::new("item.added");
    bus.subscribe(Arc::new(h1)).await;
    bus.subscribe(Arc::new(h2)).await;
    bus.subscribe(Arc::new(h3)).await;

    let event = Event::new("item.added".to_string(), json!({}));
    bus.publish(event).await.unwrap();

    assert_eq!(count1.load(Ordering::SeqCst), 1);
    assert_eq!(count2.load(Ordering::SeqCst), 1);
    assert_eq!(count3.load(Ordering::SeqCst), 1);
}

// レガシー API で異なるイベントタイプを発行してもハンドラーが呼ばれないことを確認する。
#[tokio::test]
async fn legacy_handler_not_called_for_different_event_type() {
    let bus = InMemoryEventBus::new();
    let (handler, count) = CountingHandler::new("user.created");
    bus.subscribe(Arc::new(handler)).await;

    let event = Event::new("order.placed".to_string(), json!({}));
    bus.publish(event).await.unwrap();

    assert_eq!(count.load(Ordering::SeqCst), 0);
}

// レガシー API で購読解除後にイベントが配信されなくなることを確認する。
#[tokio::test]
async fn legacy_unsubscribe_stops_delivery() {
    let bus = InMemoryEventBus::new();
    let (handler, count) = CountingHandler::new("evt.a");
    bus.subscribe(Arc::new(handler)).await;

    // First publish should deliver
    bus.publish(Event::new("evt.a".to_string(), json!({})))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // Unsubscribe
    bus.unsubscribe("evt.a").await;

    // Second publish should not deliver
    bus.publish(Event::new("evt.a".to_string(), json!({})))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// レガシー API で購読者がいない場合にイベント発行が正常に成功することを確認する。
#[tokio::test]
async fn legacy_publish_with_no_subscribers_succeeds() {
    let bus = InMemoryEventBus::new();
    let result = bus
        .publish(Event::new("nobody.listens".to_string(), json!({})))
        .await;
    assert!(result.is_ok());
}

// レガシー API でハンドラーのエラーが発行元に伝播することを確認する。
#[tokio::test]
async fn legacy_handler_error_propagates() {
    let bus = InMemoryEventBus::new();
    let handler = FailingHandler::new("fail.event", "boom");
    bus.subscribe(Arc::new(handler)).await;

    let result = bus
        .publish(Event::new("fail.event".to_string(), json!({})))
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EventBusError::HandlerFailed(msg) => assert!(msg.contains("boom")),
        other => panic!("expected HandlerFailed, got {:?}", other),
    }
}

// レガシー API で複数回イベントを発行するたびにハンドラーが呼ばれることを確認する。
#[tokio::test]
async fn legacy_multiple_publishes_increment_count() {
    let bus = InMemoryEventBus::new();
    let (handler, count) = CountingHandler::new("tick");
    bus.subscribe(Arc::new(handler)).await;

    for _ in 0..5 {
        bus.publish(Event::new("tick".to_string(), json!({})))
            .await
            .unwrap();
    }

    assert_eq!(count.load(Ordering::SeqCst), 5);
}

// レガシー API の Default コンストラクタで生成したバスが正常に動作することを確認する。
#[tokio::test]
async fn legacy_default_constructor() {
    let bus = InMemoryEventBus::default();
    let (handler, count) = CountingHandler::new("x");
    bus.subscribe(Arc::new(handler)).await;

    bus.publish(Event::new("x".to_string(), json!({})))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// ===========================================================================
// EventBus (DDD pattern) tests
// ===========================================================================

// EventBusConfig のデフォルト値が正しいことを確認する。
#[tokio::test]
async fn eventbus_config_defaults() {
    let config = EventBusConfig::new();
    assert_eq!(config.get_buffer_size(), 1024);
    assert_eq!(config.get_handler_timeout(), Duration::from_secs(30));
}

// EventBusConfig のビルダーパターンで設定を変更できることを確認する。
#[tokio::test]
async fn eventbus_config_builder() {
    let config = EventBusConfig::new()
        .buffer_size(4096)
        .handler_timeout(Duration::from_secs(120));
    assert_eq!(config.get_buffer_size(), 4096);
    assert_eq!(config.get_handler_timeout(), Duration::from_secs(120));
}

// EventBusConfig の Default トレイトが正しいデフォルト値を返すことを確認する。
#[tokio::test]
async fn eventbus_config_default_trait() {
    let config = EventBusConfig::default();
    assert_eq!(config.get_buffer_size(), 1024);
    assert_eq!(config.get_handler_timeout(), Duration::from_secs(30));
}

// EventBus 生成時に指定した設定が config() メソッドで取得できることを確認する。
#[tokio::test]
async fn eventbus_new_exposes_config() {
    let config = EventBusConfig::new().buffer_size(512);
    let bus = EventBus::new(config);
    assert_eq!(bus.config().get_buffer_size(), 512);
}

// EventBus でイベントを購読・発行するとハンドラーが呼ばれることを確認する。
#[tokio::test]
async fn eventbus_subscribe_and_publish() {
    let bus = EventBus::new(EventBusConfig::new());
    let (handler, count) = CountingHandler::new("order.created");
    let _sub = bus.subscribe(Arc::new(handler)).await;

    bus.publish(Event::new(
        "order.created".to_string(),
        json!({"order_id": "123"}),
    ))
    .await
    .unwrap();

    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// EventBus でハンドラーが発行イベントのペイロードを正しく受け取ることを確認する。
#[tokio::test]
async fn eventbus_handler_receives_event_data() {
    let bus = EventBus::new(EventBusConfig::new());
    let (handler, captured) = CapturingHandler::new("data.event");
    let _sub = bus.subscribe(Arc::new(handler)).await;

    let payload = json!({"key": "value", "nested": {"a": 1}});
    let event = Event::new("data.event".to_string(), payload.clone());
    let event_id = event.id;
    bus.publish(event).await.unwrap();

    let events = captured.read().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event_id);
    assert_eq!(events[0].payload, payload);
}

// EventBus で複数の購読者が同一イベントを受け取ることを確認する。
#[tokio::test]
async fn eventbus_multiple_subscribers_receive_same_event() {
    let bus = EventBus::new(EventBusConfig::new());
    let (h1, c1) = CountingHandler::new("shared");
    let (h2, c2) = CountingHandler::new("shared");
    let _s1 = bus.subscribe(Arc::new(h1)).await;
    let _s2 = bus.subscribe(Arc::new(h2)).await;

    bus.publish(Event::new("shared".to_string(), json!({})))
        .await
        .unwrap();

    assert_eq!(c1.load(Ordering::SeqCst), 1);
    assert_eq!(c2.load(Ordering::SeqCst), 1);
}

// EventBus でイベントタイプによるフィルタリングが正しく機能することを確認する。
#[tokio::test]
async fn eventbus_event_type_filtering() {
    let bus = EventBus::new(EventBusConfig::new());
    let (h_a, c_a) = CountingHandler::new("type.a");
    let (h_b, c_b) = CountingHandler::new("type.b");
    let _s_a = bus.subscribe(Arc::new(h_a)).await;
    let _s_b = bus.subscribe(Arc::new(h_b)).await;

    bus.publish(Event::new("type.a".to_string(), json!({})))
        .await
        .unwrap();

    assert_eq!(c_a.load(Ordering::SeqCst), 1);
    assert_eq!(c_b.load(Ordering::SeqCst), 0);
}

// EventBus で購読解除後にイベントが配信されなくなることを確認する。
#[tokio::test]
async fn eventbus_unsubscribe_stops_delivery() {
    let bus = EventBus::new(EventBusConfig::new());
    let (handler, count) = CountingHandler::new("unsub.test");
    let sub = bus.subscribe(Arc::new(handler)).await;

    // Publish before unsubscribe
    bus.publish(Event::new("unsub.test".to_string(), json!({})))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // Unsubscribe
    sub.unsubscribe().await;

    // Publish after unsubscribe
    bus.publish(Event::new("unsub.test".to_string(), json!({})))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// 一つの購読を解除しても他の購読者はイベントを受け取り続けることを確認する。
#[tokio::test]
async fn eventbus_unsubscribe_one_keeps_others() {
    let bus = EventBus::new(EventBusConfig::new());
    let (h1, c1) = CountingHandler::new("partial");
    let (h2, c2) = CountingHandler::new("partial");
    let sub1 = bus.subscribe(Arc::new(h1)).await;
    let _sub2 = bus.subscribe(Arc::new(h2)).await;

    // Unsubscribe only h1
    sub1.unsubscribe().await;

    bus.publish(Event::new("partial".to_string(), json!({})))
        .await
        .unwrap();

    assert_eq!(c1.load(Ordering::SeqCst), 0);
    assert_eq!(c2.load(Ordering::SeqCst), 1);
}

// EventSubscription が Drop されるとキャンセルフラグが立ち、次回 publish() でスキップされることを確認する。
// AtomicBool 方式により sleep 不要で即時確認できる（SH-3 監査対応）。
#[tokio::test]
async fn eventbus_subscription_drop_auto_unsubscribes() {
    let bus = EventBus::new(EventBusConfig::new());
    let (handler, count) = CountingHandler::new("drop.test");

    {
        let _sub = bus.subscribe(Arc::new(handler)).await;
        // _sub drops here, setting the AtomicBool cancellation flag
    }

    // AtomicBool 方式では Drop 直後にフラグが立つため sleep 不要
    bus.publish(Event::new("drop.test".to_string(), json!({})))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

// ハンドラーがタイムアウト設定を超えた場合に HandlerFailed エラーが返ることを確認する。
#[tokio::test]
async fn eventbus_handler_timeout() {
    let config = EventBusConfig::new().handler_timeout(Duration::from_millis(50));
    let bus = EventBus::new(config);

    let handler = SlowHandler {
        event_type: "slow.event".to_string(),
        delay: Duration::from_secs(2),
    };
    let _sub = bus.subscribe(Arc::new(handler)).await;

    let result = bus
        .publish(Event::new("slow.event".to_string(), json!({})))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        EventBusError::HandlerFailed(msg) => assert!(msg.contains("timed out")),
        other => panic!("expected HandlerFailed with timeout, got {:?}", other),
    }
}

// タイムアウト内に完了するハンドラーの発行が正常に成功することを確認する。
#[tokio::test]
async fn eventbus_handler_within_timeout_succeeds() {
    let config = EventBusConfig::new().handler_timeout(Duration::from_secs(5));
    let bus = EventBus::new(config);

    let handler = SlowHandler {
        event_type: "fast.event".to_string(),
        delay: Duration::from_millis(10),
    };
    let _sub = bus.subscribe(Arc::new(handler)).await;

    let result = bus
        .publish(Event::new("fast.event".to_string(), json!({})))
        .await;
    assert!(result.is_ok());
}

// EventBus でハンドラーのエラーが発行元に伝播することを確認する。
#[tokio::test]
async fn eventbus_handler_error_propagates() {
    let bus = EventBus::new(EventBusConfig::new());
    let handler = FailingHandler::new("err.event", "handler exploded");
    let _sub = bus.subscribe(Arc::new(handler)).await;

    let result = bus
        .publish(Event::new("err.event".to_string(), json!({})))
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EventBusError::HandlerFailed(msg) => assert!(msg.contains("handler exploded")),
        other => panic!("expected HandlerFailed, got {:?}", other),
    }
}

// EventBus で購読者がいない場合にイベント発行が正常に成功することを確認する。
#[tokio::test]
async fn eventbus_publish_no_subscribers_succeeds() {
    let bus = EventBus::new(EventBusConfig::new());
    let result = bus
        .publish(Event::new("ghost".to_string(), json!({})))
        .await;
    assert!(result.is_ok());
}

// EventBus で複数の購読者と並行発行が正しく動作することを確認する。
#[tokio::test]
async fn eventbus_concurrent_publish_subscribe() {
    let bus = Arc::new(EventBus::new(EventBusConfig::new()));
    let count = Arc::new(AtomicUsize::new(0));

    // Subscribe 10 handlers
    let mut subs = Vec::new();
    for _ in 0..10 {
        let count_clone = count.clone();

        struct ConcurrentHandler {
            count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl EventHandler for ConcurrentHandler {
            fn event_type(&self) -> &str {
                "concurrent"
            }
            async fn handle(&self, _event: Arc<Event>) -> Result<(), EventBusError> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let sub = bus
            .subscribe(Arc::new(ConcurrentHandler { count: count_clone }))
            .await;
        subs.push(sub);
    }

    // Publish 5 events concurrently
    let mut handles = Vec::new();
    for i in 0..5 {
        let bus = bus.clone();
        handles.push(tokio::spawn(async move {
            bus.publish(Event::new("concurrent".to_string(), json!({"i": i})))
                .await
                .unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // 10 handlers x 5 events = 50
    assert_eq!(count.load(Ordering::SeqCst), 50);
}

// ===========================================================================
// Event / DomainEvent tests
// ===========================================================================

// Event::new で生成したイベントが毎回一意の ID を持つことを確認する。
#[tokio::test]
async fn event_new_has_unique_id() {
    let e1 = Event::new("test".to_string(), json!({}));
    let e2 = Event::new("test".to_string(), json!({}));
    assert_ne!(e1.id, e2.id);
}

// Event::new で生成したイベントの aggregate_id が空文字であることを確認する。
#[tokio::test]
async fn event_new_has_empty_aggregate_id() {
    let event = Event::new("test".to_string(), json!({}));
    assert_eq!(event.aggregate_id, "");
}

// Event::with_aggregate_id でイベントタイプと集約 ID が正しく設定されることを確認する。
#[tokio::test]
async fn event_with_aggregate_id() {
    let event =
        Event::with_aggregate_id("user.created".to_string(), "user-42".to_string(), json!({}));
    assert_eq!(event.event_type, "user.created");
    assert_eq!(event.aggregate_id, "user-42");
}

// Event が DomainEvent トレイトのメソッドを正しく実装していることを確認する。
#[tokio::test]
async fn event_domain_event_trait() {
    let event = Event::with_aggregate_id(
        "order.shipped".to_string(),
        "order-99".to_string(),
        json!({"tracking": "ABC123"}),
    );

    assert_eq!(event.event_type(), "order.shipped");
    assert_eq!(event.aggregate_id(), "order-99");
    assert!(event.occurred_at() <= chrono::Utc::now());
}

// Event 生成時のタイムスタンプが現在時刻に近いことを確認する。
#[tokio::test]
async fn event_timestamp_is_recent() {
    let before = chrono::Utc::now();
    let event = Event::new("ts.test".to_string(), json!({}));
    let after = chrono::Utc::now();

    assert!(event.timestamp >= before);
    assert!(event.timestamp <= after);
}

// Event のクローンが元のイベントと同一の内容を持つことを確認する。
#[tokio::test]
async fn event_clone() {
    let event = Event::new("clone.test".to_string(), json!({"a": 1}));
    let cloned = event.clone();
    assert_eq!(event.id, cloned.id);
    assert_eq!(event.event_type, cloned.event_type);
    assert_eq!(event.payload, cloned.payload);
}

// ===========================================================================
// EventBusError tests
// ===========================================================================

// EventBusError の各バリアントの Display メッセージが正しいことを確認する。
#[test]
fn error_display_messages() {
    let err = EventBusError::PublishFailed("timeout".to_string());
    assert_eq!(format!("{}", err), "publish failed: timeout");

    let err = EventBusError::HandlerFailed("crash".to_string());
    assert_eq!(format!("{}", err), "handler failed: crash");

    let err = EventBusError::ChannelClosed;
    assert_eq!(format!("{}", err), "channel closed");
}

// EventBusError の Debug フォーマットがバリアント名を含むことを確認する。
#[test]
fn error_debug_format() {
    let err = EventBusError::ChannelClosed;
    let debug = format!("{:?}", err);
    assert!(debug.contains("ChannelClosed"));
}
