use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::rule_engine::v1::{
    rule_engine_service_server::RuleEngineService, CreateRuleRequest as ProtoCreateRuleRequest,
    CreateRuleResponse as ProtoCreateRuleResponse,
    CreateRuleSetRequest as ProtoCreateRuleSetRequest,
    CreateRuleSetResponse as ProtoCreateRuleSetResponse,
    DeleteRuleRequest as ProtoDeleteRuleRequest, DeleteRuleResponse as ProtoDeleteRuleResponse,
    DeleteRuleSetRequest as ProtoDeleteRuleSetRequest,
    DeleteRuleSetResponse as ProtoDeleteRuleSetResponse,
    EvaluateDryRunRequest as ProtoEvaluateDryRunRequest,
    EvaluateDryRunResponse as ProtoEvaluateDryRunResponse, EvaluateRequest as ProtoEvaluateRequest,
    EvaluateResponse as ProtoEvaluateResponse, GetRuleRequest as ProtoGetRuleRequest,
    GetRuleResponse as ProtoGetRuleResponse, GetRuleSetRequest as ProtoGetRuleSetRequest,
    GetRuleSetResponse as ProtoGetRuleSetResponse, ListRuleSetsRequest as ProtoListRuleSetsRequest,
    ListRuleSetsResponse as ProtoListRuleSetsResponse, ListRulesRequest as ProtoListRulesRequest,
    ListRulesResponse as ProtoListRulesResponse, MatchedRule as ProtoMatchedRule,
    PublishRuleSetRequest as ProtoPublishRuleSetRequest,
    PublishRuleSetResponse as ProtoPublishRuleSetResponse,
    RollbackRuleSetRequest as ProtoRollbackRuleSetRequest,
    RollbackRuleSetResponse as ProtoRollbackRuleSetResponse, Rule as ProtoRule,
    RuleSet as ProtoRuleSet, UpdateRuleRequest as ProtoUpdateRuleRequest,
    UpdateRuleResponse as ProtoUpdateRuleResponse,
    UpdateRuleSetRequest as ProtoUpdateRuleSetRequest,
    UpdateRuleSetResponse as ProtoUpdateRuleSetResponse,
};

