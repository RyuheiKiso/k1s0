use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::domain::repository::PolicyBundleRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetBundleError {
    #[error("bundle not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetBundleUseCase {
    repo: Arc<dyn PolicyBundleRepository>,
}

impl GetBundleUseCase {
    pub fn new(repo: Arc<dyn PolicyBundleRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからバンドルを取得する。
    pub async fn execute(
        &self,
        id: &Uuid,
        tenant_id: &str,
    ) -> Result<PolicyBundle, GetBundleError> {
        self.repo
            .find_by_id(id, tenant_id)
            .await
            .map_err(|e| GetBundleError::Internal(e.to_string()))?
            .ok_or_else(|| GetBundleError::NotFound(id.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::policy_bundle::PolicyBundle;
    use crate::domain::repository::bundle_repository::MockPolicyBundleRepository;
    use chrono::Utc;

    fn make_bundle(id: Uuid) -> PolicyBundle {
        PolicyBundle {
            id,
            name: "bundle-1".to_string(),
            description: Some("desc".to_string()),
            enabled: true,
            policy_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tenant_id: "tenant-a".to_string(),
        }
    }

    #[tokio::test]
    async fn success() {
        let id = Uuid::new_v4();
        let bundle = make_bundle(id);

        let mut mock = MockPolicyBundleRepository::new();
        mock.expect_find_by_id()
            .withf(move |given, _tenant_id| *given == id)
            .returning(move |_, _| Ok(Some(bundle.clone())));

        let uc = GetBundleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id, "tenant-a").await.unwrap();
        assert_eq!(result.id, id);
    }

    #[tokio::test]
    async fn not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockPolicyBundleRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetBundleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id, "tenant-a").await;
        assert!(matches!(result, Err(GetBundleError::NotFound(_))));
    }
}
