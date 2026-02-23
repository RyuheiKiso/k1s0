use std::sync::Arc;

use crate::domain::entity::scheduler_job::{validate_cron, SchedulerJob};
use crate::domain::repository::SchedulerJobRepository;

#[derive(Debug, Clone)]
pub struct CreateJobInput {
    pub name: String,
    pub cron_expression: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateJobError {
    #[error("invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl CreateJobUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &CreateJobInput) -> Result<SchedulerJob, CreateJobError> {
        if !validate_cron(&input.cron_expression) {
            return Err(CreateJobError::InvalidCron(input.cron_expression.clone()));
        }

        let job = SchedulerJob::new(
            input.name.clone(),
            input.cron_expression.clone(),
            input.payload.clone(),
        );

        self.repo
            .create(&job)
            .await
            .map_err(|e| CreateJobError::Internal(e.to_string()))?;

        Ok(job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateJobUseCase::new(Arc::new(mock));
        let input = CreateJobInput {
            name: "daily-backup".to_string(),
            cron_expression: "0 2 * * *".to_string(),
            payload: serde_json::json!({"task": "backup"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let job = result.unwrap();
        assert_eq!(job.name, "daily-backup");
        assert_eq!(job.status, "active");
    }

    #[tokio::test]
    async fn invalid_cron() {
        let mock = MockSchedulerJobRepository::new();

        let uc = CreateJobUseCase::new(Arc::new(mock));
        let input = CreateJobInput {
            name: "bad-job".to_string(),
            cron_expression: "bad".to_string(),
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateJobError::InvalidCron(expr) => assert_eq!(expr, "bad"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
