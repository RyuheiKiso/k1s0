use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::RuleSet;
use crate::domain::repository::RuleSetRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetRuleSetError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetRuleSetUseCase {
    repo: Arc<dyn RuleSetRepository>,
}

impl GetRuleSetUseCase {
    pub fn new(repo: Arc<dyn RuleSetRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<Option<RuleSet>, GetRuleSetError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetRuleSetError::Internal(e.to_string()))
    }
}
