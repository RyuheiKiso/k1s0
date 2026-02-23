use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::error::EventBusError;
use crate::event::Event;
use crate::handler::EventHandler;

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
                handler.handle(event.clone()).await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
}
