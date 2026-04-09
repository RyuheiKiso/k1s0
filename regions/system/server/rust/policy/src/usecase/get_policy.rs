use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetPolicyError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetPolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl GetPolicyUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからポリシーを取得する。
    pub async fn execute(
        &self,
        id: &Uuid,
        tenant_id: &str,
    ) -> Result<Option<Policy>, GetPolicyError> {
        self.repo
            .find_by_id(id, tenant_id)
            .await
            .map_err(|e| GetPolicyError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn found() {
        let id = Uuid::new_v4();
        let id_clone = id;
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .withf(move |i, _tenant_id| *i == id_clone)
            .returning(move |_, _| {
                Ok(Some(Policy {
                    id,
                    name: "test-policy".to_string(),
                    description: "Test".to_string(),
                    rego_content: "package test".to_string(),
                    package_path: String::new(),
                    bundle_id: None,
                    version: 1,
                    enabled: true,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    tenant_id: "tenant-a".to_string(),
                }))
            });

        let uc = GetPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute(&id, "tenant-a").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetPolicyUseCase::new(Arc::new(mock));
        let result = uc.execute(&id, "tenant-a").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
