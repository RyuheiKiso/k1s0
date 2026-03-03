use std::sync::Arc;

use crate::usecase::create_rule::CreateRuleInput;
use crate::usecase::update_rule::UpdateRuleInput;
use crate::usecase::{
    CheckRateLimitUseCase, CreateRuleUseCase, DeleteRuleUseCase, GetRuleUseCase,
    GetUsageUseCase, ListRulesUseCase, ResetRateLimitInput, ResetRateLimitUseCase,
    UpdateRuleUseCase,
};

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub struct CheckRateLimitRequest {
    pub scope: String,
    pub identifier: String,
    pub window: i64,
}

#[derive(Debug)]
pub struct CheckRateLimitResponse {
    pub allowed: bool,
    pub limit: i64,
    pub remaining: i64,
    pub reset_at: i64,
    pub reason: String,
}

pub struct CreateRuleRequest {
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: Option<String>,
    pub enabled: bool,
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
    pub window_seconds: i64,
    pub algorithm: String,
    pub enabled: bool,
    pub used: Option<i64>,
    pub remaining: Option<i64>,
    pub reset_at: Option<i64>,
}

pub struct UpdateRuleRequest {
    pub rule_id: String,
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: Option<String>,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct UpdateRuleResponse {
    pub rule: RuleResponse,
}

pub struct DeleteRuleRequest {
    pub rule_id: String,
}

#[derive(Debug)]
pub struct DeleteRuleResponse {
    pub success: bool,
}

pub struct ListRulesRequest {
    pub scope: String,
    pub enabled_only: Option<bool>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug)]
pub struct ListRulesResponse {
    pub rules: Vec<RuleResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug)]
pub struct PaginationResponse {
    pub total_count: i32,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
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
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: Option<PbTimestamp>,
    pub updated_at: Option<PbTimestamp>,
}

#[derive(Debug)]
pub struct PbTimestamp {
    pub seconds: i64,
    pub nanos: i32,
}

pub struct RateLimitGrpcService {
    check_uc: Arc<CheckRateLimitUseCase>,
    create_uc: Arc<CreateRuleUseCase>,
    get_uc: Arc<GetRuleUseCase>,
    update_uc: Arc<UpdateRuleUseCase>,
    delete_uc: Arc<DeleteRuleUseCase>,
    list_uc: Arc<ListRulesUseCase>,
    usage_uc: Arc<GetUsageUseCase>,
    reset_uc: Arc<ResetRateLimitUseCase>,
}

impl RateLimitGrpcService {
    pub fn new(
        check_uc: Arc<CheckRateLimitUseCase>,
        create_uc: Arc<CreateRuleUseCase>,
        get_uc: Arc<GetRuleUseCase>,
        update_uc: Arc<UpdateRuleUseCase>,
        delete_uc: Arc<DeleteRuleUseCase>,
        list_uc: Arc<ListRulesUseCase>,
        usage_uc: Arc<GetUsageUseCase>,
        reset_uc: Arc<ResetRateLimitUseCase>,
    ) -> Self {
        Self {
            check_uc,
            create_uc,
            get_uc,
            update_uc,
            delete_uc,
            list_uc,
            usage_uc,
            reset_uc,
        }
    }

