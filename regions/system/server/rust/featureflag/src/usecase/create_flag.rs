use std::sync::Arc;

use crate::domain::entity::feature_flag::{FeatureFlag, FlagVariant};
use crate::domain::repository::FeatureFlagRepository;
use crate::domain::service::FeatureFlagDomainService;
use crate::infrastructure::kafka_producer::FlagEventPublisher;

#[derive(Debug, Clone)]
pub struct CreateFlagInput {
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateFlagError {
    #[error("flag already exists: {0}")]
    AlreadyExists(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
    event_publisher: Arc<dyn FlagEventPublisher>,
}

impl CreateFlagUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>, event_publisher: Arc<dyn FlagEventPublisher>) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &CreateFlagInput) -> Result<FeatureFlag, CreateFlagError> {
        FeatureFlagDomainService::validate_flag_key(&input.flag_key)
            .map_err(CreateFlagError::Internal)?;
        FeatureFlagDomainService::validate_variants(&input.variants)
            .map_err(CreateFlagError::Internal)?;

        let exists = self
            .repo
            .exists_by_key(&input.flag_key)
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        if exists {
            return Err(CreateFlagError::AlreadyExists(input.flag_key.clone()));
        }

        let mut flag = FeatureFlag::new(
            input.flag_key.clone(),
            input.description.clone(),
            input.enabled,
        );
        flag.variants = input.variants.clone();

        self.repo
            .create(&flag)
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        self.event_publisher
            .publish_flag_changed(&flag.flag_key, flag.enabled)
            .await
            .map_err(|e| CreateFlagError::Internal(e.to_string()))?;

        Ok(flag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::infrastructure::kafka_producer::MockFlagEventPublisher;

    #[tokio::test]
    async fn success() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_exists_by_key()
            .withf(|key| key == "new-feature")
            .returning(|_| Ok(false));
        mock.expect_create().returning(|_| Ok(()));
        let mut mock_publisher = MockFlagEventPublisher::new();
        mock_publisher
            .expect_publish_flag_changed()
            .withf(|key, enabled| key == "new-feature" && *enabled)
            .returning(|_, _| Ok(()));

        let uc = CreateFlagUseCase::new(Arc::new(mock), Arc::new(mock_publisher));
        let input = CreateFlagInput {
            flag_key: "new-feature".to_string(),
            description: "A new feature".to_string(),
            enabled: true,
            variants: vec![FlagVariant {
                name: "on".to_string(),
                value: "true".to_string(),
                weight: 100,
            }],
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let flag = result.unwrap();
        assert_eq!(flag.flag_key, "new-feature");
        assert!(flag.enabled);
        assert_eq!(flag.variants.len(), 1);
        assert_eq!(flag.variants[0].name, "on");
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_exists_by_key()
            .withf(|key| key == "existing-feature")
            .returning(|_| Ok(true));

        let uc = CreateFlagUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopFlagEventPublisher),
        );
        let input = CreateFlagInput {
            flag_key: "existing-feature".to_string(),
            description: "Existing".to_string(),
            enabled: true,
            variants: vec![],
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateFlagError::AlreadyExists(key) => assert_eq!(key, "existing-feature"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
