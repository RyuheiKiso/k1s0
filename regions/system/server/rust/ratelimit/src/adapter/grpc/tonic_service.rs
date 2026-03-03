use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::ratelimit::v1::{
    rate_limit_service_server::RateLimitService,
    CheckRateLimitRequest as ProtoCheckRateLimitRequest,
    CheckRateLimitResponse as ProtoCheckRateLimitResponse,
    CreateRuleRequest as ProtoCreateRuleRequest, CreateRuleResponse as ProtoCreateRuleResponse,
    DeleteRuleRequest as ProtoDeleteRuleRequest, DeleteRuleResponse as ProtoDeleteRuleResponse,
    GetRuleRequest as ProtoGetRuleRequest, GetRuleResponse as ProtoGetRuleResponse,
    GetUsageRequest as ProtoGetUsageRequest, GetUsageResponse as ProtoGetUsageResponse,
    ListRulesRequest as ProtoListRulesRequest, ListRulesResponse as ProtoListRulesResponse,
    RateLimitRule as ProtoRateLimitRule,
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

fn pb_timestamp(ts: &super::ratelimit_grpc::PbTimestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: ts.seconds,
        nanos: ts.nanos,
    }
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
        let req = CheckRateLimitRequest {
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
        }))
    }

    async fn create_rule(
        &self,
        request: Request<ProtoCreateRuleRequest>,
    ) -> Result<Response<ProtoCreateRuleResponse>, Status> {
        let inner = request.into_inner();
        let req = CreateRuleRequest {
            scope: inner.scope,
            identifier_pattern: inner.identifier_pattern,
            limit: inner.limit,
            window_seconds: inner.window_seconds,
            enabled: inner.enabled,
        };

        let resp = self
            .inner
            .create_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            scope: resp.rule.scope,
            identifier_pattern: resp.rule.identifier_pattern,
            limit: resp.rule.limit,
            window_seconds: resp.rule.window_seconds,
            algorithm: resp.rule.algorithm,
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
        let inner = request.into_inner();
        let req = GetRuleRequest {
            rule_id: inner.rule_id,
        };

        let resp = self
            .inner
            .get_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            scope: resp.rule.scope,
            identifier_pattern: resp.rule.identifier_pattern,
            limit: resp.rule.limit,
            window_seconds: resp.rule.window_seconds,
            algorithm: resp.rule.algorithm,
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
        let req = GetUsageRequest {
            rule_id: inner.rule_id,
        };

        let resp = self
            .inner
            .get_usage(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetUsageResponse {
            rule_id: resp.rule_id,
            rule_name: resp.rule_name,
            limit: resp.limit,
            window_secs: resp.window_seconds,
            algorithm: resp.algorithm,
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
        let inner = request.into_inner();
        let req = UpdateRuleRequest {
            rule_id: inner.rule_id,
            scope: inner.scope,
            identifier_pattern: inner.identifier_pattern,
            limit: inner.limit,
            window_seconds: inner.window_seconds,
            enabled: inner.enabled,
        };

        let resp = self
            .inner
            .update_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            scope: resp.rule.scope,
            identifier_pattern: resp.rule.identifier_pattern,
            limit: resp.rule.limit,
            window_seconds: resp.rule.window_seconds,
            algorithm: resp.rule.algorithm,
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
        let inner = request.into_inner();
        let req = DeleteRuleRequest {
            rule_id: inner.rule_id,
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
        _request: Request<ProtoListRulesRequest>,
    ) -> Result<Response<ProtoListRulesResponse>, Status> {
        let resp = self
            .inner
            .list_rules(ListRulesRequest {})
            .await
            .map_err(Into::<Status>::into)?;
        let rules = resp
            .rules
            .into_iter()
            .map(|rule| ProtoRateLimitRule {
                id: rule.id,
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: rule.limit,
                window_seconds: rule.window_seconds,
                algorithm: rule.algorithm,
                enabled: rule.enabled,
                created_at: rule.created_at.map(|ts| pb_timestamp(&ts)),
                updated_at: rule.updated_at.map(|ts| pb_timestamp(&ts)),
            })
            .collect();
        Ok(Response::new(ProtoListRulesResponse { rules }))
    }

    async fn reset_limit(
        &self,
        request: Request<ProtoResetLimitRequest>,
    ) -> Result<Response<ProtoResetLimitResponse>, Status> {
        let inner = request.into_inner();
        let req = ResetLimitRequest {
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
