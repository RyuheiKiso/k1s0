use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::SchedulerJobRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteJobError {
    #[error("job not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteJobUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl DeleteJobUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteJobError> {
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
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteJobUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteJobUseCase::new(Arc::new(mock));
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
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteJobUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteJobError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
