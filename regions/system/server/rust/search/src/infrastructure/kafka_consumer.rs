use std::sync::Arc;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::index_document::{IndexDocumentInput, IndexDocumentUseCase};

/// IndexRequestEvent は Kafka から受信するインデックス登録リクエストイベント。
#[derive(Debug, serde::Deserialize)]
pub struct IndexRequestEvent {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
}

/// SearchKafkaConsumer はインデックス登録リクエストトピックを購読してメッセージを処理する。
pub struct SearchKafkaConsumer {
    consumer: rdkafka::consumer::StreamConsumer,
    use_case: Arc<IndexDocumentUseCase>,
    consumer_group: String,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl SearchKafkaConsumer {
    /// 新しい SearchKafkaConsumer を作成する。
    pub fn new(
        config: &KafkaConfig,
        use_case: Arc<IndexDocumentUseCase>,
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
        consumer.subscribe(&[&config.topic])?;

        tracing::info!(
            topic = %config.topic,
            group = %config.consumer_group,
            "search kafka consumer subscribed"
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
                    tracing::error!(error = %e, "search kafka consumer error");
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

                    let event: IndexRequestEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!(error = %e, "failed to deserialize index request");
                            continue;
                        }
                    };

                    let input = IndexDocumentInput {
                        id: event.id,
                        index_name: event.index_name,
                        content: event.content,
                    };

                    match self.use_case.execute(&input).await {
                        Ok(doc) => {
                            tracing::info!(
                                doc_id = %doc.id,
                                index = %doc.index_name,
                                "document indexed from kafka"
                            );
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "failed to index document from kafka");
                        }
                    }
                }
            }
        }
    }
}
