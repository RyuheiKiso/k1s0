use std::sync::Arc;

use crate::domain::entity::evaluation::{EvaluationContext, EvaluationResult};
use crate::domain::repository::FeatureFlagRepository;

#[derive(Debug, Clone)]
pub struct EvaluateFlagInput {
    pub flag_key: String,
    pub context: EvaluationContext,
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluateFlagError {
    #[error("flag not found: {0}")]
    FlagNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct EvaluateFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
}

impl EvaluateFlagUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &EvaluateFlagInput) -> Result<EvaluationResult, EvaluateFlagError> {
        let flag = self.repo.find_by_key(&input.flag_key).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                EvaluateFlagError::FlagNotFound(input.flag_key.clone())
            } else {
                EvaluateFlagError::Internal(msg)
            }
        })?;

        if !flag.enabled {
            return Ok(EvaluationResult {
                flag_key: flag.flag_key,
                enabled: false,
                variant: None,
                reason: "flag is disabled".to_string(),
            });
        }

        let variant = flag.variants.first().map(|v| v.name.clone());
        Ok(EvaluationResult {
            flag_key: flag.flag_key,
            enabled: true,
            variant,
            reason: "flag is enabled".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::{FeatureFlag, FlagVariant};
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use std::collections::HashMap;

    fn make_context() -> EvaluationContext {
        EvaluationContext {
            user_id: Some("user-1".to_string()),
            tenant_id: None,
            attributes: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn enabled_flag_returns_result() {
        let mut mock = MockFeatureFlagRepository::new();
        let mut flag = FeatureFlag::new("dark-mode".to_string(), "Dark mode".to_string(), true);
        flag.variants.push(FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: 100,
        });
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .withf(|key| key == "dark-mode")
            .returning(move |_| Ok(return_flag.clone()));

        let uc = EvaluateFlagUseCase::new(Arc::new(mock));
        let input = EvaluateFlagInput {
            flag_key: "dark-mode".to_string(),
            context: make_context(),
        };
        let result = uc.execute(&input).await.unwrap();

        assert!(result.enabled);
        assert_eq!(result.flag_key, "dark-mode");
        assert_eq!(result.variant, Some("on".to_string()));
        assert_eq!(result.reason, "flag is enabled");
    }

    #[tokio::test]
    async fn disabled_flag_returns_false() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = FeatureFlag::new("beta-feature".to_string(), "Beta".to_string(), false);
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .returning(move |_| Ok(return_flag.clone()));

        let uc = EvaluateFlagUseCase::new(Arc::new(mock));
        let input = EvaluateFlagInput {
            flag_key: "beta-feature".to_string(),
            context: make_context(),
        };
        let result = uc.execute(&input).await.unwrap();

        assert!(!result.enabled);
        assert!(result.variant.is_none());
        assert_eq!(result.reason, "flag is disabled");
    }

    #[tokio::test]
    async fn not_found_flag_error() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_by_key()
            .returning(|_| Err(anyhow::anyhow!("flag not found")));

        let uc = EvaluateFlagUseCase::new(Arc::new(mock));
        let input = EvaluateFlagInput {
            flag_key: "nonexistent".to_string(),
            context: make_context(),
        };
        let result = uc.execute(&input).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            EvaluateFlagError::FlagNotFound(key) => assert_eq!(key, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
