use std::sync::Arc;

use crate::domain::repository::AppRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteAppError {
    #[error("app not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteAppUseCase {
    repo: Arc<dyn AppRepository>,
}

impl DeleteAppUseCase {
    pub fn new(repo: Arc<dyn AppRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DeleteAppError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteAppError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteAppError::NotFound(id.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    #[tokio::test]
    async fn test_delete_app_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteAppUseCase::new(Arc::new(mock));
        let result = uc.execute("app-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_app_not_found() {
        let mut mock = MockAppRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteAppUseCase::new(Arc::new(mock));
        let result = uc.execute("missing").await;
        assert!(matches!(result, Err(DeleteAppError::NotFound(_))));
    }
}
