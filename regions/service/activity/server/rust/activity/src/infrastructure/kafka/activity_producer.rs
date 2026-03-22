use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub struct ActivityKafkaProducer {
    producer: FutureProducer,
    activity_created_topic: String,
    activity_approved_topic: String,
}

impl ActivityKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("acks", "all");
        client_config.set("enable.idempotence", "true");
        let producer = client_config.create()?;
        Ok(Self {
            producer,
            activity_created_topic: config.activity_created_topic.clone(),
            activity_approved_topic: config.activity_approved_topic.clone(),
        })
    }

    pub async fn publish(&self, event_type: &str, payload: &[u8]) -> anyhow::Result<()> {
        let topic = match event_type {
            "ActivityApproved" => &self.activity_approved_topic,
            _ => &self.activity_created_topic,
        };
        let record = FutureRecord::to(topic).payload(payload).key("activity");
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("kafka send error: {}", e))?;
        Ok(())
    }
}
