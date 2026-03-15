use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::scorecard::Scorecard;
use crate::domain::repository::ScorecardRepository;

/// GetScorecardError はスコアカード取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetScorecardError {
    #[error("scorecard not found for service: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetScorecardUseCase はスコアカード取得ユースケース。
pub struct GetScorecardUseCase {
    scorecard_repo: Arc<dyn ScorecardRepository>,
}

impl GetScorecardUseCase {
    pub fn new(scorecard_repo: Arc<dyn ScorecardRepository>) -> Self {
        Self { scorecard_repo }
    }

    pub async fn execute(&self, service_id: Uuid) -> Result<Scorecard, GetScorecardError> {
        match self.scorecard_repo.get(service_id).await {
            Ok(Some(scorecard)) => Ok(scorecard),
            Ok(None) => Err(GetScorecardError::NotFound(service_id)),
            Err(e) => Err(GetScorecardError::Internal(e.to_string())),
        }
    }
}
