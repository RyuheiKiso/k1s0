use async_trait::async_trait;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::entity::scheduler_job::SchedulerJob;

/// SchedulerEventPublisher はスケジューライベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchedulerEventPublisher: Send + Sync {
    /// ジョブ実行完了イベントを発行する。
    async fn publish_job_executed(
        &self,
        job: &SchedulerJob,
        execution: &SchedulerExecution,
    ) -> anyhow::Result<()>;

    /// ジョブ作成イベントを発行する。
    async fn publish_job_created(&self, job: &SchedulerJob) -> anyhow::Result<()>;

    /// プロデューサーを安全にシャットダウンする。
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopSchedulerEventPublisher はイベント発行を行わないデフォルト実装。
pub struct NoopSchedulerEventPublisher;

#[async_trait]
impl SchedulerEventPublisher for NoopSchedulerEventPublisher {
    async fn publish_job_executed(
        &self,
        _job: &SchedulerJob,
        _execution: &SchedulerExecution,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_job_created(&self, _job: &SchedulerJob) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaSchedulerProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaSchedulerProducer {
    producer: rdkafka::producer::FutureProducer,
    topic_executed: String,
    topic_created: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaSchedulerProducer {
    /// 新しい KafkaSchedulerProducer を作成する。
    pub fn new(config: &super::config::KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
            topic_executed: "k1s0.system.scheduler.executed.v1".to_string(),
            topic_created: "k1s0.system.scheduler.created.v1".to_string(),
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
impl SchedulerEventPublisher for KafkaSchedulerProducer {
    async fn publish_job_executed(
        &self,
        job: &SchedulerJob,
        execution: &SchedulerExecution,
    ) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = serde_json::json!({
            "job_id": job.id.to_string(),
            "job_name": job.name,
            "execution_id": execution.id.to_string(),
            "status": execution.status,
            "started_at": execution.started_at.to_rfc3339(),
            "completed_at": execution.completed_at.map(|t| t.to_rfc3339()),
            "error_message": execution.error_message,
        });

        let payload = serde_json::to_vec(&event)?;
        let key = job.id.to_string();

        let record = FutureRecord::to(&self.topic_executed)
            .key(&key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish scheduler executed event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic_executed);
        }

        Ok(())
    }

    async fn publish_job_created(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = serde_json::json!({
            "job_id": job.id.to_string(),
            "job_name": job.name,
            "cron_expression": job.cron_expression,
            "status": job.status,
            "created_at": job.created_at.to_rfc3339(),
        });

        let payload = serde_json::to_vec(&event)?;
        let key = job.id.to_string();

        let record = FutureRecord::to(&self.topic_created)
            .key(&key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish scheduler created event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic_created);
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
    use std::sync::Mutex;

    /// テスト用のインメモリプロデューサー。
    struct InMemorySchedulerProducer {
        executed_events: Mutex<Vec<Vec<u8>>>,
        created_events: Mutex<Vec<Vec<u8>>>,
        should_fail: bool,
    }

    impl InMemorySchedulerProducer {
        fn new() -> Self {
            Self {
                executed_events: Mutex::new(Vec::new()),
                created_events: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn with_error() -> Self {
            Self {
                executed_events: Mutex::new(Vec::new()),
                created_events: Mutex::new(Vec::new()),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl SchedulerEventPublisher for InMemorySchedulerProducer {
        async fn publish_job_executed(
            &self,
            job: &SchedulerJob,
            execution: &SchedulerExecution,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = serde_json::json!({
                "job_id": job.id.to_string(),
                "execution_id": execution.id.to_string(),
                "status": execution.status,
            });
            let payload = serde_json::to_vec(&event)?;
            self.executed_events.lock().unwrap().push(payload);
            Ok(())
        }

        async fn publish_job_created(&self, job: &SchedulerJob) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = serde_json::json!({
                "job_id": job.id.to_string(),
                "job_name": job.name,
            });
            let payload = serde_json::to_vec(&event)?;
            self.created_events.lock().unwrap().push(payload);
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_job() -> SchedulerJob {
        SchedulerJob::new(
            "test-job".to_string(),
            "0 12 * * *".to_string(),
            serde_json::json!({"task": "backup"}),
        )
    }

    fn make_test_execution(job_id: uuid::Uuid) -> SchedulerExecution {
        SchedulerExecution::new(job_id)
    }

    #[tokio::test]
    async fn test_publish_job_executed() {
        let producer = InMemorySchedulerProducer::new();
        let job = make_test_job();
        let execution = make_test_execution(job.id);

        let result = producer.publish_job_executed(&job, &execution).await;
        assert!(result.is_ok());

        let events = producer.executed_events.lock().unwrap();
        assert_eq!(events.len(), 1);

        let deserialized: serde_json::Value = serde_json::from_slice(&events[0]).unwrap();
        assert_eq!(deserialized["job_id"], job.id.to_string());
        assert_eq!(deserialized["execution_id"], execution.id.to_string());
        assert_eq!(deserialized["status"], "running");
    }

    #[tokio::test]
    async fn test_publish_job_created() {
        let producer = InMemorySchedulerProducer::new();
        let job = make_test_job();

        let result = producer.publish_job_created(&job).await;
        assert!(result.is_ok());

        let events = producer.created_events.lock().unwrap();
        assert_eq!(events.len(), 1);

        let deserialized: serde_json::Value = serde_json::from_slice(&events[0]).unwrap();
        assert_eq!(deserialized["job_id"], job.id.to_string());
        assert_eq!(deserialized["job_name"], "test-job");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemorySchedulerProducer::with_error();
        let job = make_test_job();
        let execution = make_test_execution(job.id);

        let result = producer.publish_job_executed(&job, &execution).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopSchedulerEventPublisher;
        let job = make_test_job();
        let execution = make_test_execution(job.id);

        assert!(publisher
            .publish_job_executed(&job, &execution)
            .await
            .is_ok());
        assert!(publisher.publish_job_created(&job).await.is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_mock_scheduler_event_publisher() {
        let mut mock = MockSchedulerEventPublisher::new();
        mock.expect_publish_job_executed().returning(|_, _| Ok(()));
        mock.expect_publish_job_created().returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let job = make_test_job();
        let execution = make_test_execution(job.id);

        assert!(mock.publish_job_executed(&job, &execution).await.is_ok());
        assert!(mock.publish_job_created(&job).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
