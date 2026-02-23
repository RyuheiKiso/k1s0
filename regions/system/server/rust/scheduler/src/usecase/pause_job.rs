use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;

#[derive(Debug, thiserror::Error)]
pub enum PauseJobError {
    #[error("job not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct PauseJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl PauseJobUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, job_id: &Uuid) -> Result<SchedulerJob, PauseJobError> {
        let mut job = self
            .repo
            .find_by_id(job_id)
            .await
            .map_err(|e| PauseJobError::Internal(e.to_string()))?
            .ok_or(PauseJobError::NotFound(*job_id))?;

        job.status = "paused".to_string();
        job.updated_at = chrono::Utc::now();

        self.repo
            .update(&job)
            .await
            .map_err(|e| PauseJobError::Internal(e.to_string()))?;

        Ok(job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        let job = SchedulerJob::new(
            "pause-test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let job_id = job.id;
        let return_job = job.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(return_job.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = PauseJobUseCase::new(Arc::new(mock));
        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let paused = result.unwrap();
        assert_eq!(paused.status, "paused");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        let missing_id = Uuid::new_v4();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = PauseJobUseCase::new(Arc::new(mock));
        let result = uc.execute(&missing_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PauseJobError::NotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
