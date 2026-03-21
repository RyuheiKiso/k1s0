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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::scorecard_repository::MockScorecardRepository;
    use chrono::Utc;

    fn sample_scorecard(service_id: Uuid) -> Scorecard {
        Scorecard {
            service_id,
            documentation_score: 80.0,
            test_coverage_score: 75.0,
            slo_compliance_score: 90.0,
            security_score: 85.0,
            overall_score: 82.5,
            evaluated_at: Utc::now(),
        }
    }

    /// スコアカードが存在する場合は取得できる
    #[tokio::test]
    async fn found() {
        let id = Uuid::new_v4();
        let mut mock = MockScorecardRepository::new();
        mock.expect_get()
            .withf(move |i| *i == id)
            .returning(move |i| Ok(Some(sample_scorecard(i))));

        let uc = GetScorecardUseCase::new(Arc::new(mock));
        let result = uc.execute(id).await.unwrap();
        assert_eq!(result.service_id, id);
        assert_eq!(result.overall_score, 82.5);
    }

    /// スコアカードが存在しない場合は NotFound エラーを返す
    #[tokio::test]
    async fn not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockScorecardRepository::new();
        mock.expect_get().returning(|_| Ok(None));

        let uc = GetScorecardUseCase::new(Arc::new(mock));
        let result = uc.execute(id).await;
        assert!(matches!(result, Err(GetScorecardError::NotFound(_))));
    }

    /// リポジトリエラー時は Internal エラーを返す
    #[tokio::test]
    async fn internal_error() {
        let id = Uuid::new_v4();
        let mut mock = MockScorecardRepository::new();
        mock.expect_get()
            .returning(|_| Err(anyhow::anyhow!("db connection failed")));

        let uc = GetScorecardUseCase::new(Arc::new(mock));
        let result = uc.execute(id).await;
        assert!(matches!(result, Err(GetScorecardError::Internal(_))));
    }
}
