use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};
use k1s0_messaging::{EventEnvelope, EventProducer};

use crate::traits::{MessageHandler, PubSub};
use crate::PubSubError;

/// KafkaPubSub は Kafka ベースの PubSub 実装。
/// k1s0-messaging の EventProducer をラップする。
pub struct KafkaPubSub {
    name: String,
    producer: Arc<dyn EventProducer>,
    status: RwLock<ComponentStatus>,
    #[allow(dead_code)]
    subscriptions: RwLock<HashMap<String, Box<dyn MessageHandler>>>,
}

impl KafkaPubSub {
    pub fn new(name: impl Into<String>, producer: Arc<dyn EventProducer>) -> Self {
        Self {
            name: name.into(),
            producer,
            status: RwLock::new(ComponentStatus::Uninitialized),
            subscriptions: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Component for KafkaPubSub {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "pubsub"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "KafkaPubSub を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        info!(component = %self.name, "KafkaPubSub をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "kafka".to_string());
        meta
    }
}

#[async_trait]
impl PubSub for KafkaPubSub {
    async fn publish(
        &self,
        topic: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), PubSubError> {
        let envelope = EventEnvelope {
            topic: topic.to_string(),
            key: String::new(),
            payload: data.to_vec(),
            headers: Vec::new(),
            metadata: metadata.unwrap_or_default(),
        };
        self.producer
            .publish(envelope)
            .await
            .map_err(|e| PubSubError::Publish(e.to_string()))
    }

    async fn subscribe(
        &self,
        _topic: &str,
        _handler: Box<dyn MessageHandler>,
    ) -> Result<String, PubSubError> {
        // Kafka コンシューマーは別途 k1s0-messaging の EventConsumer を使用する想定。
        // このメソッドは将来の拡張用プレースホルダー。
        Err(PubSubError::Subscribe(
            "Kafka サブスクリプションは EventConsumer を直接使用してください".to_string(),
        ))
    }

    async fn unsubscribe(&self, _subscription_id: &str) -> Result<(), PubSubError> {
        Err(PubSubError::Subscribe(
            "Kafka サブスクリプションは EventConsumer を直接使用してください".to_string(),
        ))
    }
}
