use axum::{
    extract::{Extension, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AppState, ErrorDetail, ErrorResponse};

/// JWT クレームからテナントIDを文字列として抽出するヘルパー。
/// クレームが存在しない場合、または tenant_id が無効な UUID の場合は 401 を返す。
fn extract_tenant_id_str(
    claims: &Option<Extension<k1s0_auth::Claims>>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let tenant_id_str = claims
        .as_ref()
        .map(|ext| ext.0.tenant_id.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Authentication required",
                    "code": 401
                })),
            )
        })?;

    // UUID として有効かどうかを検証する
    if Uuid::parse_str(tenant_id_str).is_err() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid tenant_id in JWT claims",
                "code": 401
            })),
        ));
    }

    Ok(tenant_id_str.to_string())
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check OK"),
    )
)]
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(
    get,
    path = "/readyz",
    responses(
        (status = 200, description = "Ready"),
        (status = 503, description = "Not ready"),
    )
)]
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let mut db_status = "skipped";
    let mut overall_ok = true;

    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => db_status = "ok",
            Err(_) => {
                db_status = "error";
                overall_ok = false;
            }
        }
    }

    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(serde_json::json!({
            // ADR-0068 準拠: "healthy"/"unhealthy" + timestamp
            "status": if overall_ok { "healthy" } else { "unhealthy" },
            "checks": {
                "database": db_status
            },
            "timestamp": Utc::now().to_rfc3339()
        })),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics"),
    )
)]
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// POST /api/v1/ratelimit/check のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CheckRateLimitRequest {
    pub scope: String,          // "service" | "user" | "endpoint"
    pub identifier: String,     // "user-001" など
    pub window: Option<String>, // "60s" など（省略時は60秒）
}

/// POST /api/v1/ratelimit/check のレスポンスボディ。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CheckRateLimitResponse {
    pub allowed: bool,
    pub scope: String,
    pub identifier: String,
    pub remaining: i64,
    pub used: i64,
    pub reset_at: DateTime<Utc>,
    pub limit: i64,
    pub rule_id: String,
    pub reason: String,
}

