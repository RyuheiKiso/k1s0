use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::Value;
use std::time::Duration;

pub struct MasterMaintenanceKafkaProducer {
    producer: FutureProducer,
    topic: String,
}

impl MasterMaintenanceKafkaProducer {
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
            topic: config.topic.clone(),
        })
    }

    #[must_use] 
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// シャットダウン時に未送信メッセージをフラッシュして失われるのを防ぐ（AVAIL-005 監査対応）。
    pub async fn close(&self) -> anyhow::Result<()> {
        use rdkafka::producer::Producer;
        self.producer.flush(Duration::from_secs(10))?;
        Ok(())
    }

    pub async fn publish_data_changed(&self, event: &Value) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        let key = event
            .get("resource_id")
            .and_then(Value::as_str)
            .or_else(|| event.get("resource_name").and_then(Value::as_str))
            .unwrap_or("master-maintenance");

        tracing::info!(topic = %self.topic, key, "publishing data changed event");

        self.producer
            .send(
                FutureRecord::to(&self.topic).key(key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish data changed event: {err}"))?;

        Ok(())
    }

    pub async fn publish_domain_data_changed(
        &self,
        domain: &str,
        event: &Value,
    ) -> anyhow::Result<()> {
        let domain_topic = format!("k1s0.business.{domain}.mastermaintenance.data_changed.v1");
        let payload = serde_json::to_vec(event)?;
        let key = event
            .get("resource_id")
            .and_then(Value::as_str)
            .or_else(|| event.get("resource_name").and_then(Value::as_str))
            .unwrap_or("master-maintenance");

        tracing::info!(topic = %domain_topic, key, domain, "publishing domain data changed event");

        self.producer
            .send(
                FutureRecord::to(&domain_topic).key(key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish domain data changed event: {err}")
            })?;

        Ok(())
    }
}
