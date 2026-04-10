use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::{EvaluationMode, RuleSet};
use crate::domain::repository::RuleSetRepository;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, Clone)]
pub struct CreateRuleSetInput {
    /// CRITICAL-RUST-001 監査対応: テナント分離のために追加。JWT/ヘッダーから抽出したテナント ID を渡す。
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub evaluation_mode: String,
    pub default_result: serde_json::Value,
    pub rule_ids: Vec<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateRuleSetError {
    #[error("rule set already exists: {0}")]
    AlreadyExists(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateRuleSetUseCase {
    repo: Arc<dyn RuleSetRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl CreateRuleSetUseCase {
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

    pub async fn execute(&self, input: &CreateRuleSetInput) -> Result<RuleSet, CreateRuleSetError> {
        if input.name.is_empty() {
            return Err(CreateRuleSetError::Validation(
                "name is required".to_string(),
            ));
        }
        if input.domain.is_empty() {
            return Err(CreateRuleSetError::Validation(
                "domain is required".to_string(),
            ));
        }

        let evaluation_mode =
            EvaluationMode::from_str(&input.evaluation_mode).ok_or_else(|| {
                CreateRuleSetError::Validation(format!(
                    "invalid evaluation_mode: '{}', expected 'first_match' or 'all_match'",
                    input.evaluation_mode
                ))
            })?;

        let exists = self
            .repo
            .exists_by_name(&input.name)
            .await
            .map_err(|e| CreateRuleSetError::Internal(e.to_string()))?;

        if exists {
            return Err(CreateRuleSetError::AlreadyExists(input.name.clone()));
        }

        let rule_set = RuleSet::new(
            input.tenant_id.clone(),
            input.name.clone(),
            input.description.clone(),
            input.domain.clone(),
            evaluation_mode,
            input.default_result.clone(),
            input.rule_ids.clone(),
        );

        self.repo
            .create(&rule_set)
            .await
            .map_err(|e| CreateRuleSetError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_set_created(&rule_set))
            .await
        {
            tracing::warn!(error = %e, rule_set_id = %rule_set.id, "failed to publish rule set created event");
        }

        Ok(rule_set)
    }
}
