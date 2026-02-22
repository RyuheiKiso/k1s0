use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// Re-export from the crate
use k1s0_auth_server::adapter::handler::{router, AppState};
use k1s0_auth_server::domain::entity::audit_log::{AuditLog, AuditLogSearchParams};
use k1s0_auth_server::domain::entity::claims::{Claims, RealmAccess};
use k1s0_auth_server::domain::entity::user::{Pagination, Role, User, UserListResult, UserRoles};
use k1s0_auth_server::domain::repository::{AuditLogRepository, UserRepository};
use k1s0_auth_server::infrastructure::TokenVerifier;

// --- Test doubles ---

struct TestTokenVerifier {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl TokenVerifier for TestTokenVerifier {
    async fn verify_token(&self, _token: &str) -> anyhow::Result<Claims> {
        if self.should_succeed {
            Ok(Claims {
                sub: "test-user-1".to_string(),
                iss: "test-issuer".to_string(),
                aud: "test-audience".to_string(),
                exp: chrono::Utc::now().timestamp() + 3600,
                iat: chrono::Utc::now().timestamp(),
                preferred_username: "test.user".to_string(),
                email: "test@example.com".to_string(),
                realm_access: RealmAccess {
                    roles: vec!["user".to_string(), "sys_auditor".to_string()],
                },
                ..Default::default()
            })
        } else {
            anyhow::bail!("token verification failed")
        }
    }
}

struct TestUserRepository;

#[async_trait::async_trait]
impl UserRepository for TestUserRepository {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User> {
        if user_id == "existing-user" {
            Ok(User {
                id: "existing-user".to_string(),
                username: "integration.test".to_string(),
                email: "integration@example.com".to_string(),
                first_name: "Integration".to_string(),
                last_name: "Test".to_string(),
                enabled: true,
                email_verified: true,
                created_at: chrono::Utc::now(),
                attributes: std::collections::HashMap::new(),
            })
        } else {
            anyhow::bail!("user not found: {}", user_id)
        }
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        _search: Option<String>,
        _enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult> {
        Ok(UserListResult {
            users: vec![User {
                id: "existing-user".to_string(),
                username: "integration.test".to_string(),
                email: "integration@example.com".to_string(),
                first_name: "Integration".to_string(),
                last_name: "Test".to_string(),
                enabled: true,
                email_verified: true,
                created_at: chrono::Utc::now(),
                attributes: std::collections::HashMap::new(),
            }],
            pagination: Pagination {
                total_count: 1,
                page,
                page_size,
                has_next: false,
            },
        })
    }

    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles> {
        if user_id == "existing-user" {
            Ok(UserRoles {
                user_id: "existing-user".to_string(),
                realm_roles: vec![
                    Role {
                        id: "role-1".to_string(),
                        name: "user".to_string(),
                        description: "General user".to_string(),
                    },
                ],
                client_roles: std::collections::HashMap::new(),
            })
        } else {
            anyhow::bail!("user not found: {}", user_id)
        }
    }
}

struct TestAuditLogRepository {
    logs: tokio::sync::RwLock<Vec<AuditLog>>,
}

impl TestAuditLogRepository {
    fn new() -> Self {
        Self {
            logs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl AuditLogRepository for TestAuditLogRepository {
    async fn create(&self, log: &AuditLog) -> anyhow::Result<()> {
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    async fn search(
        &self,
        params: &AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<AuditLog>, i64)> {
        let logs = self.logs.read().await;
        let filtered: Vec<_> = logs
            .iter()
            .filter(|log| {
                if let Some(ref uid) = params.user_id {
                    if log.user_id != *uid {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total = filtered.len() as i64;
        Ok((filtered, total))
    }
}

fn make_test_app(token_success: bool) -> axum::Router {
    let state = AppState::new(
        Arc::new(TestTokenVerifier {
            should_succeed: token_success,
        }),
        Arc::new(TestUserRepository),
        Arc::new(TestAuditLogRepository::new()),
        "test-issuer".to_string(),
        "test-audience".to_string(),
        None,
        None,
    );
    router(state)
}

// --- Integration Tests ---

#[tokio::test]
async fn test_full_health_check_flow() {
    let app = make_test_app(true);

    // healthz
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // readyz
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // metrics
    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_token_validate_and_introspect_flow() {
    let app = make_test_app(true);

    // validate
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/token/validate")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"token":"test-valid-token"}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["valid"], true);
    assert_eq!(json["claims"]["sub"], "test-user-1");

    // introspect
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/token/introspect")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"token":"test-valid-token"}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["active"], true);
}

#[tokio::test]
async fn test_token_validate_failure_flow() {
    let app = make_test_app(false);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/token/validate")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"token":"invalid-token"}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // introspect returns active: false
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/token/introspect")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"token":"invalid-token"}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["active"], false);
}

#[tokio::test]
async fn test_user_crud_flow() {
    let app = make_test_app(true);

    // get user
    let req = Request::builder()
        .uri("/api/v1/users/existing-user")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["id"], "existing-user");
    assert_eq!(json["username"], "integration.test");

    // get non-existent user
    let req = Request::builder()
        .uri("/api/v1/users/nonexistent")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // list users
    let req = Request::builder()
        .uri("/api/v1/users")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["users"].as_array().unwrap().len() > 0);

    // get user roles
    let req = Request::builder()
        .uri("/api/v1/users/existing-user/roles")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["user_id"], "existing-user");
}

#[tokio::test]
async fn test_audit_log_record_and_search_flow() {
    let state = AppState::new(
        Arc::new(TestTokenVerifier {
            should_succeed: true,
        }),
        Arc::new(TestUserRepository),
        Arc::new(TestAuditLogRepository::new()),
        "test-issuer".to_string(),
        "test-audience".to_string(),
        None,
        None,
    );
    let app = router(state);

    // Record an audit log
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
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert!(json["id"].is_string());

    // Search audit logs
    let req = Request::builder()
        .uri("/api/v1/audit/logs?user_id=user-uuid-1234")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(json["logs"].as_array().unwrap().len(), 1);
    assert_eq!(json["pagination"]["total_count"], 1);
}

#[tokio::test]
async fn test_audit_log_validation_errors() {
    let app = make_test_app(true);

    // Missing event_type
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
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Invalid result value
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
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
