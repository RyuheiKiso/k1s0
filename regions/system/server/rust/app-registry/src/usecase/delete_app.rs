use std::sync::Arc;

use crate::domain::repository::AppRepository;

/// アプリ削除ユースケースで発生するエラー
#[derive(Debug, thiserror::Error)]
pub enum DeleteAppError {
    /// 指定した ID のアプリが存在しない
    #[error("app not found: {0}")]
    NotFound(String),
    /// 内部エラー（DB 障害等）
    #[error("internal error: {0}")]
    Internal(String),
}

/// アプリを削除するユースケース
pub struct DeleteAppUseCase {
    repo: Arc<dyn AppRepository>,
}

impl DeleteAppUseCase {
    pub fn new(repo: Arc<dyn AppRepository>) -> Self {
        Self { repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(&self, tenant_id: &str, id: &str) -> Result<(), DeleteAppError> {
        let deleted = self
            .repo
            .delete(tenant_id, id)
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
        mock.expect_delete().returning(|_, _| Ok(true));

        let uc = DeleteAppUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1", "app-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_app_not_found() {
        let mut mock = MockAppRepository::new();
        mock.expect_delete().returning(|_, _| Ok(false));

        let uc = DeleteAppUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1", "missing").await;
        assert!(matches!(result, Err(DeleteAppError::NotFound(_))));
    }
}
