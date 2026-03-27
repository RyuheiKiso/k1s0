//! KafkaEventProducer: rdkafka を使用した EventProducer 実装。
//! feature = "kafka" で有効化される。

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::error::MessagingError;
use crate::event::EventEnvelope;
use crate::producer::EventProducer;

/// KafkaEventProducer は rdkafka の FutureProducer を使った実装。
pub struct KafkaEventProducer {
    producer: FutureProducer,
    /// send() 呼び出し時のタイムアウト。キューへの配置待ち上限時間。
    send_timeout: Duration,
}

impl KafkaEventProducer {
    /// ブローカーリストから KafkaEventProducer を生成する。デフォルトのタイムアウト値を使用する。
    pub fn new(brokers: &str) -> Result<Self, MessagingError> {
        Self::new_with_timeout(brokers, 30_000)
    }

    /// ブローカーリストとタイムアウト（ミリ秒）を指定して KafkaEventProducer を生成する。
    /// タイムアウトを設定ファイルから制御したい場合は with_config を使用すること。
    pub fn new_with_timeout(
        brokers: &str,
        message_timeout_ms: u64,
    ) -> Result<Self, MessagingError> {
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", &message_timeout_ms.to_string())
            .set("enable.idempotence", "true")
            .create()
            .map_err(|e| MessagingError::ConnectionError(e.to_string()))?;
        // send() のタイムアウトは message.timeout.ms と同じ値を使用する
        let send_timeout = Duration::from_millis(message_timeout_ms);
        Ok(Self {
            producer,
            send_timeout,
        })
    }

    /// MessagingConfig から KafkaEventProducer を生成する。
    /// タイムアウトは MessagingConfig.timeout_ms の値を使用する。
    pub fn with_config(config: &crate::config::MessagingConfig) -> Result<Self, MessagingError> {
        let brokers = config.brokers_string();
        Self::new_with_timeout(&brokers, config.timeout_ms)
    }
}

#[async_trait]
impl EventProducer for KafkaEventProducer {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), MessagingError> {
        let record = FutureRecord::to(&envelope.topic)
            .key(&envelope.key)
            .payload(&envelope.payload as &[u8]);

        self.producer
            .send(record, self.send_timeout)
            .await
            .map_err(|(err, _)| MessagingError::PublishError(err.to_string()))?;

        Ok(())
    }

    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<(), MessagingError> {
        for envelope in envelopes {
            self.publish(envelope).await?;
        }
        Ok(())
    }
}
