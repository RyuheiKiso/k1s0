use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};
use crate::infrastructure::kafka_producer::SchedulerEventPublisher;

#[derive(Debug, thiserror::Error)]
pub enum TriggerJobError {
    #[error("job not found: {0}")]
    NotFound(Uuid),

    #[error("job not active: {0}")]
    NotActive(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct TriggerJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
    event_publisher: Arc<dyn SchedulerEventPublisher>,
}

impl TriggerJobUseCase {
    pub fn new(
        repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
    ) -> Self {
        use crate::infrastructure::kafka_producer::NoopSchedulerEventPublisher;
        Self {
            repo,
            execution_repo,
            event_publisher: Arc::new(NoopSchedulerEventPublisher),
        }
    }

    pub fn with_publisher(
        repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
        event_publisher: Arc<dyn SchedulerEventPublisher>,
    ) -> Self {
        Self {
            repo,
            execution_repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, job_id: &Uuid) -> Result<SchedulerExecution, TriggerJobError> {
        let mut job = self
            .repo
            .find_by_id(job_id)
            .await
            .map_err(|e| TriggerJobError::Internal(e.to_string()))?
            .ok_or(TriggerJobError::NotFound(*job_id))?;

        if job.status != "active" {
            return Err(TriggerJobError::NotActive(*job_id));
        }

        let execution = SchedulerExecution::new(job.id);

        // 実行記録を保存
        self.execution_repo
            .create(&execution)
            .await
            .map_err(|e| TriggerJobError::Internal(e.to_string()))?;

        job.last_run_at = Some(chrono::Utc::now());
        job.updated_at = chrono::Utc::now();

        self.repo
            .update(&job)
            .await
            .map_err(|e| TriggerJobError::Internal(e.to_string()))?;

        let _ = self
            .event_publisher
            .publish_job_executed(&job, &execution)
            .await;

        // 実行完了のステータスを更新
        let _ = self
            .execution_repo
            .update_status(&execution.id, "completed".to_string(), None)
            .await;

        Ok(execution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_execution_repository::MockSchedulerExecutionRepository;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let job = SchedulerJob::new(
            "trigger-test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let job_id = job.id;
        let return_job = job.clone();

        mock_job
            .expect_find_by_id()
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(return_job.clone())));
        mock_job.expect_update().returning(|_| Ok(()));

        mock_exec.expect_create().returning(|_| Ok(()));
        mock_exec
            .expect_update_status()
            .returning(|_, _, _| Ok(()));

        let uc = TriggerJobUseCase::new(Arc::new(mock_job), Arc::new(mock_exec));
        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let execution = result.unwrap();
        assert_eq!(execution.job_id, job_id);
        assert_eq!(execution.status, "running");
    }

    #[tokio::test]
    async fn not_active() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        let mut job = SchedulerJob::new(
            "paused-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        job.status = "paused".to_string();
        let job_id = job.id;
        let return_job = job.clone();

        mock_job
            .expect_find_by_id()
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(return_job.clone())));

        let uc = TriggerJobUseCase::new(Arc::new(mock_job), Arc::new(mock_exec));
        let result = uc.execute(&job_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            TriggerJobError::NotActive(id) => assert_eq!(id, job_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
