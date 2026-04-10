use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::{EvaluationMode, RuleSet};
use crate::domain::repository::RuleSetRepository;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, Clone)]
pub struct UpdateRuleSetInput {
    pub id: Uuid,
    pub description: Option<String>,
    pub evaluation_mode: Option<String>,
    pub default_result: Option<serde_json::Value>,
    pub rule_ids: Option<Vec<Uuid>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateRuleSetError {
    #[error("rule set not found: {0}")]
    NotFound(Uuid),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateRuleSetUseCase {
    repo: Arc<dyn RuleSetRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl UpdateRuleSetUseCase {
    #[allow(dead_code)]
    pub fn new(repo: Arc<dyn RuleSetRepository>) -> Self {
        Self {
            repo,
            event_publisher: Arc::new(NoopRuleEventPublisher),
        }
    }

    pub fn with_publisher(
        repo: Arc<dyn RuleSetRepository>,
        event_publisher: Arc<dyn RuleEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &UpdateRuleSetInput) -> Result<RuleSet, UpdateRuleSetError> {
        let mut rule_set = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateRuleSetError::Internal(e.to_string()))?
            .ok_or(UpdateRuleSetError::NotFound(input.id))?;

        if let Some(ref desc) = input.description {
            rule_set.description = desc.clone();
        }
        if let Some(ref mode) = input.evaluation_mode {
            rule_set.evaluation_mode = EvaluationMode::from_str(mode).ok_or_else(|| {
                UpdateRuleSetError::Validation(format!("invalid evaluation_mode: '{mode}'"))
            })?;
        }
        if let Some(ref default) = input.default_result {
            rule_set.default_result = default.clone();
        }
        if let Some(ref ids) = input.rule_ids {
            rule_set.rule_ids = ids.clone();
        }
        if let Some(enabled) = input.enabled {
            rule_set.enabled = enabled;
        }

        rule_set.updated_at = chrono::Utc::now();

        self.repo
            .update(&rule_set)
            .await
            .map_err(|e| UpdateRuleSetError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_set_updated(&rule_set))
            .await
        {
            tracing::warn!(error = %e, rule_set_id = %rule_set.id, "failed to publish rule set updated event");
        }

        Ok(rule_set)
    }
}
