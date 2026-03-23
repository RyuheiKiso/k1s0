// タスクイベント Kafka プロデューサー。outbox_poller から呼ばれる。
// create-topics.sh のトピック名に合わせて、更新・キャンセルは status_changed トピックに統合する。
use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub struct TaskKafkaProducer {
    producer: FutureProducer,
    task_created_topic: String,
    // TaskUpdated と TaskCancelled を統合した status_changed トピック
    task_status_changed_topic: String,
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
            task_status_changed_topic: config.task_status_changed_topic.clone(),
        })
    }

    /// イベントを Kafka へ発行する。
    /// task_id をパーティションキーとして使用し、同一タスクのイベント順序を保証する。
    /// TaskCreated は task_created_topic へ、それ以外（TaskUpdated・TaskCancelled）は task_status_changed_topic へルーティングする。
    pub async fn publish(&self, event_type: &str, payload: &[u8], task_id: &str) -> anyhow::Result<()> {
        let topic = match event_type {
            "TaskCreated" => &self.task_created_topic,
            // TaskUpdated・TaskCancelled はともに status_changed トピックへ送信する
            _ => &self.task_status_changed_topic,
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
