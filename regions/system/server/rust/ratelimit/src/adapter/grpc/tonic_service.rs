// §2.2 監査対応: ADR-0034 dual-write パターンで deprecated な algorithm 文字列フィールドと
// 新 algorithm_enum フィールドを同時設定するため、このファイル全体で deprecated 警告を抑制する。
#![allow(deprecated)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use k1s0_auth::Claims;

use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
use crate::proto::k1s0::system::ratelimit::v1::{
    rate_limit_service_server::RateLimitService,
    CheckRateLimitRequest as ProtoCheckRateLimitRequest,
    CheckRateLimitResponse as ProtoCheckRateLimitResponse,
    CreateRuleRequest as ProtoCreateRuleRequest, CreateRuleResponse as ProtoCreateRuleResponse,
    DeleteRuleRequest as ProtoDeleteRuleRequest, DeleteRuleResponse as ProtoDeleteRuleResponse,
    GetRuleRequest as ProtoGetRuleRequest, GetRuleResponse as ProtoGetRuleResponse,
    GetUsageRequest as ProtoGetUsageRequest, GetUsageResponse as ProtoGetUsageResponse,
    ListRulesRequest as ProtoListRulesRequest, ListRulesResponse as ProtoListRulesResponse,
    RateLimitAlgorithm, RateLimitRule as ProtoRateLimitRule,
    ResetLimitRequest as ProtoResetLimitRequest, ResetLimitResponse as ProtoResetLimitResponse,
    UpdateRuleRequest as ProtoUpdateRuleRequest, UpdateRuleResponse as ProtoUpdateRuleResponse,
};

use super::ratelimit_grpc::{
    CheckRateLimitRequest, CreateRuleRequest, DeleteRuleRequest, GetRuleRequest, GetUsageRequest,
    GrpcError, ListRulesRequest, RateLimitGrpcService, ResetLimitRequest, UpdateRuleRequest,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

use crate::proto::k1s0::system::common::v1::Timestamp as ProtoTimestamp;

fn pb_timestamp(ts: &super::ratelimit_grpc::PbTimestamp) -> ProtoTimestamp {
    ProtoTimestamp {
        seconds: ts.seconds,
        nanos: ts.nanos,
    }
}

/// アルゴリズム文字列を RateLimitAlgorithm enum の i32 値に変換する。
/// dual-write パターンで旧文字列フィールドと新 enum フィールドを同時設定するために使用する。
fn algorithm_str_to_enum(s: &str) -> i32 {
    match s {
        "sliding_window" => RateLimitAlgorithm::SlidingWindow as i32,
        "token_bucket" => RateLimitAlgorithm::TokenBucket as i32,
        "fixed_window" => RateLimitAlgorithm::FixedWindow as i32,
        "leaky_bucket" => RateLimitAlgorithm::LeakyBucket as i32,
        _ => RateLimitAlgorithm::Unspecified as i32,
    }
}

// L-10 監査対応: algorithm_opt_to_enum は未使用のため削除

/// CRIT-005 対応: tonic Request の Extensions から Claims を取り出しテナント ID を返すヘルパー。
/// gRPC は JWT ベアラートークンを持たない場合があるため、Claims が存在しない場合はシステムテナント UUID をフォールバックとして使用する。
fn tenant_id_from_request<T>(request: &Request<T>) -> String {
    request
        .extensions()
        .get::<Claims>()
        .map(|c| c.tenant_id().to_string())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| "00000000-0000-0000-0000-000000000001".to_string())
}

pub struct RateLimitServiceTonic {
    inner: Arc<RateLimitGrpcService>,
}

