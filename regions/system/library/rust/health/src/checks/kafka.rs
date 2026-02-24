use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use crate::checker::HealthCheck;
use crate::error::HealthError;

pub struct KafkaHealthCheck {
    name: String,
    brokers: Vec<String>,
    timeout_ms: u64,
}

impl KafkaHealthCheck {
    pub fn new(name: impl Into<String>, brokers: Vec<String>) -> Self {
        Self {
            name: name.into(),
            brokers,
            timeout_ms: 5000,
        }
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

#[async_trait]
impl HealthCheck for KafkaHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> Result<(), HealthError> {
        let brokers = self.brokers.join(",");
        let timeout_ms = self.timeout_ms;

        // ブロッキング操作を spawn_blocking で実行
        tokio::task::spawn_blocking(move || {
            let consumer: BaseConsumer = ClientConfig::new()
                .set("bootstrap.servers", &brokers)
                .set("socket.timeout.ms", timeout_ms.to_string())
                .create()
                .map_err(|e| HealthError::CheckFailed(format!("Kafka client creation failed: {}", e)))?;

            consumer
                .fetch_metadata(None, std::time::Duration::from_millis(timeout_ms))
                .map(|_| ())
                .map_err(|e| HealthError::CheckFailed(format!("Kafka broker unreachable: {}", e)))
        })
        .await
        .map_err(|e| HealthError::CheckFailed(format!("Task join error: {}", e)))?
    }
}
