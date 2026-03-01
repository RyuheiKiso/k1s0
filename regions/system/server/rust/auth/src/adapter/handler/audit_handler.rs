use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use super::{AppState, ErrorResponse};
use crate::domain::entity::audit_log::{
    AuditLogSearchResult, CreateAuditLogRequest, CreateAuditLogResponse,
};
use crate::usecase::search_audit_logs::SearchAuditLogsQueryParams;

#[utoipa::path(
    post,
    path = "/api/v1/audit/logs",
    request_body = CreateAuditLogRequest,
    responses(
        (status = 201, description = "Audit log created", body = CreateAuditLogResponse),
        (status = 400, description = "Bad request"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn record_audit_log(
    State(state): State<AppState>,
    Json(req): Json<CreateAuditLogRequest>,
) -> impl IntoResponse {
    match state.record_audit_log_uc.execute(req).await {
        Ok(response) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(response).unwrap()),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_AUDIT_LOG_FAILED", &e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/logs",
    params(
        ("user_id" = Option<String>, Query, description = "Filter by user ID"),
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("page" = Option<i32>, Query, description = "Page number"),
        ("page_size" = Option<i32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Audit log search results", body = AuditLogSearchResult),
        (status = 400, description = "Bad request"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn search_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<SearchAuditLogsQueryParams>,
) -> impl IntoResponse {
    match state.search_audit_logs_uc.execute(&params).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_SEARCH_AUDIT_LOGS_FAILED", &e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler::router;
    use crate::adapter::handler::AppState;
    use crate::domain::entity::audit_log::AuditLog;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use crate::domain::repository::user_repository::MockUserRepository;
    use crate::infrastructure::MockTokenVerifier;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn make_valid_token_verifier() -> MockTokenVerifier {
        use crate::domain::entity::claims::{Claims, RealmAccess};
        use std::collections::HashMap;

        let mut token_verifier = MockTokenVerifier::new();
        token_verifier.expect_verify_token().returning(|_| {
            Ok(Claims {
                sub: "user-uuid-1234".to_string(),
                iss: "test-issuer".to_string(),
                aud: "test-audience".to_string(),
                exp: chrono::Utc::now().timestamp() + 3600,
                iat: chrono::Utc::now().timestamp(),
                jti: "token-uuid".to_string(),
                typ: "Bearer".to_string(),
                azp: "test-client".to_string(),
                scope: "openid".to_string(),
                preferred_username: "taro.yamada".to_string(),
                email: "taro@example.com".to_string(),
                realm_access: RealmAccess {
                    roles: vec!["sys_operator".to_string()],
                },
                resource_access: HashMap::new(),
                tier_access: vec![],
            })
        });
        token_verifier
    }

    fn make_app_state(audit_repo: MockAuditLogRepository) -> AppState {
        use crate::domain::repository::api_key_repository::MockApiKeyRepository;
        AppState::new(
            Arc::new(make_valid_token_verifier()),
            Arc::new(MockUserRepository::new()),
            Arc::new(audit_repo),
            Arc::new(MockApiKeyRepository::new()),
            "test-issuer".to_string(),
            "test-audience".to_string(),
            None,
            None,
            None,
        )
    }

    #[tokio::test]
    async fn test_record_audit_log_success() {
        let mut audit_repo = MockAuditLogRepository::new();
        audit_repo.expect_create().returning(|_| Ok(()));

        let state = make_app_state(audit_repo);
        let app = router(state);

        let body = serde_json::json!({
            "event_type": "LOGIN_SUCCESS",
            "user_id": "user-uuid-1234",
            "ip_address": "192.168.1.100",
            "user_agent": "Mozilla/5.0",
            "resource": "/api/v1/auth/token",
            "action": "POST",
            "result": "SUCCESS",
            "detail": {"client_id": "react-spa"}
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/logs")
            .header("content-type", "application/json")
            .header("Authorization", "Bearer valid-token")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["id"].is_string());
        assert!(json["created_at"].is_string());
    }

    #[tokio::test]
    async fn test_record_audit_log_validation_error() {
        let audit_repo = MockAuditLogRepository::new();
        let state = make_app_state(audit_repo);
        let app = router(state);

        let body = serde_json::json!({
            "event_type": "",
            "user_id": "user-1",
            "ip_address": "127.0.0.1",
            "resource": "/test",
            "action": "GET",
            "result": "SUCCESS"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/logs")
            .header("content-type", "application/json")
            .header("Authorization", "Bearer valid-token")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_search_audit_logs_success() {
        let mut audit_repo = MockAuditLogRepository::new();
        audit_repo.expect_search().returning(|_| {
            Ok((
                vec![AuditLog {
                    id: uuid::Uuid::new_v4(),
                    event_type: "LOGIN_SUCCESS".to_string(),
                    user_id: "user-1".to_string(),
                    ip_address: "192.168.1.100".to_string(),
                    user_agent: "Mozilla/5.0".to_string(),
                    resource: "/api/v1/auth/token".to_string(),
                    resource_id: None,
                    action: "POST".to_string(),
                    result: "SUCCESS".to_string(),
                    detail: Some(serde_json::json!({"client_id": "react-spa"})),
                    trace_id: None,
                    created_at: chrono::Utc::now(),
                }],
                1,
            ))
        });

        let state = make_app_state(audit_repo);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/audit/logs?page=1&page_size=50")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["logs"].as_array().unwrap().len(), 1);
        assert_eq!(json["pagination"]["total_count"], 1);
    }

    #[tokio::test]
    async fn test_search_audit_logs_with_filters() {
        let mut audit_repo = MockAuditLogRepository::new();
        audit_repo
            .expect_search()
            .withf(|params| {
                params.user_id.as_deref() == Some("user-1")
                    && params.event_type.as_deref() == Some("LOGIN_SUCCESS")
            })
            .returning(|_| Ok((vec![], 0)));

        let state = make_app_state(audit_repo);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/audit/logs?user_id=user-1&event_type=LOGIN_SUCCESS")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_record_audit_log_invalid_result_value() {
        let audit_repo = MockAuditLogRepository::new();
        let state = make_app_state(audit_repo);
        let app = router(state);

        let body = serde_json::json!({
            "event_type": "LOGIN_ATTEMPT",
            "user_id": "user-1",
            "ip_address": "127.0.0.1",
            "resource": "/test",
            "action": "POST",
            "result": "UNKNOWN"
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/audit/logs")
            .header("content-type", "application/json")
            .header("Authorization", "Bearer valid-token")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
