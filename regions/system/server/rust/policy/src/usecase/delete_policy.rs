use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::PolicyRepository;
use crate::infrastructure::kafka_producer::{
    NoopPolicyEventPublisher, PolicyChangedEvent, PolicyEventPublisher,
};

#[derive(Debug, thiserror::Error)]
pub enum DeletePolicyError {
    #[error("policy not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeletePolicyUseCase {
    repo: Arc<dyn PolicyRepository>,
    event_publisher: Arc<dyn PolicyEventPublisher>,
}

impl DeletePolicyUseCase {
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

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeletePolicyError> {
        let before = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| DeletePolicyError::Internal(e.to_string()))?;
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeletePolicyError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeletePolicyError::NotFound(*id));
        }

        if let Some(before_policy) = before {
            if let Err(e) = self
                .event_publisher
                .publish_policy_changed(&PolicyChangedEvent::deleted(&before_policy))
                .await
            {
                tracing::warn!(error = %e, policy_id = %id, "failed to publish policy deleted event");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    #[tokio::test]
    async fn success() {
        let id = Uuid::new_v4();
        let policy = crate::domain::entity::policy::Policy {
            id,
            name: "test".to_string(),
            description: "test".to_string(),
            rego_content: "package test".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(policy.clone())));
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeletePolicyUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeletePolicyUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeletePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = DeletePolicyUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeletePolicyError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
