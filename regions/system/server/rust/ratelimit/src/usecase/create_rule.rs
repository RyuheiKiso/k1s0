use std::sync::Arc;

use crate::domain::entity::{Algorithm, RateLimitRule};
use crate::domain::repository::RateLimitRepository;
use crate::domain::service::RateLimitDomainService;

/// `CreateRuleError` はルール作成に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CreateRuleError {
    #[error("invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("rule already exists: {0}")]
    AlreadyExists(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `CreateRuleInput` はルール作成の入力。
pub struct CreateRuleInput {
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: u32,
    pub window_seconds: u32,
    pub algorithm: Option<String>,
    pub enabled: bool,
    /// CRIT-005 対応: RLS テナント分離用テナント ID。
    pub tenant_id: String,
}

/// `CreateRuleUseCase` はルール作成ユースケース。
pub struct CreateRuleUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

impl CreateRuleUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからルールを作成する。
    pub async fn execute(&self, input: &CreateRuleInput) -> Result<RateLimitRule, CreateRuleError> {
        // バリデーション
        RateLimitDomainService::validate_rule_input(
            &input.scope,
            &input.identifier_pattern,
            input.limit,
            input.window_seconds,
        )
        .map_err(CreateRuleError::Validation)?;

        // CRIT-005 対応: テナント分離しながら重複チェック（scope+identifier_patternで確認）
        let existing = self
            .repo
            .find_by_scope(&input.scope, &input.tenant_id)
            .await
            .map_err(|e| CreateRuleError::Internal(e.to_string()))?;
        if existing
            .iter()
            .any(|r| r.identifier_pattern == input.identifier_pattern)
        {
            return Err(CreateRuleError::AlreadyExists(format!(
                "{}:{}",
                input.scope, input.identifier_pattern
            )));
        }

        let algorithm = Algorithm::from_str(input.algorithm.as_deref().unwrap_or("token_bucket"))
            .map_err(CreateRuleError::InvalidAlgorithm)?;

        let mut rule = RateLimitRule::new(
            input.scope.clone(),
            input.identifier_pattern.clone(),
            input.limit,
            input.window_seconds,
            algorithm,
        );
        rule.enabled = input.enabled;
        // CRIT-005 対応: テナント ID をルールエンティティにセットして RLS で分離する。
        rule.tenant_id = input.tenant_id.clone();

        let created = self
            .repo
            .create(&rule)
            .await
            .map_err(|e| CreateRuleError::Internal(e.to_string()))?;

        Ok(created)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_create_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_scope().returning(|_, _| Ok(vec![]));
        repo.expect_create().returning(|rule| Ok(rule.clone()));

        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "global".to_string(),
                limit: 100,
                window_seconds: 60,
                algorithm: None,
                enabled: true,
                tenant_id: "tenant-a".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.scope, "service");
        assert_eq!(rule.identifier_pattern, "global");
        assert_eq!(rule.limit, 100);
        assert_eq!(rule.window_seconds, 60);
        assert_eq!(rule.algorithm, Algorithm::TokenBucket);
        assert!(rule.enabled);
    }

    #[tokio::test]
    async fn test_create_rule_duplicate_scope_and_identifier() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_scope().returning(|_, _| {
            Ok(vec![RateLimitRule::new(
                "service".to_string(),
                "global".to_string(),
                100,
                60,
                Algorithm::TokenBucket,
            )])
        });

        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "global".to_string(),
                limit: 100,
                window_seconds: 60,
                algorithm: None,
                enabled: true,
                tenant_id: "tenant-a".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateRuleError::AlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_create_rule_empty_scope() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "".to_string(),
                identifier_pattern: "test".to_string(),
                limit: 100,
                window_seconds: 60,
                algorithm: None,
                enabled: true,
                tenant_id: "tenant-a".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateRuleError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_create_rule_invalid_limit() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "test".to_string(),
                limit: 0,
                window_seconds: 60,
                algorithm: None,
                enabled: true,
                tenant_id: "tenant-a".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateRuleError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_create_rule_invalid_window() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "test".to_string(),
                limit: 100,
                window_seconds: 0,
                algorithm: None,
                enabled: true,
                tenant_id: "tenant-a".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateRuleError::Validation(_)
        ));
    }
}
