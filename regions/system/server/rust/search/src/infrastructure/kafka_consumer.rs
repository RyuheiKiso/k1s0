use std::sync::Arc;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::delete_document::{DeleteDocumentInput, DeleteDocumentUseCase};
use crate::usecase::index_document::{IndexDocumentInput, IndexDocumentUseCase};

/// IndexRequestEvent は Kafka から受信するインデックス登録リクエストイベント。
#[derive(Debug, serde::Deserialize)]
pub struct IndexRequestEvent {
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(alias = "id")]
    pub document_id: String,
    #[serde(alias = "index_name")]
    pub index: String,
    #[serde(default, alias = "content")]
    pub document: Option<serde_json::Value>,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub actor_service: Option<String>,
}

/// SearchKafkaConsumer はインデックス登録リクエストトピックを購読してメッセージを処理する。
pub struct SearchKafkaConsumer {
    consumer: rdkafka::consumer::StreamConsumer,
    index_use_case: Arc<IndexDocumentUseCase>,
    delete_use_case: Arc<DeleteDocumentUseCase>,
    consumer_group: String,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl SearchKafkaConsumer {
    /// 新しい SearchKafkaConsumer を作成する。
    pub fn new(
        config: &KafkaConfig,
        index_use_case: Arc<IndexDocumentUseCase>,
        delete_use_case: Arc<DeleteDocumentUseCase>,
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
            index_use_case,
            delete_use_case,
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

                    let operation = event
                        .operation
                        .clone()
                        .unwrap_or_else(|| "INDEX".to_string())
                        .to_ascii_uppercase();

                    if operation == "DELETE" {
                        let input = DeleteDocumentInput {
                            index_name: event.index,
                            doc_id: event.document_id,
                        };
                        match self.delete_use_case.execute(&input).await {
                            Ok(_) => {
                                tracing::info!("document deleted from kafka");
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to delete document from kafka");
                            }
                        }
                        continue;
                    }

                    let Some(document) = event.document else {
                        tracing::warn!(
                            operation = %operation,
                            "received index event without document payload"
                        );
                        continue;
                    };

                    let input = IndexDocumentInput {
                        id: event.document_id,
                        index_name: event.index,
                        content: document,
                    };

                    match self.index_use_case.execute(&input).await {
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
