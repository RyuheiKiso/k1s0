use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::TeamRepository;

/// `DeleteTeamError` はチーム削除に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum DeleteTeamError {
    #[error("team not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `DeleteTeamUseCase` はチーム削除ユースケース。
pub struct DeleteTeamUseCase {
    team_repo: Arc<dyn TeamRepository>,
}

impl DeleteTeamUseCase {
    pub fn new(team_repo: Arc<dyn TeamRepository>) -> Self {
        Self { team_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), DeleteTeamError> {
        let deleted = self
            .team_repo
            .delete(id)
            .await
            .map_err(|e| DeleteTeamError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteTeamError::NotFound(id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::team_repository::MockTeamRepository;

    #[tokio::test]
    async fn test_delete_team_success() {
        let mut mock = MockTeamRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteTeamUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_team_not_found() {
        let mut mock = MockTeamRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteTeamUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteTeamError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_team_repo_error() {
        let mut mock = MockTeamRepository::new();
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeleteTeamUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteTeamError::Internal(_))));
    }
}
