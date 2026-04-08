use std::sync::Arc;

use crate::domain::entity::feature_flag::{FeatureFlag, FlagVariant};
use crate::domain::entity::flag_audit_log::FlagAuditLog;
use crate::domain::repository::{FeatureFlagRepository, FlagAuditLogRepository};
use crate::domain::service::FeatureFlagDomainService;
use crate::infrastructure::kafka_producer::FlagEventPublisher;
use crate::usecase::watch_feature_flag::FeatureFlagChangeEvent;

/// `CreateFlagInput` はフィーチャーフラグ作成の入力データ。
/// STATIC-CRITICAL-001 監査対応: `tenant_id` でテナントスコープを指定する。
/// HIGH-005 対応: `tenant_id` は String 型（migration 006 で DB の TEXT 型に変更済み）。
#[derive(Debug, Clone)]
pub struct CreateFlagInput {
    pub tenant_id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateFlagError {
    #[error("flag already exists: {0}")]
    AlreadyExists(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
    event_publisher: Arc<dyn FlagEventPublisher>,
    audit_repo: Arc<dyn FlagAuditLogRepository>,
    watch_sender: Option<tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>>,
}

impl CreateFlagUseCase {
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

    #[must_use] 
    pub fn with_watch_sender(
        mut self,
        sender: tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>,
    ) -> Self {
        self.watch_sender = Some(sender);
        self
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを作成する。
    pub async fn execute(&self, input: &CreateFlagInput) -> Result<FeatureFlag, CreateFlagError> {
        FeatureFlagDomainService::validate_flag_key(&input.flag_key)
            .map_err(CreateFlagError::Internal)?;
        FeatureFlagDomainService::validate_variants(&input.variants)
            .map_err(CreateFlagError::Internal)?;

        let exists = self
            .repo
            .exists_by_key(&input.tenant_id, &input.flag_key)
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        if exists {
            return Err(CreateFlagError::AlreadyExists(input.flag_key.clone()));
        }

        let mut flag = FeatureFlag::new(
            input.tenant_id.clone(),
            input.flag_key.clone(),
            input.description.clone(),
            input.enabled,
        );
        flag.variants = input.variants.clone();

        self.repo
            .create(&input.tenant_id, &flag)
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        let after = serde_json::json!({
            "flag_key": flag.flag_key,
            "description": flag.description,
            "enabled": flag.enabled,
            "variants": flag.variants,
            "rules": flag.rules,
        });
        self.audit_repo
            .create(&FlagAuditLog::new(
                input.tenant_id.clone(),
                flag.id,
                flag.flag_key.clone(),
                "CREATED".to_string(),
                None,
                Some(after.clone()),
                "system".to_string(),
            ))
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        self.event_publisher
            .publish_flag_changed(&flag.flag_key, flag.enabled, None, None, after)
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        if let Some(sender) = &self.watch_sender {
            let _ = sender.send(FeatureFlagChangeEvent {
                flag_key: flag.flag_key.clone(),
                change_type: "CREATED".to_string(),
                enabled: flag.enabled,
                description: flag.description.clone(),
            });
        }

        Ok(flag)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::flag_audit_log_repository::MockFlagAuditLogRepository;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::infrastructure::kafka_producer::MockFlagEventPublisher;

    /// システムテナント文字列: テスト共通（HIGH-005 対応: TEXT 型）
    fn system_tenant() -> String {
        "00000000-0000-0000-0000-000000000001".to_string()
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_exists_by_key()
            .withf(|_tid, key| key == "new-feature")
            .returning(|_, _| Ok(false));
        mock.expect_create().returning(|_, _| Ok(()));
        let mut mock_audit_repo = MockFlagAuditLogRepository::new();
        mock_audit_repo.expect_create().returning(|_| Ok(()));
        let mut mock_publisher = MockFlagEventPublisher::new();
        mock_publisher
            .expect_publish_flag_changed()
            .withf(|key, enabled, actor_user_id, before, after| {
                key == "new-feature"
                    && *enabled
                    && actor_user_id.is_none()
                    && before.is_none()
                    && after["enabled"] == true
            })
            .returning(|_, _, _, _, _| Ok(()));

        let uc = CreateFlagUseCase::new(
            Arc::new(mock),
            Arc::new(mock_publisher),
            Arc::new(mock_audit_repo),
        );
        let input = CreateFlagInput {
            tenant_id: system_tenant(),
            flag_key: "new-feature".to_string(),
            description: "A new feature".to_string(),
            enabled: true,
            variants: vec![FlagVariant {
                name: "on".to_string(),
                value: "true".to_string(),
                weight: 100,
            }],
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let flag = result.unwrap();
        assert_eq!(flag.flag_key, "new-feature");
        assert!(flag.enabled);
        assert_eq!(flag.variants.len(), 1);
        assert_eq!(flag.variants[0].name, "on");
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_exists_by_key()
            .withf(|_tid, key| key == "existing-feature")
            .returning(|_, _| Ok(true));

        let mock_audit_repo = MockFlagAuditLogRepository::new();
        let uc = CreateFlagUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopFlagEventPublisher),
            Arc::new(mock_audit_repo),
        );
        let input = CreateFlagInput {
            tenant_id: system_tenant(),
            flag_key: "existing-feature".to_string(),
            description: "Existing".to_string(),
            enabled: true,
            variants: vec![],
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateFlagError::AlreadyExists(key) => assert_eq!(key, "existing-feature"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
