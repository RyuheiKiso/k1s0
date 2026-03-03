use async_trait::async_trait;
use serde::Serialize;

use crate::domain::entity::workflow_task::WorkflowTask;

#[derive(Debug, Clone, Serialize)]
pub struct NotificationRequestedEvent {
    pub event_type: String,
    pub request_id: String,
    pub task_id: String,
    pub instance_id: String,
    pub assignee_id: Option<String>,
    pub status: String,
    pub requested_at: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationRequestPublisher: Send + Sync {
    async fn publish_task_overdue(&self, task: &WorkflowTask) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

pub struct NoopNotificationRequestPublisher;

#[async_trait]
impl NotificationRequestPublisher for NoopNotificationRequestPublisher {
    async fn publish_task_overdue(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct KafkaNotificationRequestPublisher {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}

impl KafkaNotificationRequestPublisher {
    pub fn new(config: &crate::infrastructure::config::KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;
        Ok(Self {
            producer,
            topic: config.notification_topic.clone(),
        })
    }
}

#[async_trait]
impl NotificationRequestPublisher for KafkaNotificationRequestPublisher {
    async fn publish_task_overdue(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = NotificationRequestedEvent {
            event_type: "TASK_OVERDUE".to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            task_id: task.id.clone(),
            instance_id: task.instance_id.clone(),
            assignee_id: task.assignee_id.clone(),
            status: task.status.clone(),
            requested_at: chrono::Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;
        let key = task.id.clone();
        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish notification requested event: {}", err)
            })?;

        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        use rdkafka::producer::Producer;
        self.producer.flush(std::time::Duration::from_secs(5))?;
        Ok(())
    }
}
