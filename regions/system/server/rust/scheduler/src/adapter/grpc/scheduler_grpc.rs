use std::sync::Arc;

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
}

impl SchedulerGrpcService {
    pub fn new(trigger_job_uc: Arc<TriggerJobUseCase>) -> Self {
        Self { trigger_job_uc }
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
        // Execution retrieval requires a dedicated execution repository.
        // Return Unimplemented until persistence layer is added.
        let _ = req;
        Err(GrpcError::Unimplemented(
            "get_job_execution is not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    fn make_service(mock: MockSchedulerJobRepository) -> SchedulerGrpcService {
        let repo = Arc::new(mock);
        SchedulerGrpcService::new(Arc::new(TriggerJobUseCase::new(repo)))
    }

    #[tokio::test]
    async fn test_trigger_job_success() {
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
        mock.expect_update().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = TriggerJobRequest {
            job_id: job_id.to_string(),
        };
        let resp = svc.trigger_job(req).await.unwrap();

        assert_eq!(resp.job_id, job_id.to_string());
        assert_eq!(resp.status, "running");
    }

    #[tokio::test]
    async fn test_trigger_job_not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let svc = make_service(mock);
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
        let mock = MockSchedulerJobRepository::new();
        let svc = make_service(mock);
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
    async fn test_get_job_execution_unimplemented() {
        let mock = MockSchedulerJobRepository::new();
        let svc = make_service(mock);
        let req = GetJobExecutionRequest {
            execution_id: "exec-001".to_string(),
        };
        let result = svc.get_job_execution(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::Unimplemented(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
