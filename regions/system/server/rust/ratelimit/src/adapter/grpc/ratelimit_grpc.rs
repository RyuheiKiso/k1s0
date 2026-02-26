use std::sync::Arc;

use crate::usecase::{CheckRateLimitUseCase, CreateRuleUseCase, GetRuleUseCase, GetUsageUseCase, ResetRateLimitUseCase, ResetRateLimitInput};
use crate::usecase::create_rule::CreateRuleInput;

/// GrpcError は gRPC エラー型。
#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- gRPC リクエスト/レスポンス型 ---

pub struct CheckRateLimitRequest {
    pub rule_id: String,
    pub subject: String,
}

#[derive(Debug)]
pub struct CheckRateLimitResponse {
    pub allowed: bool,
    pub remaining: i64,
    pub reset_at: i64,
    pub reason: String,
}

pub struct CreateRuleRequest {
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
}

#[derive(Debug)]
pub struct CreateRuleResponse {
    pub rule: RuleResponse,
}

pub struct GetRuleRequest {
    pub rule_id: String,
}

#[derive(Debug)]
pub struct GetRuleResponse {
    pub rule: RuleResponse,
}

pub struct GetUsageRequest {
    pub rule_id: String,
}

#[derive(Debug)]
pub struct GetUsageResponse {
    pub rule_id: String,
    pub rule_name: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
    pub enabled: bool,
}

pub struct ResetLimitRequest {
    pub scope: String,
    pub identifier: String,
}

#[derive(Debug)]
pub struct ResetLimitResponse {
    pub success: bool,
}

#[derive(Debug)]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: Option<PbTimestamp>,
}

#[derive(Debug)]
pub struct PbTimestamp {
    pub seconds: i64,
    pub nanos: i32,
}

/// RateLimitGrpcService は gRPC サービスの実装。
pub struct RateLimitGrpcService {
    check_uc: Arc<CheckRateLimitUseCase>,
    create_uc: Arc<CreateRuleUseCase>,
    get_uc: Arc<GetRuleUseCase>,
    usage_uc: Arc<GetUsageUseCase>,
    reset_uc: Arc<ResetRateLimitUseCase>,
}

impl RateLimitGrpcService {
    pub fn new(
        check_uc: Arc<CheckRateLimitUseCase>,
        create_uc: Arc<CreateRuleUseCase>,
        get_uc: Arc<GetRuleUseCase>,
        usage_uc: Arc<GetUsageUseCase>,
        reset_uc: Arc<ResetRateLimitUseCase>,
    ) -> Self {
        Self {
            check_uc,
            create_uc,
            get_uc,
            usage_uc,
            reset_uc,
        }
    }

    pub async fn check_rate_limit(
        &self,
        req: CheckRateLimitRequest,
    ) -> Result<CheckRateLimitResponse, GrpcError> {
        if req.rule_id.is_empty() {
            return Err(GrpcError::InvalidArgument("rule_id is required".to_string()));
        }
        if req.subject.is_empty() {
            return Err(GrpcError::InvalidArgument("subject is required".to_string()));
        }

        // gRPC は rule_id/subject を scope/identifier として扱う（後方互換）
        let decision = self
            .check_uc
            .execute(&req.rule_id, &req.subject, 60)
            .await
            .map_err(|e| match e {
                crate::usecase::check_rate_limit::CheckRateLimitError::RuleNotFound(msg) => {
                    GrpcError::NotFound(msg)
                }
                crate::usecase::check_rate_limit::CheckRateLimitError::ValidationError(msg) => {
                    GrpcError::InvalidArgument(msg)
                }
                crate::usecase::check_rate_limit::CheckRateLimitError::RuleDisabled(msg) => {
                    GrpcError::InvalidArgument(format!("rule is disabled: {}", msg))
                }
                crate::usecase::check_rate_limit::CheckRateLimitError::Internal(msg) => {
                    GrpcError::Internal(msg)
                }
            })?;

        Ok(CheckRateLimitResponse {
            allowed: decision.allowed,
            remaining: decision.remaining,
            reset_at: decision.reset_at,
            reason: decision.reason,
        })
    }

