use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::EventBusConfig;
use crate::error::EventBusError;
use crate::event::Event;
use crate::handler::EventHandler;

/// サブスクリプションID。
type SubscriptionId = u64;

/// InMemoryEventBus はメモリ内のイベントバス（レガシーAPI、後方互換性のため維持）。
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        let event_type = handler.event_type().to_string();
        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub async fn unsubscribe(&self, event_type: &str) {
        let mut handlers = self.handlers.write().await;
        handlers.remove(event_type);
    }

    pub async fn publish(&self, event: Event) -> Result<(), EventBusError> {
        let handlers = self.handlers.read().await;
        if let Some(event_handlers) = handlers.get(&event.event_type) {
            for handler in event_handlers {
                handler
                    .handle(event.clone())
                    .await
                    .map_err(|e| EventBusError::HandlerFailed(e.to_string()))?;
            }
        }
        Ok(())
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// DDD パターンに対応した EventBus。
/// EventBusConfig で初期化し、EventSubscription で購読を管理する。
pub struct EventBus {
    config: EventBusConfig,
    handlers: Arc<RwLock<HashMap<String, Vec<(SubscriptionId, Arc<dyn EventHandler>)>>>>,
    next_id: Arc<RwLock<SubscriptionId>>,
}

impl EventBus {
    /// 設定を指定して新しい EventBus を生成する。
    pub fn new(config: EventBusConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(0)),
        }
    }

    /// 設定を取得する。
    pub fn config(&self) -> &EventBusConfig {
        &self.config
    }

    /// ハンドラーを購読し、EventSubscription を返す。
    /// EventSubscription が Drop されると自動的に購読解除される。
    pub async fn subscribe(&self, handler: Arc<dyn EventHandler>) -> EventSubscription {
        let event_type = handler.event_type().to_string();

        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push((id, handler));

        EventSubscription {
            id,
            event_type,
            handlers: self.handlers.clone(),
        }
    }

    /// イベントを発行する。
    pub async fn publish(&self, event: Event) -> Result<(), EventBusError> {
        let handlers = self.handlers.read().await;
        if let Some(event_handlers) = handlers.get(&event.event_type) {
            let timeout = self.config.get_handler_timeout();
            for (_, handler) in event_handlers {
                let handler = handler.clone();
                let event = event.clone();
                let result = tokio::time::timeout(timeout, handler.handle(event)).await;
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => return Err(EventBusError::HandlerFailed(e.to_string())),
                    Err(_) => {
                        return Err(EventBusError::HandlerFailed(
                            "handler timed out".to_string(),
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}

/// イベント購読を表す構造体。
/// Drop 時に自動的に購読解除される。
pub struct EventSubscription {
    id: SubscriptionId,
    event_type: String,
    handlers: Arc<RwLock<HashMap<String, Vec<(SubscriptionId, Arc<dyn EventHandler>)>>>>,
}

impl EventSubscription {
    /// 購読を手動で解除する。
    pub async fn unsubscribe(self) {
        self.remove_handler().await;
        // self は消費されるので Drop は呼ばれない（ただし安全のために Drop でもチェック）
    }

    async fn remove_handler(&self) {
        let mut handlers = self.handlers.write().await;
        if let Some(entries) = handlers.get_mut(&self.event_type) {
            entries.retain(|(id, _)| *id != self.id);
            if entries.is_empty() {
                handlers.remove(&self.event_type);
            }
        }
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        let handlers = self.handlers.clone();
        let event_type = self.event_type.clone();
        let id = self.id;

        // Drop 内では async を直接使えないので、spawn で非同期処理を行う
        tokio::spawn(async move {
            let mut handlers = handlers.write().await;
            if let Some(entries) = handlers.get_mut(&event_type) {
                entries.retain(|(sub_id, _)| *sub_id != id);
                if entries.is_empty() {
                    handlers.remove(&event_type);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EventBusConfig;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    struct TestHandler {
        event_type: String,
        call_count: Arc<AtomicUsize>,
    }

    impl TestHandler {
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

    #[async_trait::async_trait]
    impl EventHandler for TestHandler {
        fn event_type(&self) -> &str {
            &self.event_type
        }

        async fn handle(&self, _event: Event) -> Result<(), EventBusError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    // --- InMemoryEventBus (レガシー) テスト ---

    #[tokio::test]
    async fn test_subscribe_and_publish() {
        let bus = InMemoryEventBus::new();
        let (handler, count) = TestHandler::new("user.created");
        bus.subscribe(Arc::new(handler)).await;

        let event = Event::new("user.created".to_string(), json!({"user": "test"}));
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_handler_not_called_for_different_event_type() {
        let bus = InMemoryEventBus::new();
        let (handler, count) = TestHandler::new("user.created");
        bus.subscribe(Arc::new(handler)).await;

        let event = Event::new("order.placed".to_string(), json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_multiple_handlers_for_same_event() {
        let bus = InMemoryEventBus::new();
        let (handler1, count1) = TestHandler::new("user.created");
        let (handler2, count2) = TestHandler::new("user.created");
        bus.subscribe(Arc::new(handler1)).await;
        bus.subscribe(Arc::new(handler2)).await;

        let event = Event::new("user.created".to_string(), json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_all_handlers_for_event_type() {
        let bus = InMemoryEventBus::new();
        let (handler, count) = TestHandler::new("user.created");
        bus.subscribe(Arc::new(handler)).await;
        bus.unsubscribe("user.created").await;

        let event = Event::new("user.created".to_string(), json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_published_event_data_is_correct() {
        let bus = InMemoryEventBus::new();
        let received = Arc::new(RwLock::new(None::<Event>));
        let received_clone = received.clone();

        struct CapturingHandler {
            event_type: String,
            received: Arc<RwLock<Option<Event>>>,
        }

        #[async_trait::async_trait]
        impl EventHandler for CapturingHandler {
            fn event_type(&self) -> &str {
                &self.event_type
            }
            async fn handle(&self, event: Event) -> Result<(), EventBusError> {
                *self.received.write().await = Some(event);
                Ok(())
            }
        }

        let handler = CapturingHandler {
            event_type: "test.event".to_string(),
            received: received_clone,
        };
        bus.subscribe(Arc::new(handler)).await;

        let payload = json!({"key": "value"});
        let event = Event::new("test.event".to_string(), payload.clone());
        let event_id = event.id;
        bus.publish(event).await.unwrap();

        let captured = received.read().await;
        let captured = captured.as_ref().unwrap();
        assert_eq!(captured.id, event_id);
        assert_eq!(captured.event_type, "test.event");
        assert_eq!(captured.payload, payload);
    }

    // --- EventBus (DDD パターン) テスト ---

    #[tokio::test]
    async fn test_eventbus_new_with_config() {
        let config = EventBusConfig::new()
            .buffer_size(2048)
            .handler_timeout(Duration::from_secs(60));
        let bus = EventBus::new(config);
        assert_eq!(bus.config().get_buffer_size(), 2048);
        assert_eq!(bus.config().get_handler_timeout(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_eventbus_subscribe_and_publish() {
        let bus = EventBus::new(EventBusConfig::new());
        let (handler, count) = TestHandler::new("order.created");
        let _sub = bus.subscribe(Arc::new(handler)).await;

        let event = Event::new("order.created".to_string(), json!({"order_id": "123"}));
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_eventbus_subscription_unsubscribe() {
        let bus = EventBus::new(EventBusConfig::new());
        let (handler, count) = TestHandler::new("order.created");
        let sub = bus.subscribe(Arc::new(handler)).await;

        // 手動で購読解除
        sub.unsubscribe().await;

        let event = Event::new("order.created".to_string(), json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_eventbus_subscription_drop_auto_unsubscribe() {
        let bus = EventBus::new(EventBusConfig::new());
        let (handler, count) = TestHandler::new("order.created");

        {
            let _sub = bus.subscribe(Arc::new(handler)).await;
            // _sub がスコープを抜けると Drop で自動解除
        }

        // Drop 内の tokio::spawn が完了するのを少し待つ
        tokio::time::sleep(Duration::from_millis(50)).await;

        let event = Event::new("order.created".to_string(), json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_eventbus_handler_timeout() {
        let config = EventBusConfig::new().handler_timeout(Duration::from_millis(50));
        let bus = EventBus::new(config);

        struct SlowHandler;

        #[async_trait::async_trait]
        impl EventHandler for SlowHandler {
            fn event_type(&self) -> &str {
                "slow.event"
            }
            async fn handle(&self, _event: Event) -> Result<(), EventBusError> {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(())
            }
        }

        let _sub = bus.subscribe(Arc::new(SlowHandler)).await;

        let event = Event::new("slow.event".to_string(), json!({}));
        let result = bus.publish(event).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            EventBusError::HandlerFailed(msg) => {
                assert!(msg.contains("timed out"));
            }
            other => panic!("expected HandlerFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_eventbus_multiple_subscriptions() {
        let bus = EventBus::new(EventBusConfig::new());
        let (handler1, count1) = TestHandler::new("evt");
        let (handler2, count2) = TestHandler::new("evt");

        let _sub1 = bus.subscribe(Arc::new(handler1)).await;
        let _sub2 = bus.subscribe(Arc::new(handler2)).await;

        let event = Event::new("evt".to_string(), json!({}));
        bus.publish(event).await.unwrap();

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_eventbus_no_subscribers() {
        let bus = EventBus::new(EventBusConfig::new());
        let event = Event::new("unknown".to_string(), json!({}));
        let result = bus.publish(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_eventbus_handler_error() {
        let bus = EventBus::new(EventBusConfig::new());

        struct FailingHandler;

        #[async_trait::async_trait]
        impl EventHandler for FailingHandler {
            fn event_type(&self) -> &str {
                "fail.event"
            }
            async fn handle(&self, _event: Event) -> Result<(), EventBusError> {
                Err(EventBusError::HandlerFailed("test error".to_string()))
            }
        }

        let _sub = bus.subscribe(Arc::new(FailingHandler)).await;

        let event = Event::new("fail.event".to_string(), json!({}));
        let result = bus.publish(event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_domain_event_impl() {
        use crate::event::DomainEvent;

        let event = Event::with_aggregate_id(
            "user.created".to_string(),
            "user-123".to_string(),
            json!({}),
        );

        assert_eq!(event.event_type(), "user.created");
        assert_eq!(event.aggregate_id(), "user-123");
        assert!(event.occurred_at() <= chrono::Utc::now());
    }
}