impl RateLimitServiceTonic {
    pub fn new(inner: Arc<RateLimitGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl RateLimitService for RateLimitServiceTonic {
    async fn check_rate_limit(
        &self,
        request: Request<ProtoCheckRateLimitRequest>,
    ) -> Result<Response<ProtoCheckRateLimitResponse>, Status> {
        let inner = request.into_inner();
        // STATIC-CRITICAL-001: proto の CheckRateLimitRequest に tenant_id フィールドがないため None を渡す。
        // ratelimit_grpc.rs 内でシステムテナントUUID へフォールバックする。
        let req = CheckRateLimitRequest {
            tenant_id: None,
            scope: inner.scope,
            identifier: inner.identifier,
            window: inner.window,
        };

        let resp = self
            .inner
            .check_rate_limit(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCheckRateLimitResponse {
            allowed: resp.allowed,
            remaining: resp.remaining,
            reset_at: resp.reset_at,
            reason: resp.reason,
            limit: resp.limit,
            scope: resp.scope,
            identifier: resp.identifier,
            used: resp.used,
            rule_id: resp.rule_id,
        }))
    }

    async fn create_rule(
        &self,
        request: Request<ProtoCreateRuleRequest>,
    ) -> Result<Response<ProtoCreateRuleResponse>, Status> {
        // CRIT-005 対応: JWT Claims からテナント ID を抽出してリクエストに設定する。
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let req = CreateRuleRequest {
            scope: inner.scope,
            identifier_pattern: inner.identifier_pattern,
            limit: inner.limit,
            window_seconds: inner.window_seconds,
            algorithm: None,
            enabled: inner.enabled,
            tenant_id,
        };

        let resp = self
            .inner
            .create_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        // dual-write: 旧文字列フィールドと新 enum フィールドを同時設定して後方互換性維持
        let algorithm_enum = algorithm_str_to_enum(&resp.rule.algorithm);
        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            name: resp.rule.name,
            scope: resp.rule.scope,
            identifier_pattern: resp.rule.identifier_pattern,
            limit: resp.rule.limit,
            window_seconds: resp.rule.window_seconds,
            algorithm: resp.rule.algorithm,
            algorithm_enum,
            enabled: resp.rule.enabled,
            created_at: resp.rule.created_at.map(|ts| pb_timestamp(&ts)),
            updated_at: resp.rule.updated_at.map(|ts| pb_timestamp(&ts)),
        };

        Ok(Response::new(ProtoCreateRuleResponse {
            rule: Some(proto_rule),
        }))
    }

    async fn get_rule(
        &self,
        request: Request<ProtoGetRuleRequest>,
    ) -> Result<Response<ProtoGetRuleResponse>, Status> {
        // CRIT-005 対応: JWT Claims からテナント ID を抽出してリクエストに設定する。
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let req = GetRuleRequest {
            rule_id: inner.rule_id,
            tenant_id,
        };

        let resp = self
            .inner
            .get_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        // dual-write: 旧文字列フィールドと新 enum フィールドを同時設定して後方互換性維持
        let algorithm_enum = algorithm_str_to_enum(&resp.rule.algorithm);
        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            name: resp.rule.name,
            scope: resp.rule.scope,
            identifier_pattern: resp.rule.identifier_pattern,
            limit: resp.rule.limit,
            window_seconds: resp.rule.window_seconds,
            algorithm: resp.rule.algorithm,
            algorithm_enum,
            enabled: resp.rule.enabled,
            created_at: resp.rule.created_at.map(|ts| pb_timestamp(&ts)),
            updated_at: resp.rule.updated_at.map(|ts| pb_timestamp(&ts)),
        };

