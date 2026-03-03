use std::sync::Arc;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::repository::FeatureFlagRepository;
use crate::infrastructure::kafka_producer::FlagEventPublisher;

#[derive(Debug, Clone)]
pub struct UpdateFlagInput {
    pub flag_key: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateFlagError {
    #[error("flag not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
    event_publisher: Arc<dyn FlagEventPublisher>,
}

impl UpdateFlagUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>, event_publisher: Arc<dyn FlagEventPublisher>) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &UpdateFlagInput) -> Result<FeatureFlag, UpdateFlagError> {
        let mut flag = self.repo.find_by_key(&input.flag_key).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                UpdateFlagError::NotFound(input.flag_key.clone())
            } else {
                UpdateFlagError::Internal(msg)
            }
        })?;

        if let Some(enabled) = input.enabled {
            flag.enabled = enabled;
        }
        if let Some(ref description) = input.description {
            flag.description = description.clone();
        }
        flag.updated_at = chrono::Utc::now();

        self.repo
            .update(&flag)
            .await
            .map_err(|e| UpdateFlagError::Internal(e.to_string()))?;

        self.event_publisher
            .publish_flag_changed(&flag.flag_key, flag.enabled)
            .await
            .map_err(|e| UpdateFlagError::Internal(e.to_string()))?;

        Ok(flag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::infrastructure::kafka_producer::MockFlagEventPublisher;

    #[tokio::test]
    async fn success() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = FeatureFlag::new("dark-mode".to_string(), "Dark mode".to_string(), true);
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .withf(|key| key == "dark-mode")
            .returning(move |_| Ok(return_flag.clone()));
        mock.expect_update().returning(|_| Ok(()));
        let mut mock_publisher = MockFlagEventPublisher::new();
        mock_publisher
            .expect_publish_flag_changed()
            .withf(|key, enabled| key == "dark-mode" && !*enabled)
            .returning(|_, _| Ok(()));

        let uc = UpdateFlagUseCase::new(Arc::new(mock), Arc::new(mock_publisher));
        let input = UpdateFlagInput {
            flag_key: "dark-mode".to_string(),
            enabled: Some(false),
            description: Some("Updated dark mode".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.flag_key, "dark-mode");
        assert!(!updated.enabled);
        assert_eq!(updated.description, "Updated dark mode");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_by_key()
            .returning(|_| Err(anyhow::anyhow!("flag not found")));

        let uc = UpdateFlagUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopFlagEventPublisher),
        );
        let input = UpdateFlagInput {
            flag_key: "nonexistent".to_string(),
            enabled: Some(true),
            description: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateFlagError::NotFound(key) => assert_eq!(key, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
