use std::sync::Arc;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;
use crate::domain::service::SchedulerDomainService;
use crate::infrastructure::kafka_producer::SchedulerEventPublisher;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct CreateJobInput {
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
    /// テナント ID: CRIT-005 対応。ジョブを作成するテナントを識別する。
    pub tenant_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateJobError {
    #[error("invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
    event_publisher: Arc<dyn SchedulerEventPublisher>,
}

impl CreateJobUseCase {
    pub fn new(
        repo: Arc<dyn SchedulerJobRepository>,
        event_publisher: Arc<dyn SchedulerEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &CreateJobInput) -> Result<SchedulerJob, CreateJobError> {
        if !SchedulerDomainService::validate_cron_expression(&input.cron_expression) {
            return Err(CreateJobError::InvalidCron(input.cron_expression.clone()));
        }
        if !crate::domain::entity::scheduler_job::validate_timezone(&input.timezone) {
            return Err(CreateJobError::InvalidTimezone(input.timezone.clone()));
        }

        let mut job = SchedulerJob::new(
            input.name.clone(),
            input.cron_expression.clone(),
            input.payload.clone(),
        );
        job.description = input.description.clone();
        job.timezone = input.timezone.clone();
        job.target_type = input.target_type.clone();
        job.target = input.target.clone();
        // CRIT-005 対応: リクエストのテナント ID をジョブに設定する
        job.tenant_id = input.tenant_id.clone();
        job.next_run_at = job.next_run_at();

        self.repo
            .create(&job)
            .await
            .map_err(|e| CreateJobError::Internal(e.to_string()))?;

        if let Err(err) = self.event_publisher.publish_job_created(&job).await {
            warn!(job_id = %job.id, error = %err, "failed to publish scheduler job created event");
        }

        Ok(job)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;
    use crate::infrastructure::kafka_producer::MockSchedulerEventPublisher;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        let mut publisher = MockSchedulerEventPublisher::new();
        mock.expect_create().returning(|_| Ok(()));
        publisher.expect_publish_job_created().returning(|_| Ok(()));

        let uc = CreateJobUseCase::new(Arc::new(mock), Arc::new(publisher));
        let input = CreateJobInput {
            name: "daily-backup".to_string(),
            description: None,
            cron_expression: "0 2 * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({"task": "backup"}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let job = result.unwrap();
        assert_eq!(job.name, "daily-backup");
        assert_eq!(job.status, "active");
        assert!(job.next_run_at.is_some());
        assert_eq!(job.tenant_id, "tenant-a");
    }

    #[tokio::test]
    async fn invalid_cron() {
        let mock = MockSchedulerJobRepository::new();
        let publisher = MockSchedulerEventPublisher::new();

        let uc = CreateJobUseCase::new(Arc::new(mock), Arc::new(publisher));
        let input = CreateJobInput {
            name: "bad-job".to_string(),
            description: None,
            cron_expression: "bad".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateJobError::InvalidCron(expr) => assert_eq!(expr, "bad"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn publish_failure_does_not_fail_create() {
        let mut mock = MockSchedulerJobRepository::new();
        let mut publisher = MockSchedulerEventPublisher::new();
        mock.expect_create().returning(|_| Ok(()));
        publisher
            .expect_publish_job_created()
            .returning(|_| Err(anyhow::anyhow!("kafka unavailable")));

        let uc = CreateJobUseCase::new(Arc::new(mock), Arc::new(publisher));
        let result = uc
            .execute(&CreateJobInput {
                name: "daily-backup".to_string(),
                description: None,
                cron_expression: "0 2 * * *".to_string(),
                timezone: "UTC".to_string(),
                target_type: "kafka".to_string(),
                target: Some("topic".to_string()),
                payload: serde_json::json!({"task": "backup"}),
                tenant_id: "tenant-a".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }
}
