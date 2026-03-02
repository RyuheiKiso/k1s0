use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::repository::SchedulerExecutionRepository;
use crate::usecase::trigger_job::{TriggerJobError, TriggerJobUseCase};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct TriggerJobRequest {
    pub job_id: String,
}

#[derive(Debug, Clone)]
pub struct TriggerJobResponse {
    pub execution_id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GetJobExecutionRequest {
    pub execution_id: String,
}

#[derive(Debug, Clone)]
pub struct GetJobExecutionResponse {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_by: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error("unimplemented: {0}")]
    Unimplemented(String),
}

// --- SchedulerGrpcService ---

pub struct SchedulerGrpcService {
    trigger_job_uc: Arc<TriggerJobUseCase>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
}

impl SchedulerGrpcService {
    pub fn new(
        trigger_job_uc: Arc<TriggerJobUseCase>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
    ) -> Self {
        Self {
            trigger_job_uc,
            execution_repo,
        }
    }

    pub async fn trigger_job(
        &self,
        req: TriggerJobRequest,
    ) -> Result<TriggerJobResponse, GrpcError> {
        let job_id = req
            .job_id
            .parse::<uuid::Uuid>()
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid job_id: {}", req.job_id)))?;

        match self.trigger_job_uc.execute(&job_id).await {
            Ok(execution) => Ok(TriggerJobResponse {
                execution_id: execution.id.to_string(),
                job_id: execution.job_id.to_string(),
                status: execution.status,
                triggered_at: execution.started_at,
            }),
            Err(TriggerJobError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("job not found: {}", id)))
            }
            Err(TriggerJobError::NotActive(id)) => {
                Err(GrpcError::InvalidArgument(format!("job not active: {}", id)))
            }
            Err(TriggerJobError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }

    pub async fn get_job_execution(
        &self,
        req: GetJobExecutionRequest,
    ) -> Result<GetJobExecutionResponse, GrpcError> {
        let execution_id = req.execution_id.parse::<uuid::Uuid>().map_err(|_| {
            GrpcError::InvalidArgument(format!("invalid execution_id: {}", req.execution_id))
        })?;

        let execution = self
            .execution_repo
            .find_by_id(&execution_id)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?
            .ok_or_else(|| {
                GrpcError::NotFound(format!("execution not found: {}", execution_id))
            })?;

        let duration_ms = execution.completed_at.and_then(|finished_at| {
            let duration = finished_at - execution.started_at;
            if duration.num_milliseconds() >= 0 {
                Some(duration.num_milliseconds() as u64)
            } else {
                None
            }
        });

        Ok(GetJobExecutionResponse {
            id: execution.id.to_string(),
            job_id: execution.job_id.to_string(),
            status: normalize_status(&execution.status),
            triggered_by: "manual".to_string(),
            started_at: execution.started_at,
            finished_at: execution.completed_at,
            duration_ms,
            error_message: execution.error_message,
        })
    }
}

fn normalize_status(status: &str) -> String {
    match status {
        "completed" => "succeeded".to_string(),
        _ => status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_execution::SchedulerExecution;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_execution_repository::MockSchedulerExecutionRepository;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    fn make_service(
        mock_job: MockSchedulerJobRepository,
        mock_exec: MockSchedulerExecutionRepository,
    ) -> SchedulerGrpcService {
        let repo = Arc::new(mock_job);
        let exec_repo = Arc::new(mock_exec);
        SchedulerGrpcService::new(
            Arc::new(TriggerJobUseCase::new(repo, exec_repo.clone())),
            exec_repo,
        )
    }

    #[tokio::test]
    async fn test_trigger_job_success() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let job = SchedulerJob::new(
            "test-job".to_string(),
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

        let svc = make_service(mock_job, mock_exec);
        let req = TriggerJobRequest {
            job_id: job_id.to_string(),
        };
        let resp = svc.trigger_job(req).await.unwrap();

        assert_eq!(resp.job_id, job_id.to_string());
        assert_eq!(resp.status, "running");
    }

    #[tokio::test]
    async fn test_trigger_job_not_found() {
        let mut mock_job = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        mock_job.expect_find_by_id().returning(|_| Ok(None));

        let svc = make_service(mock_job, mock_exec);
        let req = TriggerJobRequest {
            job_id: uuid::Uuid::new_v4().to_string(),
        };
        let result = svc.trigger_job(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_trigger_job_invalid_id() {
        let mock_job = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        let svc = make_service(mock_job, mock_exec);
        let req = TriggerJobRequest {
            job_id: "not-a-uuid".to_string(),
        };
        let result = svc.trigger_job(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_job_execution_success() {
        let mock_job = MockSchedulerJobRepository::new();
        let mut mock_exec = MockSchedulerExecutionRepository::new();
        let execution_id = uuid::Uuid::new_v4();
        let job_id = uuid::Uuid::new_v4();
        mock_exec.expect_find_by_id().returning(move |_| {
            Ok(Some(SchedulerExecution {
                id: execution_id,
                job_id,
                status: "completed".to_string(),
                started_at: chrono::Utc::now() - chrono::Duration::seconds(2),
                completed_at: Some(chrono::Utc::now()),
                error_message: None,
            }))
        });
        let svc = make_service(mock_job, mock_exec);
        let req = GetJobExecutionRequest {
            execution_id: execution_id.to_string(),
        };
        let result = svc.get_job_execution(req).await.unwrap();

        assert_eq!(result.id, execution_id.to_string());
        assert_eq!(result.job_id, job_id.to_string());
        assert_eq!(result.status, "succeeded");
        assert!(result.duration_ms.is_some());
    }

    #[tokio::test]
    async fn test_get_job_execution_invalid_id() {
        let mock_job = MockSchedulerJobRepository::new();
        let mock_exec = MockSchedulerExecutionRepository::new();
        let svc = make_service(mock_job, mock_exec);
        let req = GetJobExecutionRequest {
            execution_id: "invalid-id".to_string(),
        };
        let result = svc.get_job_execution(req).await;

        assert!(matches!(result, Err(GrpcError::InvalidArgument(_))));
    }
}
