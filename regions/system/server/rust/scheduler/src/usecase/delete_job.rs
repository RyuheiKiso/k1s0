use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};
use crate::domain::service::SchedulerDomainService;

#[derive(Debug, thiserror::Error)]
pub enum DeleteJobError {
    #[error("job not found: {0}")]
    NotFound(Uuid),

    #[error("job is currently running: {0}")]
    JobRunning(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
}

impl DeleteJobUseCase {
    pub fn new(
        repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
    ) -> Self {
        Self {
            repo,
            execution_repo,
        }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteJobError> {
        let _job = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| DeleteJobError::Internal(e.to_string()))?
            .ok_or(DeleteJobError::NotFound(*id))?;

        let executions = self
            .execution_repo
            .find_by_job_id(id)
            .await
            .map_err(|e| DeleteJobError::Internal(e.to_string()))?;
        if SchedulerDomainService::has_running_execution(&executions) {
            return Err(DeleteJobError::JobRunning(*id));
        }

        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteJobError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteJobError::NotFound(*id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_execution::SchedulerExecution;
    use crate::domain::repository::scheduler_execution_repository::MockSchedulerExecutionRepository;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let id = Uuid::new_v4();
        mock.expect_find_by_id()
            .withf(move |job_id| *job_id == id)
            .returning(|job_id| {
                Ok(Some(crate::domain::entity::scheduler_job::SchedulerJob {
                    id: *job_id,
                    name: "job".to_string(),
                    description: None,
                    cron_expression: "* * * * *".to_string(),
                    timezone: "UTC".to_string(),
                    target_type: "kafka".to_string(),
                    target: None,
                    payload: serde_json::json!({}),
                    status: "active".to_string(),
                    next_run_at: None,
                    last_run_at: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });
        mock_exec.expect_find_by_job_id().returning(|_| Ok(vec![]));
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteJobUseCase::new(Arc::new(mock), Arc::new(mock_exec));
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = DeleteJobUseCase::new(Arc::new(mock), Arc::new(mock_exec));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteJobError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteJobUseCase::new(Arc::new(mock), Arc::new(mock_exec));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteJobError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn job_running() {
        let mut mock = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let id = Uuid::new_v4();
        mock.expect_find_by_id().returning(|job_id| {
            Ok(Some(crate::domain::entity::scheduler_job::SchedulerJob {
                id: *job_id,
                name: "job".to_string(),
                description: None,
                cron_expression: "* * * * *".to_string(),
                timezone: "UTC".to_string(),
                target_type: "kafka".to_string(),
                target: None,
                payload: serde_json::json!({}),
                status: "active".to_string(),
                next_run_at: None,
                last_run_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        mock_exec.expect_find_by_job_id().returning(move |_| {
            Ok(vec![SchedulerExecution {
                id: Uuid::new_v4(),
                job_id: id,
                status: "running".to_string(),
                triggered_by: "scheduler".to_string(),
                started_at: chrono::Utc::now(),
                finished_at: None,
                error_message: None,
            }])
        });

        let uc = DeleteJobUseCase::new(Arc::new(mock), Arc::new(mock_exec));
        let result = uc.execute(&id).await;
        assert!(matches!(result, Err(DeleteJobError::JobRunning(found)) if found == id));
    }
}