        Ok(Response::new(ProtoGetRuleResponse {
            rule: Some(proto_rule),
        }))
    }

    async fn get_usage(
        &self,
        request: Request<ProtoGetUsageRequest>,
    ) -> Result<Response<ProtoGetUsageResponse>, Status> {
        let inner = request.into_inner();
        // STATIC-CRITICAL-001: proto の GetUsageRequest に tenant_id フィールドがないため None を渡す。
        // ratelimit_grpc.rs 内でシステムテナントUUID へフォールバックする。
        let req = GetUsageRequest {
            tenant_id: None,
            rule_id: inner.rule_id,
        };

        let resp = self
            .inner
            .get_usage(req)
            .await
            .map_err(Into::<Status>::into)?;

        // dual-write: 旧文字列フィールドと新 enum フィールドを同時設定して後方互換性維持
        let algorithm_enum = algorithm_str_to_enum(&resp.algorithm);
        Ok(Response::new(ProtoGetUsageResponse {
            rule_id: resp.rule_id,
            rule_name: resp.rule_name,
            limit: resp.limit,
            window_seconds: resp.window_seconds,
            algorithm: resp.algorithm,
            algorithm_enum,
            enabled: resp.enabled,
            used: resp.used,
            remaining: resp.remaining,
            reset_at: resp.reset_at,
        }))
    }

    async fn update_rule(
        &self,
        request: Request<ProtoUpdateRuleRequest>,
    ) -> Result<Response<ProtoUpdateRuleResponse>, Status> {
        // CRIT-005 対応: JWT Claims からテナント ID を抽出してリクエストに設定する。
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let req = UpdateRuleRequest {
            rule_id: inner.rule_id,
            scope: inner.scope,
            identifier_pattern: inner.identifier_pattern,
            limit: inner.limit,
            window_seconds: inner.window_seconds,
            algorithm: None,
            enabled: inner.enabled,
            tenant_id,
        };

        let resp = self
            .inner
            .update_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        // dual-write: 旧文字列フィールドと新 enum フィールドを同時設定して後方互換性維持
        let algorithm_enum = algorithm_str_to_enum(&resp.rule.algorithm);
        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            name: resp.rule.name,
            scope: resp.rule.scope,
            identifier_pattern: resp.rule.identifier_pattern,
            limit: resp.rule.limit,
            window_seconds: resp.rule.window_seconds,
            algorithm: resp.rule.algorithm,
            algorithm_enum,
            enabled: resp.rule.enabled,
            created_at: resp.rule.created_at.map(|ts| pb_timestamp(&ts)),
            updated_at: resp.rule.updated_at.map(|ts| pb_timestamp(&ts)),
        };

        Ok(Response::new(ProtoUpdateRuleResponse {
            rule: Some(proto_rule),
        }))
    }

    async fn delete_rule(
        &self,
        request: Request<ProtoDeleteRuleRequest>,
    ) -> Result<Response<ProtoDeleteRuleResponse>, Status> {
        // CRIT-005 対応: JWT Claims からテナント ID を抽出してリクエストに設定する。
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        let req = DeleteRuleRequest {
            rule_id: inner.rule_id,
            tenant_id,
        };
        let resp = self
            .inner
            .delete_rule(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteRuleResponse {
            success: resp.success,
        }))
    }

    async fn list_rules(
        &self,
        request: Request<ProtoListRulesRequest>,
    ) -> Result<Response<ProtoListRulesResponse>, Status> {
        // CRIT-005 対応: JWT Claims からテナント ID を抽出してリクエストに設定する。
        let tenant_id = tenant_id_from_request(&request);
        let inner = request.into_inner();
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = inner.pagination.unwrap_or_default();
        let resp = self
            .inner
            .list_rules(ListRulesRequest {
                scope: inner.scope,
                enabled_only: inner.enabled_only,
                page: if pagination.page <= 0 {
                    1
                } else {
                    pagination.page as u32
                },
                page_size: if pagination.page_size <= 0 {
                    20
                } else {
                    pagination.page_size as u32
                },
                tenant_id,
            })
            .await
            .map_err(Into::<Status>::into)?;
        let rules = resp
            .rules
            .into_iter()
            // dual-write: 旧文字列フィールドと新 enum フィールドを同時設定して後方互換性維持
            .map(|rule| {
                let algorithm_enum = algorithm_str_to_enum(&rule.algorithm);
                ProtoRateLimitRule {
                    id: rule.id,
                    name: rule.name,
                    scope: rule.scope,
                    identifier_pattern: rule.identifier_pattern,
                    limit: rule.limit,
                    window_seconds: rule.window_seconds,
                    algorithm: rule.algorithm,
                    algorithm_enum,
                    enabled: rule.enabled,
                    created_at: rule.created_at.map(|ts| pb_timestamp(&ts)),
                    updated_at: rule.updated_at.map(|ts| pb_timestamp(&ts)),
                }
            })
            .collect();
        Ok(Response::new(ProtoListRulesResponse {
            rules,
            pagination: Some(ProtoPaginationResult {
                total_count: resp.pagination.total_count,
                page: resp.pagination.page,
                page_size: resp.pagination.page_size,
                has_next: resp.pagination.has_next,
            }),
        }))
    }

    async fn reset_limit(
        &self,
        request: Request<ProtoResetLimitRequest>,
    ) -> Result<Response<ProtoResetLimitResponse>, Status> {
        let inner = request.into_inner();
        // STATIC-CRITICAL-001: proto の ResetLimitRequest に tenant_id フィールドがないため None を渡す。
        // ratelimit_grpc.rs 内でシステムテナントUUID へフォールバックする。
        let req = ResetLimitRequest {
            tenant_id: None,
            scope: inner.scope,
            identifier: inner.identifier,
        };

        let resp = self
            .inner
            .reset_limit(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoResetLimitResponse {
            success: resp.success,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("rule not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("scope is required".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
