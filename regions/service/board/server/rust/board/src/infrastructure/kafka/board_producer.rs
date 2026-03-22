// ボードイベント Kafka プロデューサー。
use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub struct BoardKafkaProducer {
    producer: FutureProducer,
    board_column_updated_topic: String,
}

impl BoardKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("acks", "all");
        client_config.set("enable.idempotence", "true");
        let producer = client_config.create()?;
        Ok(Self {
            producer,
            board_column_updated_topic: config.board_column_updated_topic.clone(),
        })
    }

    pub async fn publish(&self, _event_type: &str, payload: &[u8]) -> anyhow::Result<()> {
        let record = FutureRecord::to(&self.board_column_updated_topic).payload(payload).key("board");
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("kafka send error: {}", e))?;
        Ok(())
    }
}
