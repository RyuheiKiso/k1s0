use async_trait::async_trait;

use crate::domain::entity::session::Session;

/// SessionEvent はセッション関連イベントの種別を表す。
#[derive(Debug, serde::Serialize)]
#[serde(tag = "event_type")]
pub enum SessionEvent {
    #[serde(rename = "session.created")]
    Created {
        session_id: String,
        user_id: String,
        created_at: String,
        expires_at: String,
    },
    #[serde(rename = "session.revoked")]
    Revoked {
        session_id: String,
        user_id: String,
        revoked_at: String,
    },
}

/// SessionEventPublisher はセッションイベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SessionEventPublisher: Send + Sync {
    async fn publish_session_created(&self, session: &Session) -> anyhow::Result<()>;
    async fn publish_session_revoked(
        &self,
        session_id: &str,
        user_id: &str,
    ) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopSessionEventPublisher はイベントを発行しない実装（フォールバック用）。
pub struct NoopSessionEventPublisher;

#[async_trait]
impl SessionEventPublisher for NoopSessionEventPublisher {
    async fn publish_session_created(&self, _session: &Session) -> anyhow::Result<()> {
        tracing::debug!("noop: session created event skipped");
        Ok(())
    }

    async fn publish_session_revoked(
        &self,
        _session_id: &str,
        _user_id: &str,
    ) -> anyhow::Result<()> {
        tracing::debug!("noop: session revoked event skipped");
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaSessionProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaSessionProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaSessionProducer {
    /// 新しい KafkaSessionProducer を作成する。
    pub fn new(config: &crate::infrastructure::config::KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config
            .topic_created
            .clone();

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
            topic,
            metrics: None,
        })
    }

    /// メトリクスを設定する。
    pub fn with_metrics(
        mut self,
        metrics: std::sync::Arc<k1s0_telemetry::metrics::Metrics>,
    ) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

#[async_trait]
impl SessionEventPublisher for KafkaSessionProducer {
    async fn publish_session_created(&self, session: &Session) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = SessionEvent::Created {
            session_id: session.id.clone(),
            user_id: session.user_id.clone(),
            created_at: session.created_at.to_rfc3339(),
            expires_at: session.expires_at.to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;
        let key = format!("session:{}", session.id);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish session created event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }

    async fn publish_session_revoked(
        &self,
        session_id: &str,
        user_id: &str,
    ) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = SessionEvent::Revoked {
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            revoked_at: chrono::Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;
        let key = format!("session:{}", session_id);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish session revoked event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        use rdkafka::producer::Producer;
        self.producer.flush(std::time::Duration::from_secs(5))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryProducer {
        messages: Mutex<Vec<(String, Vec<u8>)>>,
        should_fail: bool,
    }

    impl InMemoryProducer {
        fn new() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn with_error() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl SessionEventPublisher for InMemoryProducer {
        async fn publish_session_created(&self, session: &Session) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = SessionEvent::Created {
                session_id: session.id.clone(),
                user_id: session.user_id.clone(),
                created_at: session.created_at.to_rfc3339(),
                expires_at: session.expires_at.to_rfc3339(),
            };
            let payload = serde_json::to_vec(&event)?;
            let key = format!("session:{}", session.id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn publish_session_revoked(
            &self,
            session_id: &str,
            user_id: &str,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = SessionEvent::Revoked {
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                revoked_at: Utc::now().to_rfc3339(),
            };
            let payload = serde_json::to_vec(&event)?;
            let key = format!("session:{}", session_id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_session() -> Session {
        Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_publish_session_created() {
        let producer = InMemoryProducer::new();
        let session = make_test_session();

        let result = producer.publish_session_created(&session).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "session:sess-1");

        let event: serde_json::Value = serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(event["event_type"], "session.created");
        assert_eq!(event["session_id"], "sess-1");
        assert_eq!(event["user_id"], "user-1");
    }

    #[tokio::test]
    async fn test_publish_session_revoked() {
        let producer = InMemoryProducer::new();

        let result = producer
            .publish_session_revoked("sess-1", "user-1")
            .await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let event: serde_json::Value = serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(event["event_type"], "session.revoked");
        assert_eq!(event["session_id"], "sess-1");
    }

    #[tokio::test]
    async fn test_publish_error() {
        let producer = InMemoryProducer::with_error();
        let session = make_test_session();

        let result = producer.publish_session_created(&session).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopSessionEventPublisher;
        let session = make_test_session();

        assert!(publisher.publish_session_created(&session).await.is_ok());
        assert!(publisher
            .publish_session_revoked("sess-1", "user-1")
            .await
            .is_ok());
        assert!(publisher.close().await.is_ok());
    }
}
