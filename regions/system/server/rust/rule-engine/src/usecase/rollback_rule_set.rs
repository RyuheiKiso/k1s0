use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repository::{RuleSetRepository, RuleSetVersionRepository};
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug)]
pub struct RollbackRuleSetOutput {
    pub id: Uuid,
    pub name: String,
    pub rolled_back_to_version: u32,
    pub previous_version: u32,
    pub rolled_back_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RollbackRuleSetError {
    #[error("rule set not found: {0}")]
    NotFound(Uuid),
    #[error("no previous version to rollback: current version is {0}")]
    NoPreviousVersion(u32),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RollbackRuleSetUseCase {
    rule_set_repo: Arc<dyn RuleSetRepository>,
    version_repo: Arc<dyn RuleSetVersionRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl RollbackRuleSetUseCase {
    pub fn new(
        rule_set_repo: Arc<dyn RuleSetRepository>,
        version_repo: Arc<dyn RuleSetVersionRepository>,
    ) -> Self {
        Self {
            rule_set_repo,
            version_repo,
            event_publisher: Arc::new(NoopRuleEventPublisher),
        }
    }

    pub fn with_publisher(
        rule_set_repo: Arc<dyn RuleSetRepository>,
        version_repo: Arc<dyn RuleSetVersionRepository>,
        event_publisher: Arc<dyn RuleEventPublisher>,
    ) -> Self {
        Self {
            rule_set_repo,
            version_repo,
            event_publisher,
        }
    }

    pub async fn execute(
        &self,
        id: &Uuid,
    ) -> Result<RollbackRuleSetOutput, RollbackRuleSetError> {
        let mut rule_set = self
            .rule_set_repo
            .find_by_id(id)
            .await
            .map_err(|e| RollbackRuleSetError::Internal(e.to_string()))?
            .ok_or(RollbackRuleSetError::NotFound(*id))?;

        let current_version = rule_set.current_version;
        if current_version <= 1 {
            return Err(RollbackRuleSetError::NoPreviousVersion(current_version));
        }

        let target_version = current_version - 1;
        let prev_ver = self
            .version_repo
            .find_by_rule_set_id_and_version(&rule_set.id, target_version)
            .await
            .map_err(|e| RollbackRuleSetError::Internal(e.to_string()))?
            .ok_or(RollbackRuleSetError::NoPreviousVersion(current_version))?;

        rule_set.rule_ids = prev_ver.rule_ids_snapshot;
        rule_set.default_result = prev_ver.default_result_snapshot;
        rule_set.current_version = target_version;
        rule_set.updated_at = chrono::Utc::now();

        self.rule_set_repo
            .update(&rule_set)
            .await
            .map_err(|e| RollbackRuleSetError::Internal(e.to_string()))?;

        let now = chrono::Utc::now();

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_set_rolled_back(
                &rule_set,
                target_version,
                current_version,
            ))
            .await
        {
            tracing::warn!(error = %e, "failed to publish rule set rolled back event");
        }

        Ok(RollbackRuleSetOutput {
            id: rule_set.id,
            name: rule_set.name,
            rolled_back_to_version: target_version,
            previous_version: current_version,
            rolled_back_at: now,
        })
    }
}