use super::rule_engine_grpc::{GrpcError, RuleData, RuleEngineGrpcService, RuleSetData};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> Option<ProtoTimestamp> {
    Some(ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

fn to_proto_rule(rule: RuleData) -> ProtoRule {
    ProtoRule {
        id: rule.id,
        name: rule.name,
        description: rule.description,
        priority: rule.priority,
        when_json: rule.when_json,
        then_json: rule.then_json,
        enabled: rule.enabled,
        version: rule.version,
        created_at: to_proto_timestamp(rule.created_at),
        updated_at: to_proto_timestamp(rule.updated_at),
    }
}

fn to_proto_rule_set(rs: RuleSetData) -> ProtoRuleSet {
    ProtoRuleSet {
        id: rs.id,
        name: rs.name,
        description: rs.description,
        domain: rs.domain,
        evaluation_mode: rs.evaluation_mode,
        default_result_json: rs.default_result_json,
        rule_ids: rs.rule_ids,
        current_version: rs.current_version,
        enabled: rs.enabled,
        created_at: to_proto_timestamp(rs.created_at),
        updated_at: to_proto_timestamp(rs.updated_at),
    }
}

pub struct RuleEngineServiceTonic {
    inner: Arc<RuleEngineGrpcService>,
}

impl RuleEngineServiceTonic {
    pub fn new(inner: Arc<RuleEngineGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl RuleEngineService for RuleEngineServiceTonic {
    async fn get_rule(
        &self,
        request: Request<ProtoGetRuleRequest>,
    ) -> Result<Response<ProtoGetRuleResponse>, Status> {
        let inner = request.into_inner();
        let data = self
            .inner
            .get_rule(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetRuleResponse {
            rule: Some(to_proto_rule(data)),
        }))
    }

    async fn list_rules(
        &self,
        request: Request<ProtoListRulesRequest>,
    ) -> Result<Response<ProtoListRulesResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));

        let (rules, total_count, p, ps, has_next) = self
            .inner
            .list_rules(page, page_size, inner.rule_set_id, inner.domain)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListRulesResponse {
            rules: rules.into_iter().map(to_proto_rule).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total_count as i64,
                page: p,
                page_size: ps,
                has_next,
            }),
        }))
    }

    async fn create_rule(
        &self,
        request: Request<ProtoCreateRuleRequest>,
    ) -> Result<Response<ProtoCreateRuleResponse>, Status> {
        // CRITICAL-RUST-001 監査対応: gRPC メタデータから x-tenant-id を取得して RLS に使用する。
        let tenant_id = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("system")
            .to_string();
        let inner = request.into_inner();
        let data = self
            .inner
            .create_rule(
                tenant_id,
                inner.name,
                inner.description,
                inner.priority,
                inner.when_json,
                inner.then_json,
            )
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCreateRuleResponse {
            rule: Some(to_proto_rule(data)),
        }))
    }

    async fn update_rule(
        &self,
        request: Request<ProtoUpdateRuleRequest>,
    ) -> Result<Response<ProtoUpdateRuleResponse>, Status> {
        let inner = request.into_inner();
        let data = self
            .inner
            .update_rule(
                inner.id,
                inner.description,
                inner.priority,
                inner.when_json,
                inner.then_json,
                inner.enabled,
            )
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoUpdateRuleResponse {
            rule: Some(to_proto_rule(data)),
        }))
    }

    async fn delete_rule(
        &self,
        request: Request<ProtoDeleteRuleRequest>,
    ) -> Result<Response<ProtoDeleteRuleResponse>, Status> {
        let inner = request.into_inner();
        let (success, message) = self
            .inner
            .delete_rule(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteRuleResponse { success, message }))
    }

    async fn get_rule_set(
        &self,
        request: Request<ProtoGetRuleSetRequest>,
    ) -> Result<Response<ProtoGetRuleSetResponse>, Status> {
        let inner = request.into_inner();
        let data = self
            .inner
            .get_rule_set(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetRuleSetResponse {
            rule_set: Some(to_proto_rule_set(data)),
        }))
    }

    async fn list_rule_sets(
        &self,
        request: Request<ProtoListRuleSetsRequest>,
    ) -> Result<Response<ProtoListRuleSetsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));

        let (rule_sets, total_count, p, ps, has_next) = self
            .inner
            .list_rule_sets(page, page_size, inner.domain)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListRuleSetsResponse {
            rule_sets: rule_sets.into_iter().map(to_proto_rule_set).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total_count as i64,
                page: p,
                page_size: ps,
                has_next,
            }),
        }))
    }

    async fn create_rule_set(
        &self,
        request: Request<ProtoCreateRuleSetRequest>,
    ) -> Result<Response<ProtoCreateRuleSetResponse>, Status> {
        // CRITICAL-RUST-001 監査対応: gRPC メタデータから x-tenant-id を取得して RLS に使用する。
        let tenant_id = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("system")
            .to_string();
        let inner = request.into_inner();
        let data = self
            .inner
            .create_rule_set(
                tenant_id,
                inner.name,
                inner.description,
                inner.domain,
                inner.evaluation_mode,
                inner.default_result_json,
                inner.rule_ids,
            )
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCreateRuleSetResponse {
            rule_set: Some(to_proto_rule_set(data)),
        }))
    }

    async fn update_rule_set(
        &self,
        request: Request<ProtoUpdateRuleSetRequest>,
    ) -> Result<Response<ProtoUpdateRuleSetResponse>, Status> {
        let inner = request.into_inner();
        let data = self
            .inner
            .update_rule_set(
                inner.id,
                inner.description,
                inner.evaluation_mode,
                inner.default_result_json,
                inner.rule_ids,
                inner.enabled,
            )
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoUpdateRuleSetResponse {
            rule_set: Some(to_proto_rule_set(data)),
        }))
    }

    async fn delete_rule_set(
        &self,
        request: Request<ProtoDeleteRuleSetRequest>,
    ) -> Result<Response<ProtoDeleteRuleSetResponse>, Status> {
        let inner = request.into_inner();
        let (success, message) = self
            .inner
            .delete_rule_set(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteRuleSetResponse {
            success,
            message,
        }))
    }

    async fn publish_rule_set(
        &self,
        request: Request<ProtoPublishRuleSetRequest>,
    ) -> Result<Response<ProtoPublishRuleSetResponse>, Status> {
        let inner = request.into_inner();
        let (id, published_version, previous_version, published_at) = self
            .inner
            .publish_rule_set(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoPublishRuleSetResponse {
            id,
            published_version,
            previous_version,
            published_at: to_proto_timestamp(published_at),
        }))
    }

    async fn rollback_rule_set(
        &self,
        request: Request<ProtoRollbackRuleSetRequest>,
    ) -> Result<Response<ProtoRollbackRuleSetResponse>, Status> {
        let inner = request.into_inner();
        let (id, rolled_back_to_version, previous_version, rolled_back_at) = self
            .inner
            .rollback_rule_set(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoRollbackRuleSetResponse {
            id,
            rolled_back_to_version,
            previous_version,
            rolled_back_at: to_proto_timestamp(rolled_back_at),
        }))
    }

    async fn evaluate(
        &self,
        request: Request<ProtoEvaluateRequest>,
    ) -> Result<Response<ProtoEvaluateResponse>, Status> {
        // CRITICAL-RUST-001 監査対応: gRPC メタデータから x-tenant-id を取得して RLS に使用する。
        let tenant_id = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("system")
            .to_string();
        let inner = request.into_inner();
        let output = self
            .inner
            .evaluate(tenant_id, inner.rule_set, inner.input_json, inner.context_json, false)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(to_proto_evaluate_response(output)))
    }

    async fn evaluate_dry_run(
        &self,
        request: Request<ProtoEvaluateDryRunRequest>,
    ) -> Result<Response<ProtoEvaluateDryRunResponse>, Status> {
        // CRITICAL-RUST-001 監査対応: gRPC メタデータから x-tenant-id を取得して RLS に使用する。
        let tenant_id = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("system")
            .to_string();
        let inner = request.into_inner();
        let output = self
            .inner
            .evaluate(tenant_id, inner.rule_set, inner.input_json, inner.context_json, true)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(to_proto_evaluate_dry_run_response(output)))
    }
}

