use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;
use crate::domain::service::PolicyDomainService;
use crate::infrastructure::kafka_producer::{
    NoopPolicyEventPublisher, PolicyChangedEvent, PolicyEventPublisher,
};

#[derive(Debug, Clone)]
pub struct CreatePolicyInput {
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub package_path: String,
    pub bundle_id: Option<Uuid>,
    /// テナント ID: CRIT-005 対応。RLS によるテナント分離のために使用する。
    pub tenant_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreatePolicyError {
    #[error("policy already exists: {0}")]
    AlreadyExists(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreatePolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
    event_publisher: Arc<dyn PolicyEventPublisher>,
}

impl CreatePolicyUseCase {
    #[allow(dead_code)]
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self {
            repo,
            event_publisher: Arc::new(NoopPolicyEventPublisher),
        }
    }

    pub fn with_publisher(
        repo: Arc<dyn PolicyRepository>,
        event_publisher: Arc<dyn PolicyEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &CreatePolicyInput) -> Result<Policy, CreatePolicyError> {
        PolicyDomainService::validate_policy_name(&input.name)
            .map_err(CreatePolicyError::Validation)?;
        PolicyDomainService::validate_rego_content(&input.rego_content)
            .map_err(CreatePolicyError::Validation)?;

        // CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定する
        let exists = self
            .repo
            .exists_by_name(&input.name, &input.tenant_id)
            .await
            .map_err(|e| CreatePolicyError::Internal(e.to_string()))?;

        if exists {
            return Err(CreatePolicyError::AlreadyExists(input.name.clone()));
        }

        let mut policy = Policy::new(
            input.name.clone(),
            input.description.clone(),
            input.rego_content.clone(),
        );
        policy.package_path = PolicyDomainService::normalize_package_path(&input.package_path);
        policy.bundle_id = input.bundle_id;
        // CRIT-005 対応: リクエストのテナント ID をポリシーに設定する
        policy.tenant_id = input.tenant_id.clone();

        self.repo
            .create(&policy)
            .await
            .map_err(|e| CreatePolicyError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_policy_changed(&PolicyChangedEvent::created(&policy))
            .await
        {
            tracing::warn!(error = %e, policy_id = %policy.id, "failed to publish policy created event");
        }

        Ok(policy)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_exists_by_name()
            .withf(|name, _tenant_id| name == "allow-read")
            .returning(|_, _| Ok(false));
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreatePolicyUseCase::new(Arc::new(mock));
        let input = CreatePolicyInput {
            name: "allow-read".to_string(),
            description: "Allow read access".to_string(),
            rego_content: "package authz\ndefault allow = true".to_string(),
            package_path: "k1s0.system.authz".to_string(),
            bundle_id: Some(Uuid::new_v4()),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.name, "allow-read");
        assert_eq!(policy.package_path, "k1s0.system.authz");
        assert!(policy.bundle_id.is_some());
        assert_eq!(policy.version, 1);
        assert!(policy.enabled);
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_exists_by_name()
            .withf(|name, _tenant_id| name == "existing-policy")
            .returning(|_, _| Ok(true));

        let uc = CreatePolicyUseCase::new(Arc::new(mock));
        let input = CreatePolicyInput {
            name: "existing-policy".to_string(),
            description: "Existing".to_string(),
            rego_content: "package authz".to_string(),
            package_path: "k1s0.system.authz".to_string(),
            bundle_id: None,
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreatePolicyError::AlreadyExists(name) => assert_eq!(name, "existing-policy"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
