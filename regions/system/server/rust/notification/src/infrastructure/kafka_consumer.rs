use std::sync::Arc;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::send_notification::{SendNotificationInput, SendNotificationUseCase};

/// NotificationRequestEvent は Kafka から受信する通知リクエストイベント。
#[derive(Debug, serde::Deserialize)]
pub struct NotificationRequestEvent {
    pub channel_id: String,
    pub recipient: String,
    #[serde(default)]
    pub subject: Option<String>,
    pub body: String,
    #[serde(default)]
    pub template_variables: Option<std::collections::HashMap<String, String>>,
}

/// NotificationKafkaConsumer は通知リクエストトピックを購読してメッセージを処理する。
pub struct NotificationKafkaConsumer {
    consumer: rdkafka::consumer::StreamConsumer,
    use_case: Arc<SendNotificationUseCase>,
    consumer_group: String,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl NotificationKafkaConsumer {
    /// 新しい NotificationKafkaConsumer を作成する。
    pub fn new(
        config: &KafkaConfig,
        use_case: Arc<SendNotificationUseCase>,
    ) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;
        use rdkafka::consumer::Consumer;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("group.id", &config.consumer_group);
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("auto.offset.reset", "earliest");
        client_config.set("enable.auto.commit", "true");

        let consumer: rdkafka::consumer::StreamConsumer = client_config.create()?;
        consumer.subscribe(&[&config.topic_requested])?;

        tracing::info!(
            topic = %config.topic_requested,
            group = %config.consumer_group,
            "notification kafka consumer subscribed"
        );

        Ok(Self {
            consumer,
            use_case,
            consumer_group: config.consumer_group.clone(),
            metrics: None,
        })
    }

    /// メトリクスを設定する。
    pub fn with_metrics(mut self, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// バックグラウンドでメッセージ取り込みを開始する。
    pub async fn run(&self) -> anyhow::Result<()> {
        use rdkafka::Message;

        loop {
            match self.consumer.recv().await {
                Err(e) => {
                    tracing::error!(error = %e, "notification kafka consumer error");
                }
                Ok(msg) => {
                    let topic = msg.topic().to_string();
                    if let Some(ref m) = self.metrics {
                        m.record_kafka_message_consumed(&topic, &self.consumer_group);
                    }

                    let payload = match msg.payload() {
                        Some(bytes) => bytes,
                        None => {
                            tracing::warn!("received kafka message with empty payload");
                            continue;
                        }
                    };

                    let event: NotificationRequestEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!(error = %e, "failed to deserialize notification request");
                            continue;
                        }
                    };

                    let channel_id: uuid::Uuid = match event.channel_id.parse() {
                        Ok(id) => id,
                        Err(e) => {
                            tracing::error!(error = %e, channel_id = %event.channel_id, "invalid channel_id");
                            continue;
                        }
                    };

                    let input = SendNotificationInput {
                        channel_id,
                        recipient: event.recipient,
                        subject: event.subject,
                        body: event.body,
                        template_variables: event.template_variables,
                    };

                    match self.use_case.execute(&input).await {
                        Ok(output) => {
                            tracing::info!(
                                log_id = %output.log_id,
                                status = %output.status,
                                "notification request processed from kafka"
                            );
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "failed to process notification request");
                        }
                    }
                }
            }
        }
    }
}
