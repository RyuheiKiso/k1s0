use std::sync::Arc;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::repository::FeatureFlagRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetFlagError {
    #[error("flag not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
}

impl GetFlagUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, flag_key: &str) -> Result<FeatureFlag, GetFlagError> {
        self.repo.find_by_key(flag_key).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GetFlagError::NotFound(flag_key.to_string())
            } else {
                GetFlagError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;

    #[tokio::test]
    async fn found() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = FeatureFlag::new("dark-mode".to_string(), "Dark mode".to_string(), true);
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .withf(|key| key == "dark-mode")
            .returning(move |_| Ok(return_flag.clone()));

        let uc = GetFlagUseCase::new(Arc::new(mock));
        let result = uc.execute("dark-mode").await;
        assert!(result.is_ok());

        let flag = result.unwrap();
        assert_eq!(flag.flag_key, "dark-mode");
        assert!(flag.enabled);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_by_key()
            .returning(|_| Err(anyhow::anyhow!("flag not found")));

        let uc = GetFlagUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetFlagError::NotFound(key) => assert_eq!(key, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
