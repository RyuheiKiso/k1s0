// タスクイベント Kafka プロデューサー。outbox_poller から呼ばれる。
use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub struct TaskKafkaProducer {
    producer: FutureProducer,
    task_created_topic: String,
    task_updated_topic: String,
    task_cancelled_topic: String,
}

impl TaskKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");
        let producer = client_config.create()?;
        Ok(Self {
            producer,
            task_created_topic: config.task_created_topic.clone(),
            task_updated_topic: config.task_updated_topic.clone(),
            task_cancelled_topic: config.task_cancelled_topic.clone(),
        })
    }

    /// イベントを Kafka へ発行する。
    /// task_id をパーティションキーとして使用し、同一タスクのイベント順序を保証する。
    pub async fn publish(&self, event_type: &str, payload: &[u8], task_id: &str) -> anyhow::Result<()> {
        let topic = match event_type {
            "TaskCreated" => &self.task_created_topic,
            "TaskCancelled" => &self.task_cancelled_topic,
            _ => &self.task_updated_topic,
        };
        // task_id をパーティションキーとして使用し、同一タスクのイベント順序を保証する
        let record = FutureRecord::to(topic).payload(payload).key(task_id);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("kafka send error: {}", e))?;
        Ok(())
    }
}