// 評価結果を EvaluateResponse に変換する
fn to_proto_evaluate_response(
    output: crate::usecase::evaluate::EvaluateOutput,
) -> ProtoEvaluateResponse {
    ProtoEvaluateResponse {
        evaluation_id: output.evaluation_id.to_string(),
        rule_set: output.rule_set,
        rule_set_version: output.rule_set_version,
        matched_rules: output
            .matched_rules
            .into_iter()
            .map(|m| ProtoMatchedRule {
                id: m.id.to_string(),
                name: m.name,
                priority: m.priority,
                result_json: serde_json::to_vec(&m.result).unwrap_or_default(),
            })
            .collect(),
        result_json: serde_json::to_vec(&output.result).unwrap_or_default(),
        default_applied: output.default_applied,
        cached: output.cached,
        evaluated_at: to_proto_timestamp(output.evaluated_at),
    }
}

// ドライラン評価結果を EvaluateDryRunResponse に変換する
fn to_proto_evaluate_dry_run_response(
    output: crate::usecase::evaluate::EvaluateOutput,
) -> ProtoEvaluateDryRunResponse {
    ProtoEvaluateDryRunResponse {
        evaluation_id: output.evaluation_id.to_string(),
        rule_set: output.rule_set,
        rule_set_version: output.rule_set_version,
        matched_rules: output
            .matched_rules
            .into_iter()
            .map(|m| ProtoMatchedRule {
                id: m.id.to_string(),
                name: m.name,
                priority: m.priority,
                result_json: serde_json::to_vec(&m.result).unwrap_or_default(),
            })
            .collect(),
        result_json: serde_json::to_vec(&output.result).unwrap_or_default(),
        default_applied: output.default_applied,
        cached: output.cached,
        evaluated_at: to_proto_timestamp(output.evaluated_at),
    }
}
