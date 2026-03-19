use std::sync::Arc;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};
use crate::domain::service::SchedulerDomainService;
use crate::infrastructure::job_executor::{JobExecutor, NoopJobExecutor};
use crate::infrastructure::kafka_producer::SchedulerEventPublisher;

#[derive(Debug, thiserror::Error)]
pub enum TriggerJobError {
    #[error("job not found: {0}")]
    NotFound(String),

    #[error("job not active: {0}")]
    NotActive(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct TriggerJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
    executor: Arc<dyn JobExecutor>,
    event_publisher: Arc<dyn SchedulerEventPublisher>,
}

impl TriggerJobUseCase {
    #[allow(dead_code)]
    pub fn new(
        repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
    ) -> Self {
        use crate::infrastructure::kafka_producer::NoopSchedulerEventPublisher;

        Self {
            repo,
            execution_repo,
            executor: Arc::new(NoopJobExecutor),
            event_publisher: Arc::new(NoopSchedulerEventPublisher),
        }
    }

    pub fn with_dependencies(
        repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
        executor: Arc<dyn JobExecutor>,
        event_publisher: Arc<dyn SchedulerEventPublisher>,
    ) -> Self {
        Self {
            repo,
            execution_repo,
            executor,
            event_publisher,
        }
    }

    pub async fn execute(&self, job_id: &str) -> Result<SchedulerExecution, TriggerJobError> {
        let mut job = self
            .repo
            .find_by_id(job_id)
            .await
            .map_err(|e| TriggerJobError::Internal(e.to_string()))?
            .ok_or_else(|| TriggerJobError::NotFound(job_id.to_string()))?;

        if !SchedulerDomainService::can_trigger(&job.status) {
            return Err(TriggerJobError::NotActive(job_id.to_string()));
        }

        let mut execution = SchedulerExecution::new(job.id.clone());
        execution.triggered_by = "manual".to_string();

        self.execution_repo
            .create(&execution)
            .await
            .map_err(|e| TriggerJobError::Internal(e.to_string()))?;

        let finished_at = chrono::Utc::now();
        match self.executor.execute(&job).await {
            Ok(()) => {
                execution.status = "succeeded".to_string();
                execution.finished_at = Some(finished_at);
            }
            Err(err) => {
                execution.status = "failed".to_string();
                execution.finished_at = Some(finished_at);
                execution.error_message = Some(err.to_string());
            }
        }

        job.last_run_at = Some(finished_at);
        job.next_run_at = job.next_run_at();
        job.updated_at = finished_at;

        self.repo
            .update(&job)
            .await
            .map_err(|e| TriggerJobError::Internal(e.to_string()))?;

        let _ = self
            .event_publisher
            .publish_job_executed(&job, &execution)
            .await;

        let _ = self
            .execution_repo
            .update_status(
                &execution.id,
                execution.status.clone(),
                execution.error_message.clone(),
            )
            .await;

        if execution.status == "failed" {
            return Err(TriggerJobError::Internal(
                execution
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "job execution failed".to_string()),
            ));
        }

        Ok(execution)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_execution_repository::MockSchedulerExecutionRepository;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;
    use crate::infrastructure::job_executor::MockJobExecutor;
    use crate::infrastructure::kafka_producer::MockSchedulerEventPublisher;

    #[tokio::test]
    async fn success() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let mut executor = MockJobExecutor::new();
        let mut publisher = MockSchedulerEventPublisher::new();
        let job = SchedulerJob::new(
            "trigger-test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let job_id = job.id.clone();
        let return_job = job.clone();
        let expected_id = job_id.clone();

        mock_job
            .expect_find_by_id()
            .withf(move |id| id == expected_id.as_str())
            .returning(move |_| Ok(Some(return_job.clone())));
        mock_job.expect_update().returning(|_| Ok(()));

        mock_exec.expect_create().returning(|_| Ok(()));
        mock_exec.expect_update_status().returning(|_, _, _| Ok(()));
        executor.expect_execute().returning(|_| Ok(()));
        publisher
            .expect_publish_job_executed()
            .returning(|_, _| Ok(()));

        let uc = TriggerJobUseCase::with_dependencies(
            Arc::new(mock_job),
            Arc::new(mock_exec),
            Arc::new(executor),
            Arc::new(publisher),
        );
        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let execution = result.unwrap();
        assert_eq!(execution.job_id, job_id);
        assert_eq!(execution.status, "succeeded");
        assert_eq!(execution.triggered_by, "manual");
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
        let job_id = job.id.clone();
        let return_job = job.clone();
        let expected_id = job_id.clone();

        mock_job
            .expect_find_by_id()
            .withf(move |id| id == expected_id.as_str())
            .returning(move |_| Ok(Some(return_job.clone())));

        let uc = TriggerJobUseCase::new(Arc::new(mock_job), Arc::new(mock_exec));
        let result = uc.execute(&job_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            TriggerJobError::NotActive(id) => assert_eq!(id, job_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn execution_failure_is_returned() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let mut executor = MockJobExecutor::new();
        let mut publisher = MockSchedulerEventPublisher::new();
        let job = SchedulerJob::new(
            "trigger-test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let return_job = job.clone();

        mock_job
            .expect_find_by_id()
            .returning(move |_| Ok(Some(return_job.clone())));
        mock_job.expect_update().returning(|_| Ok(()));
        mock_exec.expect_create().returning(|_| Ok(()));
        mock_exec.expect_update_status().returning(|_, _, _| Ok(()));
        executor
            .expect_execute()
            .returning(|_| Err(anyhow::anyhow!("target failed")));
        publisher
            .expect_publish_job_executed()
            .returning(|_, _| Ok(()));

        let uc = TriggerJobUseCase::with_dependencies(
            Arc::new(mock_job),
            Arc::new(mock_exec),
            Arc::new(executor),
            Arc::new(publisher),
        );

        let result = uc.execute(&job.id).await;
        assert!(matches!(result, Err(TriggerJobError::Internal(_))));
    }
}
