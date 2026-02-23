use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::ProvisioningJob;
use crate::domain::repository::MemberRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetProvisioningStatusError {
    #[error("job not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetProvisioningStatusUseCase {
    member_repo: Arc<dyn MemberRepository>,
}

impl GetProvisioningStatusUseCase {
    pub fn new(member_repo: Arc<dyn MemberRepository>) -> Self {
        Self { member_repo }
    }

    pub async fn execute(
        &self,
        job_id: Uuid,
    ) -> Result<ProvisioningJob, GetProvisioningStatusError> {
        self.member_repo
            .find_job(&job_id)
            .await
            .map_err(|e| GetProvisioningStatusError::Internal(e.to_string()))?
            .ok_or_else(|| GetProvisioningStatusError::NotFound(job_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::ProvisioningStatus;
    use crate::domain::repository::member_repository::MockMemberRepository;

    #[tokio::test]
    async fn test_get_provisioning_status_found() {
        let mut mock = MockMemberRepository::new();
        let job_id = Uuid::new_v4();
        let jid = job_id;
        mock.expect_find_job()
            .withf(move |id| *id == jid)
            .returning(move |_| {
                Ok(Some(ProvisioningJob {
                    id: job_id,
                    tenant_id: Uuid::new_v4(),
                    status: ProvisioningStatus::Running,
                    current_step: Some("creating_realm".to_string()),
                    error_message: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = GetProvisioningStatusUseCase::new(Arc::new(mock));
        let job = uc.execute(job_id).await.unwrap();
        assert_eq!(job.id, job_id);
        assert_eq!(job.status, ProvisioningStatus::Running);
    }

    #[tokio::test]
    async fn test_get_provisioning_status_not_found() {
        let mut mock = MockMemberRepository::new();
        mock.expect_find_job().returning(|_| Ok(None));

        let uc = GetProvisioningStatusUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetProvisioningStatusError::NotFound(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
