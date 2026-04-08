use std::sync::Arc;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::send_notification::{SendNotificationInput, SendNotificationUseCase};

/// `NotificationRequestEvent` は Kafka から受信する通知リクエストイベント。
/// MEDIUM-RUST-001 監査対応: `tenant_id` を追加してテナント分離を有効化する。
/// Kafka プロデューサーはイベント発行時に `tenant_id` を含める必要がある。
/// 後方互換のため serde(default) で未設定時は "system" にフォールバックする。
#[derive(Debug, serde::Deserialize)]
pub struct NotificationRequestEvent {
    pub channel_id: String,
    /// 通知を発行したテナントの ID。未指定の場合は "system" にフォールバックする。
    #[serde(default = "default_system_tenant")]
    pub tenant_id: String,
    #[serde(default)]
    pub template_id: Option<String>,
    pub recipient: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub template_variables: Option<std::collections::HashMap<String, String>>,
}

/// MEDIUM-RUST-001 監査対応: `tenant_id` が未指定の場合のデフォルト値。
/// 既存の Kafka プロデューサーとの後方互換性を保つためのフォールバック。
fn default_system_tenant() -> String {
    "system".to_string()
}

/// `NotificationKafkaConsumer` は通知リクエストトピックを購読してメッセージを処理する。
pub struct NotificationKafkaConsumer {
    consumer: rdkafka::consumer::StreamConsumer,
    use_case: Arc<SendNotificationUseCase>,
    consumer_group: String,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl NotificationKafkaConsumer {
    /// 新しい `NotificationKafkaConsumer` を作成する。
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
        // at-least-once セマンティクスのため auto.commit を無効化する
        client_config.set("enable.auto.commit", "false");

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
    #[must_use] 
    pub fn with_metrics(mut self, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// バックグラウンドでメッセージ取り込みを開始する。
    pub async fn run(&self) -> anyhow::Result<()> {
        use rdkafka::consumer::Consumer;
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

                    let payload = if let Some(bytes) = msg.payload() { bytes } else {
                        tracing::warn!("received kafka message with empty payload");
                        continue;
                    };

                    let event: NotificationRequestEvent = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!(error = %e, "failed to deserialize notification request");
                            continue;
                        }
                    };

                    if event.channel_id.trim().is_empty() {
                        tracing::error!("invalid channel_id");
                        continue;
                    }

                    // MEDIUM-RUST-001 監査対応: event.tenant_id を伝播してテナント分離を有効化する
                    let input = SendNotificationInput {
                        channel_id: event.channel_id,
                        tenant_id: event.tenant_id,
                        template_id: event.template_id,
                        recipient: event.recipient,
                        subject: event.subject,
                        body: event.body.unwrap_or_default(),
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

                    // 処理成功後にオフセットを手動コミットする
                    if let Err(e) = self
                        .consumer
                        .commit_message(&msg, rdkafka::consumer::CommitMode::Async)
                    {
                        tracing::warn!(error = %e, "failed to commit kafka offset");
                    }
                }
            }
        }
    }
}
