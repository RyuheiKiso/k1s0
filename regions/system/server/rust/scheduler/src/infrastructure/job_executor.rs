use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use reqwest::Client;
use serde_json::Value;

use crate::domain::entity::scheduler_job::SchedulerJob;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait JobExecutor: Send + Sync {
    async fn execute(&self, job: &SchedulerJob) -> anyhow::Result<()>;
}

#[allow(dead_code)]
pub struct NoopJobExecutor;

#[async_trait]
impl JobExecutor for NoopJobExecutor {
    async fn execute(&self, _job: &SchedulerJob) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct TargetJobExecutor {
    http: Client,
    kafka: Option<FutureProducer>,
}

impl TargetJobExecutor {
    pub fn new(kafka_config: Option<&super::config::KafkaConfig>) -> anyhow::Result<Self> {
        let kafka = if let Some(cfg) = kafka_config {
            let mut client_config = ClientConfig::new();
            client_config.set("bootstrap.servers", cfg.brokers.join(","));
            client_config.set("security.protocol", &cfg.security_protocol);
            client_config.set("acks", "all");
            client_config.set("message.timeout.ms", "5000");
            // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
            client_config.set("enable.idempotence", "true");
            Some(client_config.create()?)
        } else {
            None
        };

        Ok(Self {
            http: Client::new(),
            kafka,
        })
    }

    async fn execute_http(&self, target: &str, payload: &Value) -> anyhow::Result<()> {
        self.http
            .post(target)
            .json(payload)
            .send()
            .await
            .with_context(|| format!("failed to POST scheduler target {}", target))?
            .error_for_status()
            .with_context(|| format!("scheduler HTTP target returned error for {}", target))?;
        Ok(())
    }

    async fn execute_kafka(&self, topic: &str, payload: &Value) -> anyhow::Result<()> {
        let producer = self.kafka.as_ref().ok_or_else(|| {
            anyhow::anyhow!("kafka target requested but kafka producer is not configured")
        })?;

        let body = serde_json::to_vec(payload)?;
        producer
            .send(
                FutureRecord::to(topic).key("").payload(&body),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish scheduler target message: {}", err)
            })?;
        Ok(())
    }
}

#[async_trait]
impl JobExecutor for TargetJobExecutor {
    async fn execute(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        match job.target_type.as_str() {
            "http" => {
                let target = job
                    .target
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("http target is required for job {}", job.id))?;
                self.execute_http(target, &job.payload).await
            }
            "kafka" => {
                let target = job.target.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("kafka target is required for job {}", job.id)
                })?;
                self.execute_kafka(target, &job.payload).await
            }
            other => Err(anyhow::anyhow!(
                "unsupported scheduler target_type '{}' for job {}",
                other,
                job.id
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;

    #[tokio::test]
    async fn rejects_unsupported_target_type() {
        let executor = TargetJobExecutor::new(None).unwrap();
        let mut job = SchedulerJob::new(
            "job".to_string(),
            "*/15 * * * *".to_string(),
            serde_json::json!({}),
        );
        job.target_type = "unknown".to_string();
        let err = executor.execute(&job).await.unwrap_err();
        assert!(err
            .to_string()
            .contains("unsupported scheduler target_type"));
    }
}