fn parse_window_secs(window: &Option<String>) -> i64 {
    match window {
        Some(w) => {
            if let Some(stripped) = w.strip_suffix('s') {
                stripped.parse::<i64>().unwrap_or(60)
            } else {
                w.parse::<i64>().unwrap_or(60)
            }
        }
        None => 60,
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/ratelimit/check",
    request_body = CheckRateLimitRequest,
    responses(
        (status = 200, description = "Rate limit check result", body = CheckRateLimitResponse),
    )
)]
pub async fn check_rate_limit(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Json(req): Json<CheckRateLimitRequest>,
) -> impl IntoResponse {
    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id_str(&claims) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let window_secs = parse_window_secs(&req.window);

    match state
        .check_uc
        .execute(&tenant_id, &req.scope, &req.identifier, window_secs)
        .await
    {
        Ok(decision) => {
            let limit = decision.limit;

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&limit.to_string()).unwrap_or(HeaderValue::from_static("0")),
            );
            headers.insert(
                "X-RateLimit-Remaining",
                HeaderValue::from_str(&decision.remaining.to_string())
                    .unwrap_or(HeaderValue::from_static("0")),
            );
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from_str(&decision.reset_at.timestamp().to_string())
                    .unwrap_or(HeaderValue::from_static("0")),
            );

            (
                StatusCode::OK,
                headers,
                Json(CheckRateLimitResponse {
                    allowed: decision.allowed,
                    scope: decision.scope,
                    identifier: decision.identifier,
                    remaining: decision.remaining,
                    used: decision.used,
                    reset_at: decision.reset_at,
                    limit,
                    rule_id: decision.rule_id,
                    reason: decision.reason,
                }),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::with_details(
                "SYS_RATELIMIT_ERROR",
                &e.to_string(),
                vec![ErrorDetail {
                    field: "scope".to_string(),
                    message: "invalid check request".to_string(),
                }],
            );
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/ratelimit/reset のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResetRateLimitRequest {
    pub scope: String,
    pub identifier: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/ratelimit/reset",
    request_body = ResetRateLimitRequest,
    responses(
        (status = 200, description = "Rate limit reset"),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn reset_rate_limit(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Json(req): Json<ResetRateLimitRequest>,
) -> impl IntoResponse {
    use crate::usecase::reset_rate_limit::ResetRateLimitInput;

    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id_str(&claims) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let input = ResetRateLimitInput {
        tenant_id,
        scope: req.scope.clone(),
        identifier: req.identifier.clone(),
    };

    match state.reset_uc.execute(&input).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("rate limit reset for {}:{}", req.scope, req.identifier)
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_ERROR", &e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/ratelimit/rules のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRuleRequest {
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: Option<String>,
    pub enabled: bool,
}

/// ルールレスポンス。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: u32,
    pub window_seconds: u32,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn parse_positive_u32(value: i64, field: &str) -> Result<u32, String> {
    let parsed = u32::try_from(value).map_err(|_| format!("{field} must be >= 0"))?;
    if parsed == 0 {
        return Err(format!("{field} must be positive"));
    }
    Ok(parsed)
}

#[utoipa::path(
    post,
    path = "/api/v1/ratelimit/rules",
    request_body = CreateRuleRequest,
    responses(
        (status = 201, description = "Rule created", body = RuleResponse),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Rule already exists"),
    )
)]
pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> impl IntoResponse {
    let limit = match parse_positive_u32(req.limit, "limit") {
        Ok(v) => v,
        Err(msg) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_VALIDATION_ERROR", &msg);
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };
    let window_seconds = match parse_positive_u32(req.window_seconds, "window_seconds") {
        Ok(v) => v,
        Err(msg) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_VALIDATION_ERROR", &msg);
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    let input = crate::usecase::create_rule::CreateRuleInput {
        scope: req.scope,
        identifier_pattern: req.identifier_pattern,
        limit,
        window_seconds,
        algorithm: req.algorithm,
        enabled: req.enabled,
    };

    match state.create_uc.execute(&input).await {
        Ok(rule) => (
            StatusCode::CREATED,
            Json(RuleResponse {
                id: rule.id.to_string(),
                name: rule.name,
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: rule.limit,
                window_seconds: rule.window_seconds,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: rule.created_at.to_rfc3339(),
                updated_at: rule.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            use crate::usecase::create_rule::CreateRuleError;
            let (status, code) = match &e {
                CreateRuleError::AlreadyExists(_) => {
                    (StatusCode::CONFLICT, "SYS_RATELIMIT_RULE_EXISTS")
                }
                CreateRuleError::InvalidAlgorithm(_) | CreateRuleError::Validation(_) => {
                    (StatusCode::BAD_REQUEST, "SYS_RATELIMIT_VALIDATION_ERROR")
                }
                CreateRuleError::Internal(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SYS_RATELIMIT_INTERNAL_ERROR",
                ),
            };
            let err = ErrorResponse::new(code, &e.to_string());
            (status, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/ratelimit/rules/{id}",
    params(("id" = String, Path, description = "Rule ID")),
    responses(
        (status = 200, description = "Rule found", body = RuleResponse),
        (status = 404, description = "Rule not found"),
    )
)]
pub async fn get_rule(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.get_uc.execute(&id).await {
        Ok(rule) => (
            StatusCode::OK,
            Json(RuleResponse {
                id: rule.id.to_string(),
                name: rule.name,
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: rule.limit,
                window_seconds: rule.window_seconds,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: rule.created_at.to_rfc3339(),
                updated_at: rule.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(_) => {
            let err = ErrorResponse::new(
                "SYS_RATELIMIT_RULE_NOT_FOUND",
                "The specified rule was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
    }
}

/// PUT /api/v1/ratelimit/rules/:id のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRuleRequest {
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: Option<String>,
    pub enabled: bool,
}

/// GET /api/v1/ratelimit/usage のレスポンスボディ。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UsageResponse {
    pub rule_id: String,
    pub rule_name: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_at: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListRulesQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub scope: Option<String>,
    pub enabled_only: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/v1/ratelimit/rules",
    params(
        ListRulesQuery
    ),
    responses(
        (status = 200, description = "List of rules"),
    )
)]
pub async fn list_rules(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListRulesQuery>,
) -> impl IntoResponse {
    match state
        .list_uc
        .execute(&crate::usecase::list_rules::ListRulesInput {
            page: query.page.unwrap_or(1).max(1),
            page_size: query.page_size.unwrap_or(20).max(1),
            scope: query.scope.clone(),
            enabled_only: query.enabled_only.unwrap_or(false),
        })
        .await
    {
        Ok(output) => {
            let resp: Vec<RuleResponse> = output
                .rules
                .into_iter()
                .map(|r| RuleResponse {
                    id: r.id.to_string(),
                    name: r.name,
                    scope: r.scope,
                    identifier_pattern: r.identifier_pattern,
                    limit: r.limit,
                    window_seconds: r.window_seconds,
                    algorithm: r.algorithm.as_str().to_string(),
                    enabled: r.enabled,
                    created_at: r.created_at.to_rfc3339(),
                    updated_at: r.updated_at.to_rfc3339(),
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "rules": resp,
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
            let err = ErrorResponse::new("SYS_RATELIMIT_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/ratelimit/rules/{id}",
    params(("id" = String, Path, description = "Rule ID")),
    request_body = UpdateRuleRequest,
    responses(
        (status = 200, description = "Rule updated", body = RuleResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Rule not found"),
    )
)]
pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateRuleRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_rule::{UpdateRuleError, UpdateRuleInput};
    let limit = match parse_positive_u32(req.limit, "limit") {
        Ok(v) => v,
        Err(msg) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_VALIDATION_ERROR", &msg);
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };
    let window_seconds = match parse_positive_u32(req.window_seconds, "window_seconds") {
        Ok(v) => v,
        Err(msg) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_VALIDATION_ERROR", &msg);
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };
    let input = UpdateRuleInput {
        id,
        scope: req.scope,
        identifier_pattern: req.identifier_pattern,
        limit,
        window_seconds,
        algorithm: req.algorithm,
        enabled: req.enabled,
    };

    match state.update_uc.execute(&input).await {
        Ok(rule) => (
            StatusCode::OK,
            Json(RuleResponse {
                id: rule.id.to_string(),
                name: rule.name,
                scope: rule.scope,
                identifier_pattern: rule.identifier_pattern,
                limit: rule.limit,
                window_seconds: rule.window_seconds,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
                created_at: rule.created_at.to_rfc3339(),
                updated_at: rule.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            let (status, code) = match &e {
                UpdateRuleError::NotFound(_) => {
                    (StatusCode::NOT_FOUND, "SYS_RATELIMIT_RULE_NOT_FOUND")
                }
                UpdateRuleError::InvalidAlgorithm(_) | UpdateRuleError::Validation(_) => {
                    (StatusCode::BAD_REQUEST, "SYS_RATELIMIT_VALIDATION_ERROR")
                }
                UpdateRuleError::Internal(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SYS_RATELIMIT_INTERNAL_ERROR",
                ),
            };
            let err = ErrorResponse::new(code, &e.to_string());
            (status, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/ratelimit/rules/{id}",
    params(("id" = String, Path, description = "Rule ID")),
    responses(
        (status = 204, description = "Rule deleted"),
        (status = 404, description = "Rule not found"),
    )
)]
pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::delete_rule::DeleteRuleError;

    match state.delete_uc.execute(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteRuleError::NotFound(_)) | Err(DeleteRuleError::InvalidRuleId(_)) => {
            let err = ErrorResponse::new(
                "SYS_RATELIMIT_RULE_NOT_FOUND",
                &format!("rule not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(DeleteRuleError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/ratelimit/usage",
    params(("rule_id" = String, Query, description = "Rule ID")),
    responses(
        (status = 200, description = "Usage info", body = UsageResponse),
        (status = 404, description = "Rule not found"),
    )
)]
pub async fn get_usage(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    use crate::usecase::get_usage::GetUsageError;

    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id_str(&claims) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let rule_id = match params.get("rule_id") {
        Some(id) => id.clone(),
        None => {
            let err = ErrorResponse::new("SYS_RATELIMIT_VALIDATION_ERROR", "rule_id is required");
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    match state.get_usage_uc.execute(&tenant_id, &rule_id).await {
        Ok(info) => (
            StatusCode::OK,
            Json(UsageResponse {
                rule_id: info.rule_id,
                rule_name: info.rule_name,
                limit: info.limit,
                window_seconds: info.window_seconds,
                algorithm: info.algorithm,
                enabled: info.enabled,
                used: info.used,
                remaining: info.remaining,
                reset_at: info.reset_at,
            }),
        )
            .into_response(),
        Err(GetUsageError::NotFound(_)) | Err(GetUsageError::InvalidRuleId(_)) => {
            let err = ErrorResponse::new(
                "SYS_RATELIMIT_RULE_NOT_FOUND",
                &format!("rule not found: {}", rule_id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(GetUsageError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler::router;
    use crate::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};
    use crate::domain::repository::rate_limit_repository::{
        MockRateLimitRepository, MockRateLimitStateStore,
    };
    use axum::body::Body;
    use axum::http::Request;
    use chrono::TimeZone;
    use std::sync::Arc;
    use tower::ServiceExt;

    /// テスト用の有効なテナントUUID
    const TEST_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

    /// テスト用の有効なJWT Claimsを作成するヘルパー。
    /// リクエストに認証情報を注入する際に使用する。
    fn make_test_claims() -> k1s0_auth::Claims {
        k1s0_auth::Claims {
            sub: "test-user".to_string(),
            iss: "https://auth.example.com".to_string(),
            aud: k1s0_auth::Audience(vec!["test".to_string()]),
            exp: 9999999999,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("test-user".to_string()),
            email: None,
            realm_access: None,
            resource_access: None,
            tier_access: None,
            tenant_id: TEST_TENANT_ID.to_string(),
        }
    }

    // テスト用タイムスタンプヘルパー（指定秒数からUTCのDateTimeを生成する）
    fn ts(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0)
            .single()
            .expect("テスト用タイムスタンプの生成に失敗")
    }

    fn make_reset_uc(
        state_store: MockRateLimitStateStore,
    ) -> Arc<crate::usecase::ResetRateLimitUseCase> {
        Arc::new(crate::usecase::ResetRateLimitUseCase::new(Arc::new(
            state_store,
        )))
    }

    fn make_app_state(
        repo: MockRateLimitRepository,
        state_store: MockRateLimitStateStore,
    ) -> AppState {
        let check_uc = Arc::new(crate::usecase::CheckRateLimitUseCase::new(
            Arc::new(repo),
            Arc::new(state_store),
        ));
        let create_uc = Arc::new(crate::usecase::CreateRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let get_uc = Arc::new(crate::usecase::GetRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let list_uc = Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let update_uc = Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let delete_uc = Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let get_usage_uc = Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let reset_uc = make_reset_uc(MockRateLimitStateStore::new());

        AppState::new(
            check_uc,
            create_uc,
            get_uc,
            list_uc,
            update_uc,
            delete_uc,
            get_usage_uc,
            reset_uc,
            None,
        )
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_app_state(
            MockRateLimitRepository::new(),
            MockRateLimitStateStore::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .expect("healthzリクエストの構築に失敗");

        let resp = app
            .oneshot(req)
            .await
            .expect("healthzリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::OK);

        // レスポンスボディを読み取り、JSONとしてパースする
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_readyz_no_db() {
        let state = make_app_state(
            MockRateLimitRepository::new(),
            MockRateLimitStateStore::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/readyz")
            .body(Body::empty())
            .expect("readyzリクエストの構築に失敗");

        let resp = app
            .oneshot(req)
            .await
            .expect("readyzリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::OK);

        // DB未設定時のreadyzレスポンスを確認する
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["status"], "ready");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = make_app_state(
            MockRateLimitRepository::new(),
            MockRateLimitStateStore::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .expect("metricsリクエストの構築に失敗");

        let resp = app
            .oneshot(req)
            .await
            .expect("metricsリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_check_rate_limit_allowed() {
        let rule = RateLimitRule::new(
            "service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 99, ts(1700000060))));

        let check_uc = Arc::new(crate::usecase::CheckRateLimitUseCase::new(
            Arc::new(repo),
            Arc::new(state_store),
        ));
        let create_uc = Arc::new(crate::usecase::CreateRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let get_uc = Arc::new(crate::usecase::GetRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));

        let state = AppState::new(
            check_uc,
            create_uc,
            get_uc,
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            make_reset_uc(MockRateLimitStateStore::new()),
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "scope": "service",
            "identifier": "user-123"
        });

        // 有効なJWT Claimsを注入してレート制限チェックリクエストを送信する
        let mut req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/check")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&body).expect("リクエストボディのJSON変換に失敗"),
            ))
            .expect("rate limit checkリクエストの構築に失敗");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("rate limit checkリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::OK);

        // レート制限許可レスポンスを確認する
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["allowed"], true);
        assert_eq!(json["remaining"], 99);
    }

    #[tokio::test]
    async fn test_check_rate_limit_denied() {
        let rule = RateLimitRule::new(
            "service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| {
                Ok(RateLimitDecision::denied(
                    100,
                    0,
                    ts(1700000060),
                    "rate limit exceeded".to_string(),
                ))
            });

        let check_uc = Arc::new(crate::usecase::CheckRateLimitUseCase::new(
            Arc::new(repo),
            Arc::new(state_store),
        ));
        let create_uc = Arc::new(crate::usecase::CreateRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let get_uc = Arc::new(crate::usecase::GetRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));

        let state = AppState::new(
            check_uc,
            create_uc,
            get_uc,
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            make_reset_uc(MockRateLimitStateStore::new()),
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "scope": "service",
            "identifier": "user-123"
        });

        // 有効なJWT Claimsを注入してレート制限拒否ケースのリクエストを送信する
        let mut req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/check")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&body).expect("リクエストボディのJSON変換に失敗"),
            ))
            .expect("rate limit check deniedリクエストの構築に失敗");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("rate limit check deniedリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::OK);

        // レート制限拒否レスポンスを確認する
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["allowed"], false);
    }

    #[tokio::test]
    async fn test_create_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_scope().returning(|_| Ok(vec![]));
        repo.expect_create().returning(|rule| Ok(rule.clone()));

        let check_uc = Arc::new(crate::usecase::CheckRateLimitUseCase::new(
            Arc::new(MockRateLimitRepository::new()),
            Arc::new(MockRateLimitStateStore::new()),
        ));
        let create_uc = Arc::new(crate::usecase::CreateRuleUseCase::new(Arc::new(repo)));
        let get_uc = Arc::new(crate::usecase::GetRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));

        let state = AppState::new(
            check_uc,
            create_uc,
            get_uc,
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            make_reset_uc(MockRateLimitStateStore::new()),
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "scope": "service",
            "identifier_pattern": "global",
            "limit": 100,
            "window_seconds": 60,
            "enabled": true
        });

        // ルール作成リクエストを送信する
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/rules")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&body).expect("リクエストボディのJSON変換に失敗"),
            ))
            .expect("create ruleリクエストの構築に失敗");

        let resp = app
            .oneshot(req)
            .await
            .expect("create ruleリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::CREATED);

        // 作成されたルールのレスポンスを確認する
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["scope"], "service");
        assert_eq!(json["identifier_pattern"], "global");
    }

    #[tokio::test]
    async fn test_get_rule_not_found() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not found")));

        let check_uc = Arc::new(crate::usecase::CheckRateLimitUseCase::new(
            Arc::new(MockRateLimitRepository::new()),
            Arc::new(MockRateLimitStateStore::new()),
        ));
        let create_uc = Arc::new(crate::usecase::CreateRuleUseCase::new(Arc::new(
            MockRateLimitRepository::new(),
        )));
        let get_uc = Arc::new(crate::usecase::GetRuleUseCase::new(Arc::new(repo)));

        let state = AppState::new(
            check_uc,
            create_uc,
            get_uc,
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            make_reset_uc(MockRateLimitStateStore::new()),
            None,
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/ratelimit/rules/550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .expect("get rule not_foundリクエストの構築に失敗");

        let resp = app
            .oneshot(req)
            .await
            .expect("get rule not_foundリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_reset_rate_limit_success() {
        let mut state_store = MockRateLimitStateStore::new();
        state_store.expect_reset().returning(|_| Ok(()));

        let check_uc = Arc::new(crate::usecase::CheckRateLimitUseCase::new(
            Arc::new(MockRateLimitRepository::new()),
            Arc::new(MockRateLimitStateStore::new()),
        ));
        let reset_uc = make_reset_uc(state_store);

        let state = AppState::new(
            check_uc,
            Arc::new(crate::usecase::CreateRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::GetRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(
                MockRateLimitRepository::new(),
            ))),
            reset_uc,
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "scope": "service",
            "identifier": "user-123"
        });

        // 有効なJWT Claimsを注入してレート制限リセットリクエストを送信する
        let mut req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/reset")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&body).expect("リクエストボディのJSON変換に失敗"),
            ))
            .expect("rate limit resetリクエストの構築に失敗");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("rate limit resetリクエストの送信に失敗");
        assert_eq!(resp.status(), StatusCode::OK);

        // リセット成功レスポンスを確認する
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["success"], true);
    }

    /// JWT クレームなし（認証情報未設定）の場合に 401 を返すことを確認するテスト。
    /// これにより SYSTEM テナントへのフォールバックが廃止されたことを検証する。
    #[tokio::test]
    async fn test_check_rate_limit_unauthorized_no_claims() {
        let state = make_app_state(
            MockRateLimitRepository::new(),
            MockRateLimitStateStore::new(),
        );
        let app = router(state);

        let body = serde_json::json!({
            "scope": "service",
            "identifier": "user-123"
        });

        // JWT Claimsを注入せずにリクエストを送信する
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/check")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&body).expect("リクエストボディのJSON変換に失敗"),
            ))
            .expect("rate limit check unauthorizedリクエストの構築に失敗");

        let resp = app
            .oneshot(req)
            .await
            .expect("rate limit check unauthorizedリクエストの送信に失敗");
        // クレームなしの場合は 401 が返されることを確認する
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("レスポンスボディの読み取りに失敗");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("レスポンスのJSONパースに失敗");
        assert_eq!(json["error"], "Authentication required");
        assert_eq!(json["code"], 401);
    }
}
