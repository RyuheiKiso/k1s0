use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;
use crate::domain::service::condition_parser::ConditionParser;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, Clone)]
pub struct UpdateRuleInput {
    pub id: Uuid,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub when_condition: Option<serde_json::Value>,
    pub then_result: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateRuleError {
    #[error("rule not found: {0}")]
    NotFound(Uuid),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("invalid condition: {0}")]
    InvalidCondition(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateRuleUseCase {
    repo: Arc<dyn RuleRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl UpdateRuleUseCase {
    pub fn new(repo: Arc<dyn RuleRepository>) -> Self {
        Self {
            repo,
            event_publisher: Arc::new(NoopRuleEventPublisher),
        }
    }

    pub fn with_publisher(
        repo: Arc<dyn RuleRepository>,
        event_publisher: Arc<dyn RuleEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &UpdateRuleInput) -> Result<Rule, UpdateRuleError> {
        let mut rule = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateRuleError::Internal(e.to_string()))?
            .ok_or(UpdateRuleError::NotFound(input.id))?;

        if let Some(ref desc) = input.description {
            rule.description = desc.clone();
        }
        if let Some(priority) = input.priority {
            if priority < 1 || priority > 1000 {
                return Err(UpdateRuleError::Validation(
                    "priority must be between 1 and 1000".to_string(),
                ));
            }
            rule.priority = priority;
        }
        if let Some(ref when) = input.when_condition {
            ConditionParser::parse(when)
                .map_err(|e| UpdateRuleError::InvalidCondition(e))?;
            rule.when_condition = when.clone();
        }
        if let Some(ref then) = input.then_result {
            rule.then_result = then.clone();
        }
        if let Some(enabled) = input.enabled {
            rule.enabled = enabled;
        }

        rule.version += 1;
        rule.updated_at = chrono::Utc::now();

        self.repo
            .update(&rule)
            .await
            .map_err(|e| UpdateRuleError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_updated(&rule))
            .await
        {
            tracing::warn!(error = %e, rule_id = %rule.id, "failed to publish rule updated event");
        }

        Ok(rule)
    }
}
