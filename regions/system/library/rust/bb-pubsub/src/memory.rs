use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};

use crate::traits::{Message, MessageHandler, PubSub};
use crate::PubSubError;

struct Subscription {
    topic: String,
    handler: Box<dyn MessageHandler>,
}

/// InMemoryPubSub はテスト・開発用のインメモリ PubSub 実装。
pub struct InMemoryPubSub {
    name: String,
    status: RwLock<ComponentStatus>,
    subscriptions: RwLock<HashMap<String, Subscription>>,
}

impl InMemoryPubSub {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: RwLock::new(ComponentStatus::Uninitialized),
            subscriptions: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Component for InMemoryPubSub {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "pubsub"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "InMemoryPubSub を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        let mut subs = self.subscriptions.write().await;
        subs.clear();
        info!(component = %self.name, "InMemoryPubSub をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "memory".to_string());
        meta
    }
}

#[async_trait]
impl PubSub for InMemoryPubSub {
    async fn publish(
        &self,
        topic: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), PubSubError> {
        let message = Message {
            topic: topic.to_string(),
            data: data.to_vec(),
            metadata: metadata.unwrap_or_default(),
            id: Uuid::new_v4().to_string(),
        };

        let subscriptions = self.subscriptions.read().await;
        for sub in subscriptions.values() {
            if sub.topic == topic {
                sub.handler.handle(message.clone()).await?;
            }
        }
        Ok(())
    }

    async fn subscribe(
        &self,
        topic: &str,
        handler: Box<dyn MessageHandler>,
    ) -> Result<String, PubSubError> {
        let subscription_id = Uuid::new_v4().to_string();
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(
            subscription_id.clone(),
            Subscription {
                topic: topic.to_string(),
                handler,
            },
        );
        info!(
            component = %self.name,
            topic = %topic,
            subscription_id = %subscription_id,
            "サブスクリプションを追加しました"
        );
        Ok(subscription_id)
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), PubSubError> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions
            .remove(subscription_id)
            .ok_or_else(|| PubSubError::SubscriptionNotFound(subscription_id.to_string()))?;
        info!(
            component = %self.name,
            subscription_id = %subscription_id,
            "サブスクリプションを解除しました"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountingHandler {
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl MessageHandler for CountingHandler {
        async fn handle(&self, _message: Message) -> Result<(), PubSubError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    // InMemoryPubSub の初期化後にステータスが Ready になることを確認する。
    #[tokio::test]
    async fn test_init_and_status() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        assert_eq!(pubsub.status().await, ComponentStatus::Uninitialized);
        pubsub.init().await.unwrap();
        assert_eq!(pubsub.status().await, ComponentStatus::Ready);
    }

    // クローズ後にステータスが Closed になることを確認する。
    #[tokio::test]
    async fn test_close() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        pubsub.init().await.unwrap();
        pubsub.close().await.unwrap();
        assert_eq!(pubsub.status().await, ComponentStatus::Closed);
    }

    // パブリッシュ時にサブスクライブ済みトピックのハンドラーが呼び出されることを確認する。
    #[tokio::test]
    async fn test_publish_subscribe() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        pubsub.init().await.unwrap();

        let count = Arc::new(AtomicUsize::new(0));
        let handler = Box::new(CountingHandler {
            count: count.clone(),
        });

        let sub_id = pubsub.subscribe("test-topic", handler).await.unwrap();
        assert!(!sub_id.is_empty());

        pubsub
            .publish("test-topic", b"hello", None)
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);

        pubsub
            .publish("other-topic", b"world", None)
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    // サブスクリプション解除後にメッセージが配信されないことを確認する。
    #[tokio::test]
    async fn test_unsubscribe() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        pubsub.init().await.unwrap();

        let count = Arc::new(AtomicUsize::new(0));
        let handler = Box::new(CountingHandler {
            count: count.clone(),
        });

        let sub_id = pubsub.subscribe("test-topic", handler).await.unwrap();
        pubsub.unsubscribe(&sub_id).await.unwrap();

        pubsub
            .publish("test-topic", b"hello", None)
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    // 存在しないサブスクリプション ID を解除しようとするとエラーになることを確認する。
    #[tokio::test]
    async fn test_unsubscribe_not_found() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        let result = pubsub.unsubscribe("nonexistent").await;
        assert!(result.is_err());
    }

    // メタデータにバックエンドが "memory" として設定されていることを確認する。
    #[tokio::test]
    async fn test_metadata() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        let meta = pubsub.metadata();
        assert_eq!(meta.get("backend").unwrap(), "memory");
    }

    // コンポーネントタイプが "pubsub" であることを確認する。
    #[tokio::test]
    async fn test_component_type() {
        let pubsub = InMemoryPubSub::new("test-pubsub");
        assert_eq!(pubsub.component_type(), "pubsub");
    }
}
