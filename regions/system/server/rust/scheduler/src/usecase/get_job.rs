use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetJobError {
    #[error("job not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl GetJobUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<SchedulerJob, GetJobError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetJobError::Internal(e.to_string()))?
            .ok_or(GetJobError::NotFound(*id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn found() {
        let mut mock = MockSchedulerJobRepository::new();
        let job = SchedulerJob::new(
            "test-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let job_id = job.id;
        let return_job = job.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(return_job.clone())));

        let uc = GetJobUseCase::new(Arc::new(mock));
        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "test-job");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        let missing_id = Uuid::new_v4();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetJobUseCase::new(Arc::new(mock));
        let result = uc.execute(&missing_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetJobError::NotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
