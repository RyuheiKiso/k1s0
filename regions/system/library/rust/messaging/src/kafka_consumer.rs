//! KafkaEventConsumer: rdkafka を使用した EventConsumer 実装。
//! feature = "kafka" で有効化される。

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::TopicPartitionList;

use crate::consumer::{ConsumedMessage, EventConsumer};
use crate::error::MessagingError;

/// KafkaEventConsumer は rdkafka の StreamConsumer を使った実装。
pub struct KafkaEventConsumer {
    consumer: StreamConsumer,
}

impl KafkaEventConsumer {
    /// 新しい KafkaEventConsumer を生成する。
    pub fn new(
        brokers: &str,
        group_id: &str,
        topics: &[&str],
    ) -> Result<Self, MessagingError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "false")
            .set("session.timeout.ms", "30000")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| MessagingError::ConnectionError(e.to_string()))?;

        consumer
            .subscribe(topics)
            .map_err(|e| MessagingError::ConnectionError(e.to_string()))?;

        Ok(Self { consumer })
    }
}

#[async_trait]
impl EventConsumer for KafkaEventConsumer {
    async fn receive(&self) -> Result<ConsumedMessage, MessagingError> {
        let msg = self
            .consumer
            .recv()
            .await
            .map_err(|e| MessagingError::ConsumeError(e.to_string()))?;

        Ok(ConsumedMessage {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            key: msg.key().map(|k| k.to_vec()),
            payload: msg.payload().unwrap_or_default().to_vec(),
        })
    }

    async fn commit(&self, msg: &ConsumedMessage) -> Result<(), MessagingError> {
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(
            &msg.topic,
            msg.partition,
            rdkafka::Offset::Offset(msg.offset + 1),
        )
        .map_err(|e| MessagingError::CommitError(e.to_string()))?;

        self.consumer
            .commit(&tpl, rdkafka::consumer::CommitMode::Sync)
            .map_err(|e| MessagingError::CommitError(e.to_string()))?;

        Ok(())
    }
}