    pub async fn create_rule(
        &self,
        req: CreateRuleRequest,
    ) -> Result<CreateRuleResponse, GrpcError> {
        let input = CreateRuleInput {
            scope: req.name,
            identifier_pattern: req.key,
            limit: req.limit,
            window_seconds: req.window_secs,
            enabled: true,
        };

        let rule = self.create_uc.execute(&input).await.map_err(|e| match e {
            crate::usecase::create_rule::CreateRuleError::AlreadyExists(msg) => {
                GrpcError::InvalidArgument(format!("rule already exists: {}", msg))
            }
            crate::usecase::create_rule::CreateRuleError::InvalidAlgorithm(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::create_rule::CreateRuleError::Validation(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::create_rule::CreateRuleError::Internal(msg) => {
                GrpcError::Internal(msg)
            }
        })?;

        let ts = PbTimestamp {
            seconds: rule.created_at.timestamp(),
            nanos: rule.created_at.timestamp_subsec_nanos() as i32,
        };

        Ok(CreateRuleResponse {
            rule: RuleResponse {
                id: rule.id.to_string(),
                name: rule.scope,
                key: rule.identifier_pattern,
                limit: rule.limit,
                window_secs: rule.window_seconds,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: Some(ts),
            },
        })
    }

    pub async fn get_rule(&self, req: GetRuleRequest) -> Result<GetRuleResponse, GrpcError> {
        if req.rule_id.is_empty() {
            return Err(GrpcError::InvalidArgument("rule_id is required".to_string()));
        }

        let rule = self.get_uc.execute(&req.rule_id).await.map_err(|e| match e {
            crate::usecase::get_rule::GetRuleError::NotFound(msg) => GrpcError::NotFound(msg),
            crate::usecase::get_rule::GetRuleError::InvalidRuleId(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::get_rule::GetRuleError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        let ts = PbTimestamp {
            seconds: rule.created_at.timestamp(),
            nanos: rule.created_at.timestamp_subsec_nanos() as i32,
        };

        Ok(GetRuleResponse {
            rule: RuleResponse {
                id: rule.id.to_string(),
                name: rule.scope,
                key: rule.identifier_pattern,
                limit: rule.limit,
                window_secs: rule.window_seconds,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: Some(ts),
            },
        })
    }

    pub async fn get_usage(&self, req: GetUsageRequest) -> Result<GetUsageResponse, GrpcError> {
        if req.rule_id.is_empty() {
            return Err(GrpcError::InvalidArgument("rule_id is required".to_string()));
        }

        let info = self.usage_uc.execute(&req.rule_id).await.map_err(|e| match e {
            crate::usecase::get_usage::GetUsageError::NotFound(msg) => GrpcError::NotFound(msg),
            crate::usecase::get_usage::GetUsageError::InvalidRuleId(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::get_usage::GetUsageError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(GetUsageResponse {
            rule_id: info.rule_id,
            rule_name: info.rule_name,
            limit: info.limit,
            window_secs: info.window_seconds,
            algorithm: info.algorithm,
            enabled: info.enabled,
        })
    }

    pub async fn reset_limit(&self, req: ResetLimitRequest) -> Result<ResetLimitResponse, GrpcError> {
        let input = ResetRateLimitInput {
            scope: req.scope,
            identifier: req.identifier,
        };

        self.reset_uc.execute(&input).await.map_err(|e| match e {
            crate::usecase::reset_rate_limit::ResetRateLimitError::ValidationError(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::reset_rate_limit::ResetRateLimitError::Internal(msg) => {
                GrpcError::Internal(msg)
            }
        })?;

        Ok(ResetLimitResponse { success: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};
    use crate::domain::repository::rate_limit_repository::{
        MockRateLimitRepository, MockRateLimitStateStore,
    };

    fn make_service_with(
        check_uc: Arc<CheckRateLimitUseCase>,
        create_uc: Arc<CreateRuleUseCase>,
        get_uc: Arc<GetRuleUseCase>,
    ) -> RateLimitGrpcService {
        let usage_uc = Arc::new(GetUsageUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let reset_uc = Arc::new(ResetRateLimitUseCase::new(Arc::new(MockRateLimitStateStore::new())));
        RateLimitGrpcService::new(check_uc, create_uc, get_uc, usage_uc, reset_uc)
    }

    fn make_rule() -> RateLimitRule {
        RateLimitRule::new(
            "service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        )
    }

    #[tokio::test]
    async fn test_grpc_check_rate_limit_allowed() {
        let rule = make_rule();

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(99, 1700000060)));

        let check_uc = Arc::new(CheckRateLimitUseCase::new(
            Arc::new(repo),
            Arc::new(state_store),
        ));
        let create_uc = Arc::new(CreateRuleUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let get_uc = Arc::new(GetRuleUseCase::new(Arc::new(MockRateLimitRepository::new())));

        let svc = make_service_with(check_uc, create_uc, get_uc);
        let result = svc
            .check_rate_limit(CheckRateLimitRequest {
                rule_id: "service".to_string(),
                subject: "user-123".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.allowed);
        assert_eq!(resp.remaining, 99);
    }

    #[tokio::test]
    async fn test_grpc_check_rate_limit_empty_rule_id() {
        let svc = make_service_with(
            Arc::new(CheckRateLimitUseCase::new(
                Arc::new(MockRateLimitRepository::new()),
                Arc::new(MockRateLimitStateStore::new()),
            )),
            Arc::new(CreateRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(GetRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
        );

        let result = svc
            .check_rate_limit(CheckRateLimitRequest {
                rule_id: "".to_string(),
                subject: "user-123".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GrpcError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_grpc_get_rule_empty_id() {
        let svc = make_service_with(
            Arc::new(CheckRateLimitUseCase::new(
                Arc::new(MockRateLimitRepository::new()),
                Arc::new(MockRateLimitStateStore::new()),
            )),
            Arc::new(CreateRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(GetRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
        );

        let result = svc
            .get_rule(GetRuleRequest {
                rule_id: "".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GrpcError::InvalidArgument(_)));
    }
}
