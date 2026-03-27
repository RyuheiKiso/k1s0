use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    EvaluationContext, EvaluationResult, FeatureFlag, FeatureFlagClient, FeatureFlagError,
};

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

// テストコードでは unwrap() を許可する（unwrap_used = "deny" はプロダクションコード向け）
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::flag::FlagVariant;

    fn make_flag(key: &str, enabled: bool) -> FeatureFlag {
        FeatureFlag {
            id: format!("id-{key}"),
            flag_key: key.to_string(),
            description: "test flag".to_string(),
            enabled,
            variants: vec![],
        }
    }

    fn make_flag_with_variant(key: &str, enabled: bool, variant: &str) -> FeatureFlag {
        FeatureFlag {
            id: format!("id-{key}"),
            flag_key: key.to_string(),
            description: "test flag".to_string(),
            enabled,
            variants: vec![FlagVariant {
                name: variant.to_string(),
                value: "1".to_string(),
                weight: 100,
            }],
        }
    }

    /// 有効なフラグを evaluate すると FLAG_ENABLED reason が返る
    #[tokio::test]
    async fn evaluate_enabled_flag_returns_enabled_reason() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("feature-a", true)).await;
        let ctx = EvaluationContext::new();
        let result = client.evaluate("feature-a", &ctx).await.unwrap();
        assert!(result.enabled);
        assert_eq!(result.reason, "FLAG_ENABLED");
        assert_eq!(result.flag_key, "feature-a");
    }

    /// 無効なフラグを evaluate すると FLAG_DISABLED reason が返る
    #[tokio::test]
    async fn evaluate_disabled_flag_returns_disabled_reason() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("feature-b", false)).await;
        let ctx = EvaluationContext::new();
        let result = client.evaluate("feature-b", &ctx).await.unwrap();
        assert!(!result.enabled);
        assert_eq!(result.reason, "FLAG_DISABLED");
    }

    /// 存在しないフラグを evaluate すると FlagNotFound エラーが返る
    #[tokio::test]
    async fn evaluate_missing_flag_returns_not_found() {
        let client = InMemoryFeatureFlagClient::new();
        let ctx = EvaluationContext::new();
        let err = client.evaluate("nonexistent", &ctx).await.unwrap_err();
        assert!(matches!(err, FeatureFlagError::FlagNotFound { .. }));
    }

    /// get_flag で登録済みフラグを取得できる
    #[tokio::test]
    async fn get_flag_returns_stored_flag() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("my-flag", true)).await;
        let flag = client.get_flag("my-flag").await.unwrap();
        assert_eq!(flag.flag_key, "my-flag");
        assert!(flag.enabled);
    }

    /// is_enabled で有効フラグが true を返す
    #[tokio::test]
    async fn is_enabled_returns_true_for_enabled_flag() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("flag-enabled", true)).await;
        let ctx = EvaluationContext::new().with_user_id("user-1");
        assert!(client.is_enabled("flag-enabled", &ctx).await.unwrap());
    }

    /// is_enabled で無効フラグが false を返す
    #[tokio::test]
    async fn is_enabled_returns_false_for_disabled_flag() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("flag-disabled", false)).await;
        let ctx = EvaluationContext::new();
        assert!(!client.is_enabled("flag-disabled", &ctx).await.unwrap());
    }

    /// バリアント付きフラグで variant が返る
    #[tokio::test]
    async fn evaluate_flag_with_variant_returns_variant_name() {
        let client = InMemoryFeatureFlagClient::new();
        client
            .set_flag(make_flag_with_variant("flag-var", true, "beta"))
            .await;
        let ctx = EvaluationContext::new();
        let result = client.evaluate("flag-var", &ctx).await.unwrap();
        assert_eq!(result.variant, Some("beta".to_string()));
    }

    /// set_flag で上書きができる
    #[tokio::test]
    async fn set_flag_overwrites_existing_flag() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("flag-x", true)).await;
        client.set_flag(make_flag("flag-x", false)).await;
        let ctx = EvaluationContext::new();
        let result = client.evaluate("flag-x", &ctx).await.unwrap();
        assert!(!result.enabled);
    }
}
