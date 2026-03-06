use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetRuleError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetRuleUseCase {
    repo: Arc<dyn RuleRepository>,
}

impl GetRuleUseCase {
    pub fn new(repo: Arc<dyn RuleRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<Option<Rule>, GetRuleError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetRuleError::Internal(e.to_string()))
    }
}
