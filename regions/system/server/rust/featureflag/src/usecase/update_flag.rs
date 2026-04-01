use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::entity::feature_flag::{FlagRule, FlagVariant};
use crate::domain::entity::flag_audit_log::FlagAuditLog;
use crate::domain::repository::{FeatureFlagRepository, FlagAuditLogRepository};
use crate::domain::service::FeatureFlagDomainService;
use crate::infrastructure::kafka_producer::FlagEventPublisher;
use crate::usecase::watch_feature_flag::FeatureFlagChangeEvent;

/// UpdateFlagInput はフィーチャーフラグ更新の入力データ。
/// STATIC-CRITICAL-001 監査対応: tenant_id でテナントスコープを指定する。
#[derive(Debug, Clone)]
pub struct UpdateFlagInput {
    pub tenant_id: Uuid,
    pub flag_key: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub variants: Option<Vec<FlagVariant>>,
    pub rules: Option<Vec<FlagRule>>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateFlagError {
    #[error("flag not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
    event_publisher: Arc<dyn FlagEventPublisher>,
    audit_repo: Arc<dyn FlagAuditLogRepository>,
    watch_sender: Option<tokio::sync::broadcast::Sender<FeatureFlagChangeEvent>>,
}

impl UpdateFlagUseCase {
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

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを更新する。
    pub async fn execute(&self, input: &UpdateFlagInput) -> Result<FeatureFlag, UpdateFlagError> {
        let mut flag = self
            .repo
            .find_by_key(input.tenant_id, &input.flag_key)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    UpdateFlagError::NotFound(input.flag_key.clone())
                } else {
                    UpdateFlagError::Internal(msg)
                }
            })?;
        let before = serde_json::json!({
            "flag_key": flag.flag_key,
            "description": flag.description,
            "enabled": flag.enabled,
            "variants": flag.variants,
            "rules": flag.rules,
        });

        if let Some(enabled) = input.enabled {
            flag.enabled = enabled;
        }
        if let Some(ref description) = input.description {
            flag.description = description.clone();
        }
        if let Some(ref variants) = input.variants {
            FeatureFlagDomainService::validate_variants(variants)
                .map_err(UpdateFlagError::Internal)?;
            flag.variants = variants.clone();
        }
        if let Some(ref rules) = input.rules {
            flag.rules = rules.clone();
        }
        flag.updated_at = chrono::Utc::now();

        self.repo
            .update(input.tenant_id, &flag)
            .await
            .map_err(|e| UpdateFlagError::Internal(e.to_string()))?;

        let after = serde_json::json!({
            "flag_key": flag.flag_key,
            "description": flag.description,
            "enabled": flag.enabled,
            "variants": flag.variants,
            "rules": flag.rules,
        });
        self.audit_repo
            .create(&FlagAuditLog::new(
                input.tenant_id,
                flag.id,
                flag.flag_key.clone(),
                "UPDATED".to_string(),
                Some(before.clone()),
                Some(after.clone()),
                "system".to_string(),
            ))
            .await
            .map_err(|e| UpdateFlagError::Internal(e.to_string()))?;

        self.event_publisher
            .publish_flag_changed(&flag.flag_key, flag.enabled, None, Some(before), after)
            .await
            .map_err(|e| UpdateFlagError::Internal(e.to_string()))?;

        if let Some(sender) = &self.watch_sender {
            let _ = sender.send(FeatureFlagChangeEvent {
                flag_key: flag.flag_key.clone(),
                change_type: "UPDATED".to_string(),
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
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::flag_audit_log_repository::MockFlagAuditLogRepository;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::infrastructure::kafka_producer::MockFlagEventPublisher;
    use chrono::Utc;

    /// システムテナントUUID: テスト共通
    fn system_tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn make_flag(flag_key: &str, enabled: bool) -> FeatureFlag {
        FeatureFlag {
            id: Uuid::new_v4(),
            tenant_id: system_tenant(),
            flag_key: flag_key.to_string(),
            description: "Dark mode".to_string(),
            enabled,
            variants: vec![],
            rules: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockFeatureFlagRepository::new();
        let return_flag = make_flag("dark-mode", true);

        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "dark-mode")
            .returning(move |_, _| Ok(return_flag.clone()));
        mock.expect_update().returning(|_, _| Ok(()));
        let mut mock_audit_repo = MockFlagAuditLogRepository::new();
        mock_audit_repo.expect_create().returning(|_| Ok(()));
        let mut mock_publisher = MockFlagEventPublisher::new();
        mock_publisher
            .expect_publish_flag_changed()
            .withf(|key, enabled, actor_user_id, before, after| {
                key == "dark-mode"
                    && !*enabled
                    && actor_user_id.is_none()
                    && before.is_some()
                    && after["enabled"] == false
            })
            .returning(|_, _, _, _, _| Ok(()));

        let uc = UpdateFlagUseCase::new(
            Arc::new(mock),
            Arc::new(mock_publisher),
            Arc::new(mock_audit_repo),
        );
        let input = UpdateFlagInput {
            tenant_id: system_tenant(),
            flag_key: "dark-mode".to_string(),
            enabled: Some(false),
            description: Some("Updated dark mode".to_string()),
            variants: None,
            rules: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.flag_key, "dark-mode");
        assert!(!updated.enabled);
        assert_eq!(updated.description, "Updated dark mode");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_by_key()
            .returning(|_, _| Err(anyhow::anyhow!("flag not found")));

        let mock_audit_repo = MockFlagAuditLogRepository::new();
        let uc = UpdateFlagUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopFlagEventPublisher),
            Arc::new(mock_audit_repo),
        );
        let input = UpdateFlagInput {
            tenant_id: system_tenant(),
            flag_key: "nonexistent".to_string(),
            enabled: Some(true),
            description: None,
            variants: None,
            rules: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateFlagError::NotFound(key) => assert_eq!(key, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
