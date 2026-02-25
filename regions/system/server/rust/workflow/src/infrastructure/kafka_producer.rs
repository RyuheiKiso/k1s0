use async_trait::async_trait;
use serde::Serialize;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::entity::workflow_task::WorkflowTask;

/// ワークフローイベントの Kafka 発行用トレイト。
#[async_trait]
pub trait WorkflowEventPublisher: Send + Sync {
    async fn publish_instance_started(&self, instance: &WorkflowInstance) -> anyhow::Result<()>;
    async fn publish_task_completed(&self, task: &WorkflowTask) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// InstanceStartedEvent は Kafka へ発行するワークフロー開始イベント。
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct InstanceStartedEvent {
    pub instance_id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub status: String,
    pub timestamp: String,
}

/// TaskCompletedEvent は Kafka へ発行するタスク完了イベント。
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct TaskCompletedEvent {
    pub task_id: String,
    pub instance_id: String,
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub actor_id: Option<String>,
    pub timestamp: String,
}

/// NoopWorkflowEventPublisher は何も発行しないプロデューサー（InMemory / テスト用）。
pub struct NoopWorkflowEventPublisher;

#[async_trait]
impl WorkflowEventPublisher for NoopWorkflowEventPublisher {
    async fn publish_instance_started(&self, _instance: &WorkflowInstance) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_task_completed(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaWorkflowEventPublisher は rdkafka FutureProducer を使った実装。
pub struct KafkaWorkflowEventPublisher {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaWorkflowEventPublisher {
    /// 新しい KafkaWorkflowEventPublisher を作成する。
    pub fn new(config: &crate::infrastructure::config::KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config.state_topic.clone();

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

    /// 配信先トピック名を返す。
    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[async_trait]
impl WorkflowEventPublisher for KafkaWorkflowEventPublisher {
    async fn publish_instance_started(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = InstanceStartedEvent {
            instance_id: instance.id.clone(),
            workflow_id: instance.workflow_id.clone(),
            workflow_name: instance.workflow_name.clone(),
            title: instance.title.clone(),
            initiator_id: instance.initiator_id.clone(),
            status: instance.status.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;
        let key = format!("instance:{}", instance.id);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish instance started event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }

    async fn publish_task_completed(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = TaskCompletedEvent {
            task_id: task.id.clone(),
            instance_id: task.instance_id.clone(),
            step_id: task.step_id.clone(),
            step_name: task.step_name.clone(),
            status: task.status.clone(),
            actor_id: task.actor_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;
        let key = format!("task:{}", task.id);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish task completed event: {}", err)
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
    use chrono::Utc;
    use std::sync::Mutex;

    /// テスト用のインメモリプロデューサー。
    struct InMemoryPublisher {
        instance_events: Mutex<Vec<Vec<u8>>>,
        task_events: Mutex<Vec<Vec<u8>>>,
        should_fail: bool,
    }

    impl InMemoryPublisher {
        fn new() -> Self {
            Self {
                instance_events: Mutex::new(Vec::new()),
                task_events: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn with_error() -> Self {
            Self {
                instance_events: Mutex::new(Vec::new()),
                task_events: Mutex::new(Vec::new()),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl WorkflowEventPublisher for InMemoryPublisher {
        async fn publish_instance_started(
            &self,
            instance: &WorkflowInstance,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = InstanceStartedEvent {
                instance_id: instance.id.clone(),
                workflow_id: instance.workflow_id.clone(),
                workflow_name: instance.workflow_name.clone(),
                title: instance.title.clone(),
                initiator_id: instance.initiator_id.clone(),
                status: instance.status.clone(),
                timestamp: Utc::now().to_rfc3339(),
            };
            let payload = serde_json::to_vec(&event)?;
            self.instance_events.lock().unwrap().push(payload);
            Ok(())
        }

        async fn publish_task_completed(&self, task: &WorkflowTask) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = TaskCompletedEvent {
                task_id: task.id.clone(),
                instance_id: task.instance_id.clone(),
                step_id: task.step_id.clone(),
                step_name: task.step_name.clone(),
                status: task.status.clone(),
                actor_id: task.actor_id.clone(),
                timestamp: Utc::now().to_rfc3339(),
            };
            let payload = serde_json::to_vec(&event)?;
            self.task_events.lock().unwrap().push(payload);
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn sample_instance() -> WorkflowInstance {
        WorkflowInstance::new(
            "inst_001".to_string(),
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "PC Purchase".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({"item": "laptop"}),
        )
    }

    fn sample_task() -> WorkflowTask {
        let mut task = WorkflowTask::new(
            "task_001".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "Manager Approval".to_string(),
            Some("user-002".to_string()),
            Some(Utc::now() + chrono::Duration::hours(48)),
        );
        task.approve("user-002".to_string(), Some("LGTM".to_string()));
        task
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopWorkflowEventPublisher;
        let inst = sample_instance();
        assert!(publisher.publish_instance_started(&inst).await.is_ok());
        let task = sample_task();
        assert!(publisher.publish_task_completed(&task).await.is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_publish_instance_started_event() {
        let publisher = InMemoryPublisher::new();
        let inst = sample_instance();

        publisher.publish_instance_started(&inst).await.unwrap();

        let events = publisher.instance_events.lock().unwrap();
        assert_eq!(events.len(), 1);

        let deserialized: InstanceStartedEvent = serde_json::from_slice(&events[0]).unwrap();
        assert_eq!(deserialized.instance_id, "inst_001");
        assert_eq!(deserialized.workflow_id, "wf_001");
        assert_eq!(deserialized.workflow_name, "purchase-approval");
        assert_eq!(deserialized.status, "running");
    }

    #[tokio::test]
    async fn test_publish_task_completed_event() {
        let publisher = InMemoryPublisher::new();
        let task = sample_task();

        publisher.publish_task_completed(&task).await.unwrap();

        let events = publisher.task_events.lock().unwrap();
        assert_eq!(events.len(), 1);

        let deserialized: TaskCompletedEvent = serde_json::from_slice(&events[0]).unwrap();
        assert_eq!(deserialized.task_id, "task_001");
        assert_eq!(deserialized.instance_id, "inst_001");
        assert_eq!(deserialized.step_id, "step-1");
        assert_eq!(deserialized.status, "approved");
        assert_eq!(deserialized.actor_id, Some("user-002".to_string()));
    }

    #[tokio::test]
    async fn test_publish_error() {
        let publisher = InMemoryPublisher::with_error();
        let inst = sample_instance();

        let result = publisher.publish_instance_started(&inst).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_close_graceful() {
        let publisher = InMemoryPublisher::new();
        assert!(publisher.close().await.is_ok());
    }

    #[test]
    fn test_instance_started_event_serialization() {
        let event = InstanceStartedEvent {
            instance_id: "inst_001".to_string(),
            workflow_id: "wf_001".to_string(),
            workflow_name: "purchase-approval".to_string(),
            title: "PC Purchase".to_string(),
            initiator_id: "user-001".to_string(),
            status: "running".to_string(),
            timestamp: "2026-02-26T00:00:00Z".to_string(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["instance_id"], "inst_001");
        assert_eq!(json["workflow_name"], "purchase-approval");
        assert_eq!(json["status"], "running");
    }

    #[test]
    fn test_task_completed_event_serialization() {
        let event = TaskCompletedEvent {
            task_id: "task_001".to_string(),
            instance_id: "inst_001".to_string(),
            step_id: "step-1".to_string(),
            step_name: "Manager Approval".to_string(),
            status: "approved".to_string(),
            actor_id: Some("user-002".to_string()),
            timestamp: "2026-02-26T00:00:00Z".to_string(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["task_id"], "task_001");
        assert_eq!(json["status"], "approved");
        assert_eq!(json["actor_id"], "user-002");
    }
}
