use std::sync::Arc;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;

#[derive(Debug, thiserror::Error)]
pub enum ResumeJobError {
    #[error("job not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ResumeJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl ResumeJobUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからジョブを再開する。
    pub async fn execute(
        &self,
        job_id: &str,
        tenant_id: &str,
    ) -> Result<SchedulerJob, ResumeJobError> {
        let mut job = self
            .repo
            .find_by_id(job_id, tenant_id)
            .await
            .map_err(|e| ResumeJobError::Internal(e.to_string()))?
            .ok_or_else(|| ResumeJobError::NotFound(job_id.to_string()))?;

        job.status = "active".to_string();
        job.updated_at = chrono::Utc::now();

        self.repo
            .update(&job)
            .await
            .map_err(|e| ResumeJobError::Internal(e.to_string()))?;

        Ok(job)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        let mut job = SchedulerJob::new(
            "resume-test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        job.status = "paused".to_string();
        let job_id = job.id.clone();
        let return_job = job.clone();
        let expected_id = job_id.clone();

        mock.expect_find_by_id()
            .withf(move |id, _tenant_id| id == expected_id.as_str())
            .returning(move |_, _| Ok(Some(return_job.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = ResumeJobUseCase::new(Arc::new(mock));
        let result = uc.execute(&job_id, "tenant-a").await;
        assert!(result.is_ok());

        let resumed = result.unwrap();
        assert_eq!(resumed.status, "active");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        let missing_id = "job_missing".to_string();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = ResumeJobUseCase::new(Arc::new(mock));
        let result = uc.execute(&missing_id, "tenant-a").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ResumeJobError::NotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
