use std::sync::Arc;

use crate::domain::entity::evaluation::{EvaluationContext, EvaluationResult};
use crate::domain::repository::FeatureFlagRepository;
use crate::domain::service::FeatureFlagDomainService;

/// EvaluateFlagInput はフィーチャーフラグ評価の入力データ。
/// STATIC-CRITICAL-001 監査対応: tenant_id でテナントスコープを指定する。
/// HIGH-005 対応: tenant_id は String 型（migration 006 で DB の TEXT 型に変更済み）。
#[derive(Debug, Clone)]
pub struct EvaluateFlagInput {
    pub tenant_id: String,
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

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを評価する。
    pub async fn execute(
        &self,
        input: &EvaluateFlagInput,
    ) -> Result<EvaluationResult, EvaluateFlagError> {
        let flag = self
            .repo
            .find_by_key(&input.tenant_id, &input.flag_key)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    EvaluateFlagError::FlagNotFound(input.flag_key.clone())
                } else {
                    EvaluateFlagError::Internal(msg)
                }
            })?;
        let (enabled, variant, reason) = FeatureFlagDomainService::evaluate(&flag, &input.context);
        Ok(EvaluationResult {
            flag_key: flag.flag_key,
            enabled,
            variant,
            reason,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::{FeatureFlag, FlagVariant};
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// システムテナント文字列: テスト共通（HIGH-005 対応: TEXT 型）
    fn system_tenant() -> String {
        "00000000-0000-0000-0000-000000000001".to_string()
    }

    fn make_context() -> EvaluationContext {
        EvaluationContext {
            user_id: Some("user-1".to_string()),
            tenant_id: None,
            attributes: HashMap::new(),
        }
    }

    fn make_flag(flag_key: &str, enabled: bool) -> FeatureFlag {
        FeatureFlag {
            id: Uuid::new_v4(),
            tenant_id: system_tenant(),
            flag_key: flag_key.to_string(),
            description: String::new(),
            enabled,
            variants: vec![],
            rules: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn enabled_flag_returns_result() {
        let mut mock = MockFeatureFlagRepository::new();
        let mut flag = make_flag("dark-mode", true);
        flag.variants.push(FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: 100,
        });
        let return_flag = flag.clone();

        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "dark-mode")
            .returning(move |_, _| Ok(return_flag.clone()));

        let uc = EvaluateFlagUseCase::new(Arc::new(mock));
        let input = EvaluateFlagInput {
            tenant_id: system_tenant(),
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
        let flag = make_flag("beta-feature", false);
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .returning(move |_, _| Ok(return_flag.clone()));

        let uc = EvaluateFlagUseCase::new(Arc::new(mock));
        let input = EvaluateFlagInput {
            tenant_id: system_tenant(),
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
            .returning(|_, _| Err(anyhow::anyhow!("flag not found")));

        let uc = EvaluateFlagUseCase::new(Arc::new(mock));
        let input = EvaluateFlagInput {
            tenant_id: system_tenant(),
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
