use crate::infrastructure::config::KafkaConfig;
use serde_json::Value;

pub struct MasterMaintenanceKafkaProducer {
    topic: String,
    // TODO: rdkafka::producer::FutureProducer
}

impl MasterMaintenanceKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        Ok(Self {
            topic: config.topic.clone(),
        })
    }

    pub async fn publish_data_changed(&self, event: &Value) -> anyhow::Result<()> {
        tracing::info!(topic = %self.topic, "publishing data changed event");
        let _ = event;
        // TODO: Implement Kafka publish
        Ok(())
    }
}
