use std::sync::Arc;

use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::domain::repository::PolicyBundleRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListBundlesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListBundlesUseCase {
    repo: Arc<dyn PolicyBundleRepository>,
}

impl ListBundlesUseCase {
    pub fn new(repo: Arc<dyn PolicyBundleRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してからバンドル一覧を取得する。
    pub async fn execute(&self, tenant_id: &str) -> Result<Vec<PolicyBundle>, ListBundlesError> {
        self.repo
            .find_all(tenant_id)
            .await
            .map_err(|e| ListBundlesError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::bundle_repository::MockPolicyBundleRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockPolicyBundleRepository::new();
        mock.expect_find_all().returning(|_| Ok(vec![]));

        let uc = ListBundlesUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-a").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockPolicyBundleRepository::new();
        mock.expect_find_all()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ListBundlesUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-a").await;
        assert!(result.is_err());
    }
}
