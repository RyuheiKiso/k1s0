use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::RuleSetVersion;
use crate::domain::repository::{RuleSetRepository, RuleSetVersionRepository};
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug)]
pub struct PublishRuleSetOutput {
    pub id: Uuid,
    pub name: String,
    pub published_version: u32,
    pub previous_version: u32,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum PublishRuleSetError {
    #[error("rule set not found: {0}")]
    NotFound(Uuid),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct PublishRuleSetUseCase {
    rule_set_repo: Arc<dyn RuleSetRepository>,
    version_repo: Arc<dyn RuleSetVersionRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl PublishRuleSetUseCase {
    #[allow(dead_code)]
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

    pub async fn execute(&self, id: &Uuid) -> Result<PublishRuleSetOutput, PublishRuleSetError> {
        let mut rule_set = self
            .rule_set_repo
            .find_by_id(id)
            .await
            .map_err(|e| PublishRuleSetError::Internal(e.to_string()))?
            .ok_or(PublishRuleSetError::NotFound(*id))?;

        let previous_version = rule_set.current_version;
        let new_version = previous_version + 1;

        let version = RuleSetVersion::new(
            rule_set.id,
            new_version,
            rule_set.rule_ids.clone(),
            rule_set.default_result.clone(),
            "system".to_string(),
        );

        self.version_repo
            .create(&version)
            .await
            .map_err(|e| PublishRuleSetError::Internal(e.to_string()))?;

        rule_set.current_version = new_version;
        rule_set.updated_at = chrono::Utc::now();

        self.rule_set_repo
            .update(&rule_set)
            .await
            .map_err(|e| PublishRuleSetError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_set_published(
                &rule_set,
                new_version,
                previous_version,
            ))
            .await
        {
            tracing::warn!(error = %e, "failed to publish rule set published event");
        }

        Ok(PublishRuleSetOutput {
            id: rule_set.id,
            name: rule_set.name,
            published_version: new_version,
            previous_version,
            published_at: version.published_at,
        })
    }
}
