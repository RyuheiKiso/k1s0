use std::sync::Arc;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::revoke_all_sessions::{RevokeAllSessionsInput, RevokeAllSessionsUseCase};

/// `RevokeAllRequestEvent` は Kafka から受信する全セッション失効リクエストイベント。
#[derive(Debug, serde::Deserialize)]
pub struct RevokeAllRequestEvent {
    pub user_id: String,
}

/// `SessionKafkaConsumer` は全セッション失効リクエストトピックを購読してメッセージを処理する。
pub struct SessionKafkaConsumer {
    consumer: rdkafka::consumer::StreamConsumer,
    use_case: Arc<RevokeAllSessionsUseCase>,
    consumer_group: String,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl SessionKafkaConsumer {
    /// 新しい `SessionKafkaConsumer` を作成する。
    pub fn new(
        config: &KafkaConfig,
        use_case: Arc<RevokeAllSessionsUseCase>,
    ) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;
        use rdkafka::consumer::Consumer;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("group.id", &config.consumer_group);
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("auto.offset.reset", "earliest");
        // at-least-once セマンティクスのため auto.commit を無効化する
        client_config.set("enable.auto.commit", "false");

        let consumer: rdkafka::consumer::StreamConsumer = client_config.create()?;
        consumer.subscribe(&[&config.topic_revoke_all])?;

        tracing::info!(
            topic = %config.topic_revoke_all,
            group = %config.consumer_group,
            "session kafka consumer subscribed"
        );

        Ok(Self {
            consumer,
            use_case,
            consumer_group: config.consumer_group.clone(),
            metrics: None,
        })
    }

    /// メトリクスを設定する。
    #[must_use] 
    pub fn with_metrics(mut self, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// バックグラウンドでメッセージ取り込みを開始する。
    pub async fn run(&self) -> anyhow::Result<()> {
        use rdkafka::consumer::{CommitMode, Consumer};
        use rdkafka::Message;

        loop {
            match self.consumer.recv().await {
                Err(e) => {
                    tracing::error!(error = %e, "session kafka consumer error");
                }
                Ok(msg) => {
                    let topic = msg.topic().to_string();
                    if let Some(ref m) = self.metrics {
                        m.record_kafka_message_consumed(&topic, &self.consumer_group);
                    }

                    let payload = if let Some(bytes) = msg.payload() { bytes } else {
                        tracing::warn!("received kafka message with empty payload");
                        continue;
                    };

                    let event: RevokeAllRequestEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!(error = %e, "failed to deserialize revoke all request");
                            continue;
                        }
                    };

                    let user_id = event.user_id;
                    let input = RevokeAllSessionsInput {
                        user_id: user_id.clone(),
                    };

                    match self.use_case.execute(&input).await {
                        Ok(output) => {
                            tracing::info!(
                                user_id = %user_id,
                                revoked_count = output.count,
                                "all sessions revoked from kafka"
                            );
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "failed to revoke all sessions from kafka");
                        }
                    }

                    // 処理成功後にオフセットを手動コミットする
                    if let Err(e) = self.consumer.commit_message(&msg, CommitMode::Async) {
                        tracing::warn!(error = %e, "failed to commit kafka offset");
                    }
                }
            }
        }
    }
}
