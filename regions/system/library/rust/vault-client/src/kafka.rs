use crate::error::VaultError;
use crate::secret::SecretRotatedEvent;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;
use rdkafka::Message;

/// Kafka subscriber for secret rotation events.
pub struct VaultSecretRotationSubscriber {
    consumer: StreamConsumer,
}

impl VaultSecretRotationSubscriber {
    pub fn new(
        brokers: &str,
        consumer_group: &str,
        topic: &str,
    ) -> Result<Self, VaultError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", consumer_group)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| VaultError::ServerError(format!("kafka create consumer failed: {e}")))?;

        consumer
            .subscribe(&[topic])
            .map_err(|e| VaultError::ServerError(format!("kafka subscribe failed: {e}")))?;

        Ok(Self { consumer })
    }

    pub async fn next_event(&self) -> Result<SecretRotatedEvent, VaultError> {
        let message = self
            .consumer
            .recv()
            .await
            .map_err(|e| VaultError::ServerError(format!("kafka receive failed: {e}")))?;

        let payload = message
            .payload()
            .ok_or_else(|| VaultError::ServerError("kafka payload is empty".to_string()))?;

        serde_json::from_slice::<SecretRotatedEvent>(payload)
            .map_err(|e| VaultError::ServerError(format!("kafka payload decode failed: {e}")))
    }
}
