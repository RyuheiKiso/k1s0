use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flag_audit_log::FlagAuditLog;
use crate::domain::repository::{FeatureFlagRepository, FlagAuditLogRepository};
use crate::infrastructure::kafka_producer::FlagEventPublisher;
use crate::usecase::watch_feature_flag::FeatureFlagChangeEvent;

#[derive(Debug, thiserror::Error)]
pub enum DeleteFlagError {
    #[error("flag not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
    event_publisher: Arc<dyn FlagEventPublisher>,
    audit_repo: Arc<dyn FlagAuditLogRepository>,
    watch_sender: Option<tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>>,
}

impl DeleteFlagUseCase {
    pub fn new(
        repo: Arc<dyn FeatureFlagRepository>,
        event_publisher: Arc<dyn FlagEventPublisher>,
        audit_repo: Arc<dyn FlagAuditLogRepository>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
            audit_repo,
            watch_sender: None,
        }
    }

    pub fn with_watch_sender(
        mut self,
        sender: tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>,
    ) -> Self {
        self.watch_sender = Some(sender);
        self
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteFlagError> {
        let flags = self
            .repo
            .find_all()
            .await
            .map_err(|e| DeleteFlagError::Internal(e.to_string()))?;
        let target = flags
            .into_iter()
            .find(|f| f.id == *id)
            .ok_or(DeleteFlagError::NotFound(*id))?;
        let before = serde_json::json!({
            "flag_key": target.flag_key,
            "description": target.description,
            "enabled": target.enabled,
            "variants": target.variants,
            "rules": target.rules,
        });

        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteFlagError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteFlagError::NotFound(*id));
        }

        self.audit_repo
            .create(&FlagAuditLog::new(
                target.id,
                target.flag_key.clone(),
                "DELETED".to_string(),
                Some(before.clone()),
                None,
                "system".to_string(),
            ))
            .await
            .map_err(|e| DeleteFlagError::Internal(e.to_string()))?;

        self.event_publisher
            .publish_flag_changed(
                &target.flag_key,
                false,
                None,
                Some(before),
                serde_json::json!({
                    "flag_key": target.flag_key,
                    "action": "DELETED"
                }),
            )
            .await
            .map_err(|e| DeleteFlagError::Internal(e.to_string()))?;

        if let Some(sender) = &self.watch_sender {
            let _ = sender.send(FeatureFlagChangeEvent {
                flag_key: target.flag_key.clone(),
                change_type: "DELETED".to_string(),
                enabled: target.enabled,
                description: target.description.clone(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::flag_audit_log_repository::MockFlagAuditLogRepository;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::infrastructure::kafka_producer::MockFlagEventPublisher;

    #[tokio::test]
    async fn success() {
        let target_id = Uuid::new_v4();
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_all().returning(move || {
            let mut f = FeatureFlag::new(
                "feature.delete".to_string(),
                "delete target".to_string(),
                true,
            );
            f.id = target_id;
            Ok(vec![f])
        });
        mock.expect_delete().returning(|_| Ok(true));
        let mut mock_publisher = MockFlagEventPublisher::new();
        mock_publisher
            .expect_publish_flag_changed()
            .returning(|_, _, _, _, _| Ok(()));
        let mut mock_audit_repo = MockFlagAuditLogRepository::new();
        mock_audit_repo.expect_create().returning(|_| Ok(()));

        let uc = DeleteFlagUseCase::new(
            Arc::new(mock),
            Arc::new(mock_publisher),
            Arc::new(mock_audit_repo),
        );
        let id = target_id;
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_all().returning(|| Ok(vec![]));
        let mock_publisher = MockFlagEventPublisher::new();
        let mock_audit_repo = MockFlagAuditLogRepository::new();
        let uc = DeleteFlagUseCase::new(
            Arc::new(mock),
            Arc::new(mock_publisher),
            Arc::new(mock_audit_repo),
        );
        let id = Uuid::new_v4();
        let result = uc.execute(&id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteFlagError::NotFound(found_id) => assert_eq!(found_id, id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFeatureFlagRepository::new();
        let target_id = Uuid::new_v4();
        mock.expect_find_all().returning(move || {
            let mut f = FeatureFlag::new(
                "feature.delete".to_string(),
                "delete target".to_string(),
                true,
            );
            f.id = target_id;
            Ok(vec![f])
        });
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("db error")));
        let mock_publisher = MockFlagEventPublisher::new();
        let mock_audit_repo = MockFlagAuditLogRepository::new();

        let uc = DeleteFlagUseCase::new(
            Arc::new(mock),
            Arc::new(mock_publisher),
            Arc::new(mock_audit_repo),
        );
        let result = uc.execute(&target_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteFlagError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
