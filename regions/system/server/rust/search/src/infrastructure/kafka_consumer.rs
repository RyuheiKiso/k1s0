use std::sync::Arc;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::delete_document::{DeleteDocumentInput, DeleteDocumentUseCase};
use crate::usecase::index_document::{IndexDocumentInput, IndexDocumentUseCase};

/// IndexRequestEvent は Kafka から受信するインデックス登録リクエストイベント。
/// CRIT-002 対応: tenant_id フィールドを追加し、送信元サービスから正しいテナントを受け取れるようにする。
/// tenant_id が省略されている古いメッセージとの後方互換性のため serde(default) を使用する。
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
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
    /// イベント送信元サービスのテナント ID。
    /// 省略された場合（古いメッセージ）は "system" にフォールバックする。
    /// フォールバックは後方互換性のためだけに使用し、新規メッセージでは必ず含めること。
    #[serde(default)]
    pub tenant_id: Option<String>,
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
        // at-least-once セマンティクスのため auto.commit を無効化する
        client_config.set("enable.auto.commit", "false");

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
        use rdkafka::consumer::{CommitMode, Consumer};
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
                        // CRIT-002 対応: Kafkaイベントの tenant_id フィールドを使用する。
                        // tenant_id が省略されている古いメッセージは "system" にフォールバックし、後方互換性を保つ。
                        // 新規メッセージでは送信元サービスが必ず tenant_id を含めること。
                        let tenant_id = event.tenant_id.clone().unwrap_or_else(|| {
                            tracing::warn!(
                                index = %event.index,
                                "Kafka DELETE イベントに tenant_id が含まれていません。\"system\" にフォールバックします"
                            );
                            "system".to_string()
                        });
                        let input = DeleteDocumentInput {
                            index_name: event.index,
                            doc_id: event.document_id,
                            tenant_id,
                        };
                        match self.delete_use_case.execute(&input).await {
                            Ok(_) => {
                                tracing::info!("document deleted from kafka");
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to delete document from kafka");
                            }
                        }
                        // 処理成功後にオフセットを手動コミットする
                        if let Err(e) = self.consumer.commit_message(&msg, CommitMode::Async) {
                            tracing::warn!(error = %e, "failed to commit kafka offset");
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

                    // CRIT-002 対応: Kafkaイベントの tenant_id フィールドを使用する。
                    // tenant_id が省略されている古いメッセージは "system" にフォールバックし、後方互換性を保つ。
                    // 新規メッセージでは送信元サービスが必ず tenant_id を含めること。
                    let tenant_id = event.tenant_id.unwrap_or_else(|| {
                        tracing::warn!(
                            index = %event.index,
                            "Kafka INDEX イベントに tenant_id が含まれていません。\"system\" にフォールバックします"
                        );
                        "system".to_string()
                    });
                    let input = IndexDocumentInput {
                        id: event.document_id,
                        index_name: event.index,
                        content: document,
                        tenant_id,
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

                    // 処理成功後にオフセットを手動コミットする
                    if let Err(e) = self.consumer.commit_message(&msg, CommitMode::Async) {
                        tracing::warn!(error = %e, "failed to commit kafka offset");
                    }
                }
            }
        }
    }
}
