//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の RateLimitService トレイトを実装する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::ratelimit::v1::{
    rate_limit_service_server::RateLimitService,
    CheckRateLimitRequest as ProtoCheckRateLimitRequest,
    CheckRateLimitResponse as ProtoCheckRateLimitResponse,
    CreateRuleRequest as ProtoCreateRuleRequest, CreateRuleResponse as ProtoCreateRuleResponse,
    GetRuleRequest as ProtoGetRuleRequest, GetRuleResponse as ProtoGetRuleResponse,
    RateLimitRule as ProtoRateLimitRule,
};

use super::ratelimit_grpc::{
    CheckRateLimitRequest, CreateRuleRequest, GetRuleRequest, GrpcError, RateLimitGrpcService,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- 変換ヘルパー ---

fn pb_timestamp(ts: &super::ratelimit_grpc::PbTimestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: ts.seconds,
        nanos: ts.nanos,
    }
}

/// RateLimitServiceTonic は tonic の RateLimitService として RateLimitGrpcService をラップする。
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
            rule_id: inner.rule_id,
            subject: inner.subject,
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
            name: inner.name,
            key: inner.key,
            limit: inner.limit,
            window_secs: inner.window_secs,
            algorithm: inner.algorithm,
        };

        let resp = self
            .inner
            .create_rule(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_rule = ProtoRateLimitRule {
            id: resp.rule.id,
            name: resp.rule.name,
            key: resp.rule.key,
            limit: resp.rule.limit,
            window_secs: resp.rule.window_secs,
            algorithm: resp.rule.algorithm,
            enabled: resp.rule.enabled,
            created_at: resp.rule.created_at.map(|ts| pb_timestamp(&ts)),
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
            name: resp.rule.name,
            key: resp.rule.key,
            limit: resp.rule.limit,
            window_secs: resp.rule.window_secs,
            algorithm: resp.rule.algorithm,
            enabled: resp.rule.enabled,
            created_at: resp.rule.created_at.map(|ts| pb_timestamp(&ts)),
        };

        Ok(Response::new(ProtoGetRuleResponse {
            rule: Some(proto_rule),
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
        let err = GrpcError::InvalidArgument("rule_id is required".to_string());
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
