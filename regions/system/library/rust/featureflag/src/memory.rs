use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{EvaluationContext, EvaluationResult, FeatureFlag, FeatureFlagClient, FeatureFlagError};

#[derive(Clone)]
pub struct InMemoryFeatureFlagClient {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl InMemoryFeatureFlagClient {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_flag(&self, flag: FeatureFlag) {
        self.flags.write().await.insert(flag.flag_key.clone(), flag);
    }
}

impl Default for InMemoryFeatureFlagClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeatureFlagClient for InMemoryFeatureFlagClient {
    async fn evaluate(
        &self,
        flag_key: &str,
        _context: &EvaluationContext,
    ) -> Result<EvaluationResult, FeatureFlagError> {
        let flags = self.flags.read().await;
        match flags.get(flag_key) {
            Some(flag) => Ok(EvaluationResult {
                flag_key: flag_key.to_string(),
                enabled: flag.enabled,
                variant: flag.variants.first().map(|v| v.name.clone()),
                reason: if flag.enabled {
                    "FLAG_ENABLED".to_string()
                } else {
                    "FLAG_DISABLED".to_string()
                },
            }),
            None => Err(FeatureFlagError::FlagNotFound {
                key: flag_key.to_string(),
            }),
        }
    }

    async fn get_flag(&self, flag_key: &str) -> Result<FeatureFlag, FeatureFlagError> {
        let flags = self.flags.read().await;
        flags
            .get(flag_key)
            .cloned()
            .ok_or_else(|| FeatureFlagError::FlagNotFound {
                key: flag_key.to_string(),
            })
    }

    async fn is_enabled(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<bool, FeatureFlagError> {
        Ok(self.evaluate(flag_key, context).await?.enabled)
    }
}
