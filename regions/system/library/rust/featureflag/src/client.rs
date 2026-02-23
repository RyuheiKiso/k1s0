use async_trait::async_trait;

use crate::{EvaluationContext, FeatureFlag, FeatureFlagError};

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub flag_key: String,
    pub enabled: bool,
    pub variant: Option<String>,
    pub reason: String,
}

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait FeatureFlagClient: Send + Sync {
    async fn evaluate(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, FeatureFlagError>;

    async fn get_flag(&self, flag_key: &str) -> Result<FeatureFlag, FeatureFlagError>;

    async fn is_enabled(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<bool, FeatureFlagError>;
}
