use std::sync::Arc;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::repository::FeatureFlagRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListFlagsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListFlagsUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
}

impl ListFlagsUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<FeatureFlag>, ListFlagsError> {
        self.repo
            .find_all()
            .await
            .map_err(|e| ListFlagsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;

    #[tokio::test]
    async fn test_list_flags_success() {
        let mut repo = MockFeatureFlagRepository::new();
        repo.expect_find_all().returning(|| Ok(vec![]));

        let uc = ListFlagsUseCase::new(Arc::new(repo));
        let flags = uc.execute().await.unwrap();
        assert!(flags.is_empty());
    }
}