    pub async fn check_rate_limit(
        &self,
        req: CheckRateLimitRequest,
    ) -> Result<CheckRateLimitResponse, GrpcError> {
        if req.scope.is_empty() {
            return Err(GrpcError::InvalidArgument("scope is required".to_string()));
        }
        if req.identifier.is_empty() {
            return Err(GrpcError::InvalidArgument("identifier is required".to_string()));
        }

        let window = if req.window > 0 { req.window } else { 60 };
        let decision = self
            .check_uc
            .execute(&req.scope, &req.identifier, window)
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
            limit: decision.limit,
            remaining: decision.remaining,
            reset_at: decision.reset_at,
            reason: decision.reason,
        })
    }

    pub async fn create_rule(
        &self,
        req: CreateRuleRequest,
    ) -> Result<CreateRuleResponse, GrpcError> {
        let limit = u32::try_from(req.limit)
            .map_err(|_| GrpcError::InvalidArgument("limit must be >= 0".to_string()))?;
        let window_seconds = u32::try_from(req.window_seconds)
            .map_err(|_| GrpcError::InvalidArgument("window_seconds must be >= 0".to_string()))?;
        let input = CreateRuleInput {
            scope: req.scope,
            identifier_pattern: req.identifier_pattern,
            limit,
            window_seconds,
            algorithm: req.algorithm,
            enabled: req.enabled,
        };

        let rule = self.create_uc.execute(&input).await.map_err(|e| match e {
            crate::usecase::create_rule::CreateRuleError::AlreadyExists(msg) => {
                GrpcError::AlreadyExists(format!("rule already exists: {}", msg))
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

        Ok(CreateRuleResponse {
            rule: RuleResponse {
                id: rule.id.to_string(),
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: i64::from(rule.limit),
                window_seconds: i64::from(rule.window_seconds),
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: Some(PbTimestamp {
                    seconds: rule.created_at.timestamp(),
                    nanos: rule.created_at.timestamp_subsec_nanos() as i32,
                }),
                updated_at: Some(PbTimestamp {
                    seconds: rule.updated_at.timestamp(),
                    nanos: rule.updated_at.timestamp_subsec_nanos() as i32,
                }),
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

        Ok(GetRuleResponse {
            rule: RuleResponse {
                id: rule.id.to_string(),
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: i64::from(rule.limit),
                window_seconds: i64::from(rule.window_seconds),
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: Some(PbTimestamp {
                    seconds: rule.created_at.timestamp(),
                    nanos: rule.created_at.timestamp_subsec_nanos() as i32,
                }),
                updated_at: Some(PbTimestamp {
                    seconds: rule.updated_at.timestamp(),
                    nanos: rule.updated_at.timestamp_subsec_nanos() as i32,
                }),
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
            window_seconds: info.window_seconds,
            algorithm: info.algorithm,
            enabled: info.enabled,
            used: info.used,
            remaining: info.remaining,
            reset_at: info.reset_at,
        })
    }

    pub async fn update_rule(
        &self,
        req: UpdateRuleRequest,
    ) -> Result<UpdateRuleResponse, GrpcError> {
        let limit = u32::try_from(req.limit)
            .map_err(|_| GrpcError::InvalidArgument("limit must be >= 0".to_string()))?;
        let window_seconds = u32::try_from(req.window_seconds)
            .map_err(|_| GrpcError::InvalidArgument("window_seconds must be >= 0".to_string()))?;
        let input = UpdateRuleInput {
            id: req.rule_id,
            scope: req.scope,
            identifier_pattern: req.identifier_pattern,
            limit,
            window_seconds,
            algorithm: req.algorithm,
            enabled: req.enabled,
        };

        let rule = self.update_uc.execute(&input).await.map_err(|e| match e {
            crate::usecase::update_rule::UpdateRuleError::NotFound(msg) => GrpcError::NotFound(msg),
            crate::usecase::update_rule::UpdateRuleError::Validation(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::update_rule::UpdateRuleError::InvalidAlgorithm(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::update_rule::UpdateRuleError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(UpdateRuleResponse {
            rule: RuleResponse {
                id: rule.id.to_string(),
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: i64::from(rule.limit),
                window_seconds: i64::from(rule.window_seconds),
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: Some(PbTimestamp {
                    seconds: rule.created_at.timestamp(),
                    nanos: rule.created_at.timestamp_subsec_nanos() as i32,
                }),
                updated_at: Some(PbTimestamp {
                    seconds: rule.updated_at.timestamp(),
                    nanos: rule.updated_at.timestamp_subsec_nanos() as i32,
                }),
            },
        })
    }

    pub async fn delete_rule(
        &self,
        req: DeleteRuleRequest,
    ) -> Result<DeleteRuleResponse, GrpcError> {
        self.delete_uc.execute(&req.rule_id).await.map_err(|e| match e {
            crate::usecase::delete_rule::DeleteRuleError::NotFound(msg) => GrpcError::NotFound(msg),
            crate::usecase::delete_rule::DeleteRuleError::InvalidRuleId(msg) => {
                GrpcError::InvalidArgument(msg)
            }
            crate::usecase::delete_rule::DeleteRuleError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(DeleteRuleResponse { success: true })
    }

    pub async fn list_rules(
        &self,
        req: ListRulesRequest,
    ) -> Result<ListRulesResponse, GrpcError> {
        let page = if req.page == 0 { 1 } else { req.page };
        let page_size = if req.page_size == 0 { 20 } else { req.page_size };
        let output = self
            .list_uc
            .execute(&crate::usecase::list_rules::ListRulesInput {
                page,
                page_size,
                scope: if req.scope.is_empty() {
                    None
                } else {
                    Some(req.scope)
                },
                enabled_only: req.enabled_only.unwrap_or(false),
            })
            .await
            .map_err(|e| match e {
            crate::usecase::list_rules::ListRulesError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(ListRulesResponse {
            rules: output
                .rules
                .into_iter()
                .map(|rule| RuleResponse {
                    id: rule.id.to_string(),
                    scope: rule.scope,
                    identifier_pattern: rule.identifier_pattern,
                    limit: i64::from(rule.limit),
                    window_seconds: i64::from(rule.window_seconds),
                    algorithm: rule.algorithm.as_str().to_string(),
                    enabled: rule.enabled,
                    created_at: Some(PbTimestamp {
                        seconds: rule.created_at.timestamp(),
                        nanos: rule.created_at.timestamp_subsec_nanos() as i32,
                    }),
                    updated_at: Some(PbTimestamp {
                        seconds: rule.updated_at.timestamp(),
                        nanos: rule.updated_at.timestamp_subsec_nanos() as i32,
                    }),
                })
                .collect(),
            pagination: PaginationResponse {
                total_count: output.total_count.min(i32::MAX as u64) as i32,
                page: output.page.min(i32::MAX as u32) as i32,
                page_size: output.page_size.min(i32::MAX as u32) as i32,
                has_next: output.has_next,
            },
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
        let usage_uc =
            Arc::new(GetUsageUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let update_uc = Arc::new(UpdateRuleUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let delete_uc = Arc::new(DeleteRuleUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let list_uc = Arc::new(ListRulesUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let reset_uc = Arc::new(ResetRateLimitUseCase::new(Arc::new(MockRateLimitStateStore::new())));
        RateLimitGrpcService::new(
            check_uc,
            create_uc,
            get_uc,
            update_uc,
            delete_uc,
            list_uc,
            usage_uc,
            reset_uc,
        )
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
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 99, 1700000060)));

        let check_uc = Arc::new(CheckRateLimitUseCase::new(
            Arc::new(repo),
            Arc::new(state_store),
        ));
        let create_uc = Arc::new(CreateRuleUseCase::new(Arc::new(MockRateLimitRepository::new())));
        let get_uc = Arc::new(GetRuleUseCase::new(Arc::new(MockRateLimitRepository::new())));

        let svc = make_service_with(check_uc, create_uc, get_uc);
        let result = svc
            .check_rate_limit(CheckRateLimitRequest {
                scope: "service".to_string(),
                identifier: "user-123".to_string(),
                window: 60,
            })
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.allowed);
        assert_eq!(resp.remaining, 99);
    }

    #[tokio::test]
    async fn test_grpc_check_rate_limit_empty_scope() {
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
                scope: "".to_string(),
                identifier: "user-123".to_string(),
                window: 60,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GrpcError::InvalidArgument(_)));
    }
}
