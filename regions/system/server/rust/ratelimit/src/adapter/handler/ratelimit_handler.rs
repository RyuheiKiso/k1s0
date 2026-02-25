use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::{AppState, ErrorResponse};

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
            "status": if overall_ok { "ready" } else { "not ready" },
            "checks": {
                "database": db_status
            }
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
    pub rule_id: String,
    pub subject: String,
}

/// POST /api/v1/ratelimit/check のレスポンスボディ。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CheckRateLimitResponse {
    pub allowed: bool,
    pub remaining: i64,
    pub reset_at: i64,
    pub reason: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/ratelimit/check",
    request_body = CheckRateLimitRequest,
    responses(
        (status = 200, description = "Rate limit check result", body = CheckRateLimitResponse),
        (status = 404, description = "Rule not found"),
        (status = 429, description = "Rate limit exceeded"),
    )
)]
pub async fn check_rate_limit(
    State(state): State<AppState>,
    Json(req): Json<CheckRateLimitRequest>,
) -> impl IntoResponse {
    match state.check_uc.execute(&req.rule_id, &req.subject).await {
        Ok(decision) => {
            let status = if decision.allowed {
                StatusCode::OK
            } else {
                StatusCode::TOO_MANY_REQUESTS
            };
            (
                status,
                Json(CheckRateLimitResponse {
                    allowed: decision.allowed,
                    remaining: decision.remaining,
                    reset_at: decision.reset_at,
                    reason: decision.reason,
                }),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_RATELIMIT_ERROR", &e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/ratelimit/rules のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRuleRequest {
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
}

/// ルールレスポンス。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
    pub enabled: bool,
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
    let input = crate::usecase::create_rule::CreateRuleInput {
        name: req.name,
        key: req.key,
        limit: req.limit,
        window_secs: req.window_secs,
        algorithm: req.algorithm,
    };

    match state.create_uc.execute(&input).await {
        Ok(rule) => (
            StatusCode::CREATED,
            Json(RuleResponse {
                id: rule.id.to_string(),
                name: rule.name,
                key: rule.key,
                limit: rule.limit,
                window_secs: rule.window_secs,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
            }),
        )
            .into_response(),
        Err(e) => {
            use crate::usecase::create_rule::CreateRuleError;
            let (status, code) = match &e {
                CreateRuleError::AlreadyExists(_) => (StatusCode::CONFLICT, "SYS_RATELIMIT_RULE_EXISTS"),
                CreateRuleError::InvalidAlgorithm(_) | CreateRuleError::Validation(_) => {
                    (StatusCode::BAD_REQUEST, "SYS_RATELIMIT_VALIDATION_ERROR")
                }
                CreateRuleError::Internal(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "SYS_RATELIMIT_INTERNAL_ERROR")
                }
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
pub async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_uc.execute(&id).await {
        Ok(rule) => (
            StatusCode::OK,
            Json(RuleResponse {
                id: rule.id.to_string(),
                name: rule.name,
                key: rule.key,
                limit: rule.limit,
                window_secs: rule.window_secs,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
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
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
    pub enabled: bool,
}

/// GET /api/v1/ratelimit/usage のレスポンスボディ。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UsageResponse {
    pub rule_id: String,
    pub rule_name: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
    pub enabled: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/ratelimit/rules",
    responses(
        (status = 200, description = "List of rules"),
    )
)]
pub async fn list_rules(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_uc.execute().await {
        Ok(rules) => {
            let resp: Vec<RuleResponse> = rules
                .into_iter()
                .map(|r| RuleResponse {
                    id: r.id.to_string(),
                    name: r.name,
                    key: r.key,
                    limit: r.limit,
                    window_secs: r.window_secs,
                    algorithm: r.algorithm.as_str().to_string(),
                    enabled: r.enabled,
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({ "rules": resp }))).into_response()
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
    let input = UpdateRuleInput {
        id,
        name: req.name,
        key: req.key,
        limit: req.limit,
        window_secs: req.window_secs,
        algorithm: req.algorithm,
        enabled: req.enabled,
    };

    match state.update_uc.execute(&input).await {
        Ok(rule) => (
            StatusCode::OK,
            Json(RuleResponse {
                id: rule.id.to_string(),
                name: rule.name,
                key: rule.key,
                limit: rule.limit,
                window_secs: rule.window_secs,
                algorithm: rule.algorithm.as_str().to_string(),
                enabled: rule.enabled,
            }),
        )
            .into_response(),
        Err(e) => {
            let (status, code) = match &e {
                UpdateRuleError::NotFound(_) => (StatusCode::NOT_FOUND, "SYS_RATELIMIT_RULE_NOT_FOUND"),
                UpdateRuleError::InvalidAlgorithm(_) | UpdateRuleError::Validation(_) => {
                    (StatusCode::BAD_REQUEST, "SYS_RATELIMIT_VALIDATION_ERROR")
                }
                UpdateRuleError::Internal(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "SYS_RATELIMIT_INTERNAL_ERROR")
                }
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
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    use crate::usecase::get_usage::GetUsageError;

    let rule_id = match params.get("rule_id") {
        Some(id) => id.clone(),
        None => {
            let err = ErrorResponse::new("SYS_RATELIMIT_VALIDATION_ERROR", "rule_id is required");
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    match state.get_usage_uc.execute(&rule_id).await {
        Ok(info) => (
            StatusCode::OK,
            Json(UsageResponse {
                rule_id: info.rule_id,
                rule_name: info.rule_name,
                limit: info.limit,
                window_secs: info.window_secs,
                algorithm: info.algorithm,
                enabled: info.enabled,
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
    use std::sync::Arc;
    use tower::ServiceExt;

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

        AppState::new(check_uc, create_uc, get_uc, list_uc, update_uc, delete_uc, get_usage_uc, None)
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_app_state(MockRateLimitRepository::new(), MockRateLimitStateStore::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_readyz_no_db() {
        let state = make_app_state(MockRateLimitRepository::new(), MockRateLimitStateStore::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/readyz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ready");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = make_app_state(MockRateLimitRepository::new(), MockRateLimitStateStore::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_check_rate_limit_allowed() {
        let rule = RateLimitRule::new(
            "api-global".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(99, 1700000060)));

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
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "rule_id": rule_id.to_string(),
            "subject": "user-123"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["allowed"], true);
        assert_eq!(json["remaining"], 99);
    }

    #[tokio::test]
    async fn test_check_rate_limit_denied() {
        let rule = RateLimitRule::new(
            "api-global".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| {
                Ok(RateLimitDecision::denied(
                    0,
                    1700000060,
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
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "rule_id": rule_id.to_string(),
            "subject": "user-123"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/check")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["allowed"], false);
    }

    #[tokio::test]
    async fn test_create_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_name().returning(|_| Ok(None));
        repo.expect_create()
            .returning(|rule| Ok(rule.clone()));

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
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            None,
        );
        let app = router(state);

        let body = serde_json::json!({
            "name": "api-global",
            "key": "global",
            "limit": 100,
            "window_secs": 60,
            "algorithm": "token_bucket"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/ratelimit/rules")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["name"], "api-global");
        assert_eq!(json["algorithm"], "token_bucket");
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
            Arc::new(crate::usecase::ListRulesUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::UpdateRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::DeleteRuleUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            Arc::new(crate::usecase::GetUsageUseCase::new(Arc::new(MockRateLimitRepository::new()))),
            None,
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/ratelimit/rules/550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
