use std::sync::Arc;

use crate::domain::entity::DlqMessage;
use crate::domain::repository::DlqMessageRepository;

use super::KafkaConfig;

/// DlqKafkaConsumer は DLQ トピックを購読して新しいメッセージを取り込む。
pub struct DlqKafkaConsumer {
    _consumer: rdkafka::consumer::StreamConsumer,
    repo: Arc<dyn DlqMessageRepository>,
    consumer_group: String,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl DlqKafkaConsumer {
    /// 新しい DlqKafkaConsumer を作成する。
    pub fn new(config: &KafkaConfig, repo: Arc<dyn DlqMessageRepository>) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;
        use rdkafka::consumer::Consumer;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("group.id", &config.consumer_group);
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("auto.offset.reset", "earliest");
        client_config.set("enable.auto.commit", "true");

        let consumer: rdkafka::consumer::StreamConsumer = client_config.create()?;

        // DLQ トピックパターンを購読
        let pattern = &config.dlq_topic_pattern;
        consumer.subscribe(&[pattern])?;

        tracing::info!(
            pattern = %pattern,
            "DLQ kafka consumer subscribed"
        );

        Ok(Self {
            _consumer: consumer,
            repo,
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
        use rdkafka::message::Headers;
        use rdkafka::Message;

        let consumer = &self._consumer;

        loop {
            match consumer.recv().await {
                Err(e) => {
                    tracing::error!(error = %e, "kafka consumer error");
                }
                Ok(msg) => {
                    let topic = msg.topic().to_string();
                    if let Some(ref m) = self.metrics {
                        m.record_kafka_message_consumed(&topic, &self.consumer_group);
                    }

                    // ペイロードのデシリアライズを試行し、失敗時はポイズンピルとして即座に Dead 扱いにする
                    let (payload, is_poison) = match msg.payload() {
                        Some(bytes) => match serde_json::from_slice::<serde_json::Value>(bytes) {
                            Ok(val) => (val, false),
                            Err(parse_err) => {
                                tracing::warn!(
                                    topic = %topic,
                                    error = %parse_err,
                                    "ポイズンピル検出: デシリアライズ失敗、即座に Dead としてマーク"
                                );
                                // パースエラー時は生バイトを文字列として保持する
                                let raw = String::from_utf8_lossy(bytes).to_string();
                                (serde_json::json!({"raw_payload": raw}), true)
                            }
                        },
                        None => (serde_json::Value::Null, false),
                    };

                    let error_message = msg
                        .headers()
                        .and_then(|h| {
                            for i in 0..h.count() {
                                let header = h.get(i);
                                if header.key == "error" {
                                    return Some(
                                        String::from_utf8_lossy(header.value.unwrap_or(b""))
                                            .to_string(),
                                    );
                                }
                            }
                            None
                        })
                        .unwrap_or_else(|| "unknown error".to_string());

                    let mut dlq_message = DlqMessage::new(topic, error_message, payload, 3);

                    // ポイズンピル（パースエラー）の場合は即座に Dead 状態にする（リトライ不要）
                    if is_poison {
                        dlq_message.mark_dead();
                    }

                    if let Err(e) = self.repo.create(&dlq_message).await {
                        tracing::error!(error = %e, "failed to persist DLQ message");
                    } else {
                        tracing::info!(
                            message_id = %dlq_message.id,
                            topic = %dlq_message.original_topic,
                            is_poison = is_poison,
                            "DLQ message ingested"
                        );
                    }
                }
            }
        }
    }
}
