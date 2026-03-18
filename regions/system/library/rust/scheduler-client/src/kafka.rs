use crate::error::SchedulerError;
use crate::job::JobCompletedEvent;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;
use rdkafka::Message;

/// Kafka subscriber for `JobCompletedEvent`.
pub struct KafkaJobCompletedSubscriber {
    consumer: StreamConsumer,
}

impl KafkaJobCompletedSubscriber {
    pub fn new(brokers: &str, consumer_group: &str, topic: &str) -> Result<Self, SchedulerError> {
        // at-least-once セマンティクスのため auto.commit を無効化する
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", consumer_group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| {
                SchedulerError::ServerError(format!("kafka create consumer failed: {e}"))
            })?;

        consumer
            .subscribe(&[topic])
            .map_err(|e| SchedulerError::ServerError(format!("kafka subscribe failed: {e}")))?;

        Ok(Self { consumer })
    }

    /// 次のジョブ完了イベントを取得し、成功後にオフセットをコミットする。
    pub async fn next_event(&self) -> Result<JobCompletedEvent, SchedulerError> {
        let message = self
            .consumer
            .recv()
            .await
            .map_err(|e| SchedulerError::ServerError(format!("kafka receive failed: {e}")))?;

        let payload = message
            .payload()
            .ok_or_else(|| SchedulerError::ServerError("kafka payload is empty".to_string()))?;

        let event = serde_json::from_slice::<JobCompletedEvent>(payload)
            .map_err(|e| SchedulerError::ServerError(format!("kafka payload decode failed: {e}")))?;

        // 処理成功後にオフセットを手動コミットする
        self.consumer
            .commit_message(&message, rdkafka::consumer::CommitMode::Async)
            .map_err(|e| SchedulerError::ServerError(format!("kafka commit failed: {e}")))?;

        Ok(event)
    }
}
