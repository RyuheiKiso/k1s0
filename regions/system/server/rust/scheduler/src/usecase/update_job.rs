use std::sync::Arc;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;
use crate::domain::service::SchedulerDomainService;

#[derive(Debug, Clone)]
pub struct UpdateJobInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
    /// テナント ID: CRIT-005 対応。テナント分離のために使用する。
    pub tenant_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateJobError {
    #[error("job not found: {0}")]
    NotFound(String),

    #[error("invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl UpdateJobUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからジョブを更新する。
    pub async fn execute(&self, input: &UpdateJobInput) -> Result<SchedulerJob, UpdateJobError> {
        if !SchedulerDomainService::validate_cron_expression(&input.cron_expression) {
            return Err(UpdateJobError::InvalidCron(input.cron_expression.clone()));
        }
        if !crate::domain::entity::scheduler_job::validate_timezone(&input.timezone) {
            return Err(UpdateJobError::InvalidTimezone(input.timezone.clone()));
        }

        let mut job = self
            .repo
            .find_by_id(&input.id, &input.tenant_id)
            .await
            .map_err(|e| UpdateJobError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateJobError::NotFound(input.id.clone()))?;

        job.name = input.name.clone();
        job.description = input.description.clone();
        job.cron_expression = input.cron_expression.clone();
        job.timezone = input.timezone.clone();
        job.target_type = input.target_type.clone();
        job.target = input.target.clone();
        job.payload = input.payload.clone();
        job.next_run_at = job.next_run_at();
        job.updated_at = chrono::Utc::now();

        self.repo
            .update(&job)
            .await
            .map_err(|e| UpdateJobError::Internal(e.to_string()))?;

        Ok(job)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        let job = SchedulerJob::new(
            "original-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({"task": "original"}),
        );
        let job_id = job.id.clone();
        let return_job = job.clone();
        let expected_id = job_id.clone();

        mock.expect_find_by_id()
            .withf(move |id, _tenant_id| id == expected_id.as_str())
            .returning(move |_, _| Ok(Some(return_job.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateJobUseCase::new(Arc::new(mock));
        let input = UpdateJobInput {
            id: job_id.clone(),
            name: "updated-job".to_string(),
            description: None,
            cron_expression: "0 12 * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({"task": "updated"}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-job");
        assert_eq!(updated.cron_expression, "0 12 * * *");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        let missing_id = "job_missing".to_string();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = UpdateJobUseCase::new(Arc::new(mock));
        let expected_missing_id = missing_id.clone();
        let input = UpdateJobInput {
            id: missing_id,
            name: "test".to_string(),
            description: None,
            cron_expression: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateJobError::NotFound(id) => assert_eq!(id, expected_missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn invalid_cron() {
        let mock = MockSchedulerJobRepository::new();

        let uc = UpdateJobUseCase::new(Arc::new(mock));
        let input = UpdateJobInput {
            id: "job_invalid_cron".to_string(),
            name: "test".to_string(),
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
            UpdateJobError::InvalidCron(expr) => assert_eq!(expr, "bad"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
