use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::scheduler_job::{validate_cron, SchedulerJob};
use crate::domain::repository::SchedulerJobRepository;

#[derive(Debug, Clone)]
pub struct UpdateJobInput {
    pub id: Uuid,
    pub name: String,
    pub cron_expression: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateJobError {
    #[error("job not found: {0}")]
    NotFound(Uuid),

    #[error("invalid cron expression: {0}")]
    InvalidCron(String),

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

    pub async fn execute(&self, input: &UpdateJobInput) -> Result<SchedulerJob, UpdateJobError> {
        if !validate_cron(&input.cron_expression) {
            return Err(UpdateJobError::InvalidCron(input.cron_expression.clone()));
        }

        let mut job = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateJobError::Internal(e.to_string()))?
            .ok_or(UpdateJobError::NotFound(input.id))?;

        job.name = input.name.clone();
        job.cron_expression = input.cron_expression.clone();
        job.payload = input.payload.clone();
        job.updated_at = chrono::Utc::now();

        self.repo
            .update(&job)
            .await
            .map_err(|e| UpdateJobError::Internal(e.to_string()))?;

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
        let job = SchedulerJob::new(
            "original-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({"task": "original"}),
        );
        let job_id = job.id;
        let return_job = job.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(return_job.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateJobUseCase::new(Arc::new(mock));
        let input = UpdateJobInput {
            id: job_id,
            name: "updated-job".to_string(),
            cron_expression: "0 12 * * *".to_string(),
            payload: serde_json::json!({"task": "updated"}),
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
        let missing_id = Uuid::new_v4();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateJobUseCase::new(Arc::new(mock));
        let input = UpdateJobInput {
            id: missing_id,
            name: "test".to_string(),
            cron_expression: "* * * * *".to_string(),
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateJobError::NotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn invalid_cron() {
        let mock = MockSchedulerJobRepository::new();

        let uc = UpdateJobUseCase::new(Arc::new(mock));
        let input = UpdateJobInput {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            cron_expression: "bad".to_string(),
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateJobError::InvalidCron(expr) => assert_eq!(expr, "bad"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
