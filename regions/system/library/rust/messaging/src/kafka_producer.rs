//! KafkaEventProducer: rdkafka を使用した EventProducer 実装。
//! feature = "kafka" で有効化される。

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::error::MessagingError;
use crate::event::EventEnvelope;
use crate::producer::EventProducer;

/// KafkaEventProducer は rdkafka の FutureProducer を使った実装。
pub struct KafkaEventProducer {
    producer: FutureProducer,
}

impl KafkaEventProducer {
    /// ブローカーリストから KafkaEventProducer を生成する。
    pub fn new(brokers: &str) -> Result<Self, MessagingError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "30000")
            .create()
            .map_err(|e| MessagingError::ConnectionError(e.to_string()))?;
        Ok(Self { producer })
    }

    /// MessagingConfig から KafkaEventProducer を生成する。
    pub fn with_config(config: &crate::config::MessagingConfig) -> Result<Self, MessagingError> {
        let brokers = config.brokers_string();
        Self::new(&brokers)
    }
}

#[async_trait]
impl EventProducer for KafkaEventProducer {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), MessagingError> {
        let record = FutureRecord::to(&envelope.topic)
            .key(&envelope.key)
            .payload(&envelope.payload as &[u8]);

        self.producer
            .send(record, Duration::from_secs(10))
            .await
            .map_err(|(err, _)| MessagingError::PublishError(err.to_string()))?;

        Ok(())
    }

    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<(), MessagingError> {
        for envelope in envelopes {
            self.publish(envelope).await?;
        }
        Ok(())
    }
}
