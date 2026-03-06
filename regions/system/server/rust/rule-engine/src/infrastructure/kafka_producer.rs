use async_trait::async_trait;

#[derive(Debug, serde::Serialize)]
pub struct RuleChangedEvent {
    pub event_type: String,
    pub rule_set_id: Option<String>,
    pub rule_set_name: Option<String>,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub domain: Option<String>,
    pub action: String,
    pub version: Option<u32>,
    pub previous_version: Option<u32>,
    pub timestamp: String,
    pub actor_user_id: Option<String>,
}

impl RuleChangedEvent {
    pub fn rule_created(rule: &crate::domain::entity::rule::Rule) -> Self {
        Self {
            event_type: "RULE_CHANGED".to_string(),
            rule_set_id: None,
            rule_set_name: None,
            rule_id: Some(rule.id.to_string()),
            rule_name: Some(rule.name.clone()),
            domain: None,
            action: "CREATED".to_string(),
            version: Some(rule.version),
            previous_version: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor_user_id: None,
        }
    }

    pub fn rule_updated(rule: &crate::domain::entity::rule::Rule) -> Self {
        Self {
            event_type: "RULE_CHANGED".to_string(),
            rule_set_id: None,
            rule_set_name: None,
            rule_id: Some(rule.id.to_string()),
            rule_name: Some(rule.name.clone()),
            domain: None,
            action: "UPDATED".to_string(),
            version: Some(rule.version),
            previous_version: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor_user_id: None,
        }
    }

    pub fn rule_deleted(rule: &crate::domain::entity::rule::Rule) -> Self {
        Self {
            event_type: "RULE_CHANGED".to_string(),
            rule_set_id: None,
            rule_set_name: None,
            rule_id: Some(rule.id.to_string()),
            rule_name: Some(rule.name.clone()),
            domain: None,
            action: "DELETED".to_string(),
            version: Some(rule.version),
            previous_version: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor_user_id: None,
        }
    }

    pub fn rule_set_published(
        rs: &crate::domain::entity::rule::RuleSet,
        new_version: u32,
        prev_version: u32,
    ) -> Self {
        Self {
            event_type: "RULE_SET_PUBLISHED".to_string(),
            rule_set_id: Some(rs.id.to_string()),
            rule_set_name: Some(rs.name.clone()),
            rule_id: None,
            rule_name: None,
            domain: Some(rs.domain.clone()),
            action: "PUBLISHED".to_string(),
            version: Some(new_version),
            previous_version: Some(prev_version),
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor_user_id: None,
        }
    }

    pub fn rule_set_rolled_back(
        rs: &crate::domain::entity::rule::RuleSet,
        rolled_back_to: u32,
        prev_version: u32,
    ) -> Self {
        Self {
            event_type: "RULE_SET_ROLLED_BACK".to_string(),
            rule_set_id: Some(rs.id.to_string()),
            rule_set_name: Some(rs.name.clone()),
            rule_id: None,
            rule_name: None,
            domain: Some(rs.domain.clone()),
            action: "ROLLED_BACK".to_string(),
            version: Some(rolled_back_to),
            previous_version: Some(prev_version),
            timestamp: chrono::Utc::now().to_rfc3339(),
            actor_user_id: None,
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RuleEventPublisher: Send + Sync {
    async fn publish_rule_changed(&self, event: &RuleChangedEvent) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

pub struct NoopRuleEventPublisher;

#[async_trait]
impl RuleEventPublisher for NoopRuleEventPublisher {
    async fn publish_rule_changed(&self, _event: &RuleChangedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct KafkaRuleProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaRuleProducer {
    pub fn new(config: &crate::infrastructure::config::KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config.topic.clone();

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

    pub fn with_metrics(
        mut self,
        metrics: std::sync::Arc<k1s0_telemetry::metrics::Metrics>,
    ) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

#[async_trait]
impl RuleEventPublisher for KafkaRuleProducer {
    async fn publish_rule_changed(&self, event: &RuleChangedEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = event
            .rule_set_id
            .as_deref()
            .or(event.rule_id.as_deref())
            .unwrap_or("unknown");

        let record = FutureRecord::to(&self.topic)
            .key(key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish rule changed event: {}", err)
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
