use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repository::RuleSetRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteRuleSetError {
    #[error("rule set not found: {0}")]
    NotFound(Uuid),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteRuleSetUseCase {
    repo: Arc<dyn RuleSetRepository>,
}

impl DeleteRuleSetUseCase {
    pub fn new(repo: Arc<dyn RuleSetRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteRuleSetError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteRuleSetError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteRuleSetError::NotFound(*id));
        }

        Ok(())
    }
}
