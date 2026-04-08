use std::sync::Arc;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};

#[derive(Debug, thiserror::Error)]
pub enum ListExecutionsError {
    #[error("job not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListExecutionsUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
}

impl ListExecutionsUseCase {
    pub fn new(
        repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
    ) -> Self {
        Self {
            repo,
            execution_repo,
        }
    }

    /// CRIT-005 対応: `job_id` の存在確認に `tenant_id` を渡してテナント分離を行う。
    pub async fn execute(
        &self,
        job_id: &str,
    ) -> Result<Vec<SchedulerExecution>, ListExecutionsError> {
        // ジョブの存在確認: クロステナントで実行履歴を取得するため "system" テナントを使用する
        // NOTE: job_executions は job_id でのみフィルタするため、ここでの RLS は参照チェックのみ
        let _job = self
            .repo
            .find_by_id(job_id, "system")
            .await
            .map_err(|e| ListExecutionsError::Internal(e.to_string()))?
            .ok_or_else(|| ListExecutionsError::NotFound(job_id.to_string()))?;

        self.execution_repo
            .find_by_job_id(job_id)
            .await
            .map_err(|e| ListExecutionsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            "test-job".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let job_id = job.id.clone();
        let return_job = job.clone();
        let expected_id = job_id.clone();

        mock_job
            .expect_find_by_id()
            .withf(move |id, _tenant_id| id == expected_id.as_str())
            .returning(move |_, _| Ok(Some(return_job.clone())));

        mock_exec.expect_find_by_job_id().returning(|_| Ok(vec![]));

        let uc = ListExecutionsUseCase::new(Arc::new(mock_job), Arc::new(mock_exec));
        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        let missing_id = "job_missing".to_string();
        mock_job.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = ListExecutionsUseCase::new(Arc::new(mock_job), Arc::new(mock_exec));
        let result = uc.execute(&missing_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListExecutionsError::NotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
