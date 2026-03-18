use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::usecase::create_rule::CreateRuleInput;
use crate::usecase::create_rule_set::CreateRuleSetInput;
use crate::usecase::evaluate::EvaluateInput;
use crate::usecase::list_evaluation_logs::ListEvaluationLogsInput;
use crate::usecase::list_rule_sets::ListRuleSetsInput;
use crate::usecase::list_rules::ListRulesInput;
use crate::usecase::update_rule::UpdateRuleInput;
use crate::usecase::update_rule_set::UpdateRuleSetInput;

// --- Rules ---

pub async fn list_rules(
    State(state): State<AppState>,
    Query(params): Query<ListRulesParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);
    let rule_set_id = match params.rule_set_id {
        Some(ref id) => match Uuid::parse_str(id) {
            Ok(uid) => Some(uid),
            Err(_) => {
                let err =
                    ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", "invalid rule_set_id format");
                return (StatusCode::BAD_REQUEST, Json(err)).into_response();
            }
        },
        None => None,
    };

    let input = ListRulesInput {
        page,
        page_size,
        rule_set_id,
        domain: params.domain,
    };

    match state.list_rules_uc.execute(&input).await {
        Ok(output) => {
            let items: Vec<RuleResponse> =
                output.rules.into_iter().map(RuleResponse::from).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "rules": items,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_rule(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.get_rule_uc.execute(&id).await {
        Ok(Some(rule)) => {
            let resp = RuleResponse::from(rule);
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("ルールレスポンスのJSON変換に失敗"))).into_response()
        }
        Ok(None) => {
            let err = ErrorResponse::new("SYS_RULE_NOT_FOUND", &format!("rule not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> impl IntoResponse {
    let input = CreateRuleInput {
        name: req.name,
        description: req.description,
        priority: req.priority,
        when_condition: req.when_condition,
        then_result: req.then_result,
    };

    match state.create_rule_uc.execute(&input).await {
        Ok(rule) => {
            let resp = RuleResponse::from(rule);
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).expect("ルールレスポンスのJSON変換に失敗")),
            )
                .into_response()
        }
        Err(crate::usecase::create_rule::CreateRuleError::AlreadyExists(name)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_ALREADY_EXISTS",
                &format!("rule already exists: {}", name),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(crate::usecase::create_rule::CreateRuleError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::create_rule::CreateRuleError::InvalidCondition(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INVALID_CONDITION", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::create_rule::CreateRuleError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRuleRequest>,
) -> impl IntoResponse {
    let input = UpdateRuleInput {
        id,
        description: req.description,
        priority: req.priority,
        when_condition: req.when_condition,
        then_result: req.then_result,
        enabled: req.enabled,
    };

    match state.update_rule_uc.execute(&input).await {
        Ok(rule) => {
            let resp = RuleResponse::from(rule);
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("ルールレスポンスのJSON変換に失敗"))).into_response()
        }
        Err(crate::usecase::update_rule::UpdateRuleError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_RULE_NOT_FOUND", &format!("rule not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::update_rule::UpdateRuleError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::update_rule::UpdateRuleError::InvalidCondition(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INVALID_CONDITION", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::update_rule::UpdateRuleError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn delete_rule(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.delete_rule_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("rule {} deleted", id)
            })),
        )
            .into_response(),
        Err(crate::usecase::delete_rule::DeleteRuleError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_RULE_NOT_FOUND", &format!("rule not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::delete_rule::DeleteRuleError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Rule Sets ---

pub async fn list_rule_sets(
    State(state): State<AppState>,
    Query(params): Query<ListRuleSetsParams>,
) -> impl IntoResponse {
    let input = ListRuleSetsInput {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        domain: params.domain,
    };

    match state.list_rule_sets_uc.execute(&input).await {
        Ok(output) => {
            let items: Vec<RuleSetResponse> = output
                .rule_sets
                .into_iter()
                .map(RuleSetResponse::from)
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "rule_sets": items,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_rule_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_rule_set_uc.execute(&id).await {
        Ok(Some(rs)) => {
            let resp = RuleSetResponse::from(rs);
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("ルールレスポンスのJSON変換に失敗"))).into_response()
        }
        Ok(None) => {
            let err = ErrorResponse::new(
                "SYS_RULE_SET_NOT_FOUND",
                &format!("rule set not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn create_rule_set(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleSetRequestBody>,
) -> impl IntoResponse {
    let rule_ids: Result<Vec<Uuid>, _> = req.rule_ids.iter().map(|s| Uuid::parse_str(s)).collect();
    let rule_ids = match rule_ids {
        Ok(ids) => ids,
        Err(_) => {
            let err = ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", "invalid rule_ids format");
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    let input = CreateRuleSetInput {
        name: req.name,
        description: req.description,
        domain: req.domain,
        evaluation_mode: req.evaluation_mode,
        default_result: req.default_result,
        rule_ids,
    };

    match state.create_rule_set_uc.execute(&input).await {
        Ok(rs) => {
            let resp = RuleSetResponse::from(rs);
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).expect("ルールレスポンスのJSON変換に失敗")),
            )
                .into_response()
        }
        Err(crate::usecase::create_rule_set::CreateRuleSetError::AlreadyExists(name)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_ALREADY_EXISTS",
                &format!("rule set already exists: {}", name),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(crate::usecase::create_rule_set::CreateRuleSetError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::create_rule_set::CreateRuleSetError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn update_rule_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRuleSetRequestBody>,
) -> impl IntoResponse {
    let rule_ids = match req.rule_ids {
        Some(ref ids) => {
            let parsed: Result<Vec<Uuid>, _> = ids.iter().map(|s| Uuid::parse_str(s)).collect();
            match parsed {
                Ok(uuids) => Some(uuids),
                Err(_) => {
                    let err =
                        ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", "invalid rule_ids format");
                    return (StatusCode::BAD_REQUEST, Json(err)).into_response();
                }
            }
        }
        None => None,
    };

    let input = UpdateRuleSetInput {
        id,
        description: req.description,
        evaluation_mode: req.evaluation_mode,
        default_result: req.default_result,
        rule_ids,
        enabled: req.enabled,
    };

    match state.update_rule_set_uc.execute(&input).await {
        Ok(rs) => {
            let resp = RuleSetResponse::from(rs);
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("ルールレスポンスのJSON変換に失敗"))).into_response()
        }
        Err(crate::usecase::update_rule_set::UpdateRuleSetError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_SET_NOT_FOUND",
                &format!("rule set not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::update_rule_set::UpdateRuleSetError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::update_rule_set::UpdateRuleSetError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn delete_rule_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.delete_rule_set_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("rule set {} deleted", id)
            })),
        )
            .into_response(),
        Err(crate::usecase::delete_rule_set::DeleteRuleSetError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_SET_NOT_FOUND",
                &format!("rule set not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::delete_rule_set::DeleteRuleSetError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Publish / Rollback ---

pub async fn publish_rule_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.publish_rule_set_uc.execute(&id).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": output.id.to_string(),
                "name": output.name,
                "published_version": output.published_version,
                "previous_version": output.previous_version,
                "published_at": output.published_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(crate::usecase::publish_rule_set::PublishRuleSetError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_SET_NOT_FOUND",
                &format!("rule set not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::publish_rule_set::PublishRuleSetError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn rollback_rule_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.rollback_rule_set_uc.execute(&id).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": output.id.to_string(),
                "name": output.name,
                "rolled_back_to_version": output.rolled_back_to_version,
                "previous_version": output.previous_version,
                "rolled_back_at": output.rolled_back_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(crate::usecase::rollback_rule_set::RollbackRuleSetError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_SET_NOT_FOUND",
                &format!("rule set not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::rollback_rule_set::RollbackRuleSetError::NoPreviousVersion(v)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_NO_PREVIOUS_VERSION",
                &format!("no previous version to rollback: current version is {}", v),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(crate::usecase::rollback_rule_set::RollbackRuleSetError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Evaluate ---

pub async fn evaluate(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequestBody>,
) -> impl IntoResponse {
    let input = EvaluateInput {
        rule_set: req.rule_set,
        input: req.input,
        context: req.context.unwrap_or(serde_json::json!({})),
        dry_run: false,
    };
    evaluate_inner(state, input).await
}

pub async fn evaluate_dry_run(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequestBody>,
) -> impl IntoResponse {
    let input = EvaluateInput {
        rule_set: req.rule_set,
        input: req.input,
        context: req.context.unwrap_or(serde_json::json!({})),
        dry_run: true,
    };
    evaluate_inner(state, input).await
}

async fn evaluate_inner(state: AppState, input: EvaluateInput) -> impl IntoResponse {
    match state.evaluate_uc.execute(&input).await {
        Ok(output) => {
            let matched_rules: Vec<serde_json::Value> = output
                .matched_rules
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id.to_string(),
                        "name": m.name,
                        "priority": m.priority,
                        "result": m.result
                    })
                })
                .collect();

            let mut resp = serde_json::json!({
                "evaluation_id": output.evaluation_id.to_string(),
                "rule_set": output.rule_set,
                "rule_set_version": output.rule_set_version,
                "result": output.result,
                "cached": output.cached,
                "evaluated_at": output.evaluated_at.to_rfc3339()
            });

            if output.default_applied {
                resp["default_applied"] = serde_json::json!(true);
                resp["matched_rule"] = serde_json::Value::Null;
            } else if matched_rules.len() == 1 {
                resp["matched_rule"] = matched_rules[0].clone();
            } else {
                resp["matched_rules"] = serde_json::Value::Array(matched_rules);
            }

            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(crate::usecase::evaluate::EvaluateError::RuleSetNotFound(name)) => {
            let err = ErrorResponse::new(
                "SYS_RULE_SET_NOT_FOUND",
                &format!("rule set not found: {}", name),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::evaluate::EvaluateError::EvaluationError(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_EVALUATION_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
        Err(crate::usecase::evaluate::EvaluateError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Evaluation Logs ---

pub async fn list_evaluation_logs(
    State(state): State<AppState>,
    Query(params): Query<ListEvaluationLogsParams>,
) -> impl IntoResponse {
    let from = params
        .from
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let to = params
        .to
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let input = ListEvaluationLogsInput {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        rule_set_name: params.rule_set,
        domain: params.domain,
        from,
        to,
    };

    match state.list_evaluation_logs_uc.execute(&input).await {
        Ok(output) => {
            let items: Vec<serde_json::Value> = output
                .logs
                .into_iter()
                .map(|log| {
                    serde_json::json!({
                        "evaluation_id": log.id.to_string(),
                        "rule_set": log.rule_set_name,
                        "rule_set_version": log.rule_set_version,
                        "matched_rule_id": log.matched_rule_id.map(|id| id.to_string()),
                        "input_hash": log.input_hash,
                        "result": log.result,
                        "context": log.context,
                        "evaluated_at": log.evaluated_at.to_rfc3339()
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "logs": items,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_RULE_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct ListRulesParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub rule_set_id: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRuleSetsParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListEvaluationLogsParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub rule_set: Option<String>,
    pub domain: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: String,
    pub priority: i32,
    #[serde(rename = "when")]
    pub when_condition: serde_json::Value,
    #[serde(rename = "then")]
    pub then_result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleRequest {
    pub description: Option<String>,
    pub priority: Option<i32>,
    #[serde(rename = "when")]
    pub when_condition: Option<serde_json::Value>,
    #[serde(rename = "then")]
    pub then_result: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleSetRequestBody {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub evaluation_mode: String,
    pub default_result: serde_json::Value,
    pub rule_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleSetRequestBody {
    pub description: Option<String>,
    pub evaluation_mode: Option<String>,
    pub default_result: Option<serde_json::Value>,
    pub rule_ids: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateRequestBody {
    pub rule_set: String,
    pub input: serde_json::Value,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: i32,
    #[serde(rename = "when")]
    pub when_condition: serde_json::Value,
    #[serde(rename = "then")]
    pub then_result: serde_json::Value,
    pub enabled: bool,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::rule::Rule> for RuleResponse {
    fn from(r: crate::domain::entity::rule::Rule) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            priority: r.priority,
            when_condition: r.when_condition,
            then_result: r.then_result,
            enabled: r.enabled,
            version: r.version,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RuleSetResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub evaluation_mode: String,
    pub default_result: serde_json::Value,
    pub rule_ids: Vec<String>,
    pub current_version: u32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::rule::RuleSet> for RuleSetResponse {
    fn from(rs: crate::domain::entity::rule::RuleSet) -> Self {
        Self {
            id: rs.id.to_string(),
            name: rs.name,
            description: rs.description,
            domain: rs.domain,
            evaluation_mode: rs.evaluation_mode.as_str().to_string(),
            default_result: rs.default_result,
            rule_ids: rs.rule_ids.iter().map(|id| id.to_string()).collect(),
            current_version: rs.current_version,
            enabled: rs.enabled,
            created_at: rs.created_at.to_rfc3339(),
            updated_at: rs.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details: vec![],
            },
        }
    }

    #[allow(dead_code)]
    pub fn with_details(code: &str, message: &str, details: Vec<ErrorDetail>) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details,
            },
        }
    }
}
