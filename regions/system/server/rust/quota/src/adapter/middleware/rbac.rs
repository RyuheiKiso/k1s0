use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use k1s0_auth::Claims;

/// require_permission は resource/action ベースのアクセス制御ミドルウェアファクトリ。
/// auth_middleware の後に使用すること。
///
/// quota-server の RBAC マッピング:
/// - GET/check/usage -> quotas/read
/// - POST/PUT/increment -> quotas/write
/// - DELETE/reset -> quotas/admin
pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    Request<Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |req: Request<Body>, next: Next| Box::pin(rbac_check(req, next, resource, action))
}

/// system tier 向けのパーミッションチェック。
/// auth-server の AuthDomainService::check_permission と同じロジック:
/// - sys_admin   : 全アクション許可
/// - sys_operator: "read" / "write" を許可
/// - sys_auditor : "read" のみ許可
fn check_system_permission(roles: &[String], _resource: &str, action: &str) -> bool {
    for role in roles {
        match role.as_str() {
            "sys_admin" => return true,
            "sys_operator" => {
                if action == "read" || action == "write" {
                    return true;
                }
            }
            "sys_auditor" => {
                if action == "read" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// RBAC チェックのコアロジック。
async fn rbac_check(req: Request<Body>, next: Next, resource: &str, action: &str) -> Response {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "SYS_AUTH_MISSING_CLAIMS",
                        "message": "Authentication is required. Please provide a valid Bearer token."
                    }
                })),
            )
                .into_response();
        }
    };

    let roles = claims.realm_roles().to_vec();

    if check_system_permission(&roles, resource, action) {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_AUTH_PERMISSION_DENIED",
                    "message": format!(
                        "Insufficient permissions: action '{}' on resource '{}' is not allowed for the current roles.",
                        action, resource
                    )
                }
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::{delete, get, put};
    use axum::Router;
    use k1s0_auth::claims::{Audience, RealmAccess};
    use tower::ServiceExt;

    fn make_claims_with_roles(role_names: &[&str]) -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: Audience(vec!["k1s0-api".to_string()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("taro.yamada".to_string()),
            email: Some("taro@example.com".to_string()),
            realm_access: Some(RealmAccess {
                roles: role_names.iter().map(|s| s.to_string()).collect(),
            }),
            resource_access: None,
            tier_access: None,
        }
    }

    fn make_request_with_claims(method: &str, uri: &str, claims: Claims) -> Request<Body> {
        let mut req = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(claims);
        req
    }

    #[tokio::test]
    async fn test_rbac_missing_claims_returns_401() {
        let app = Router::new().route(
            "/api/v1/quotas",
            get(|| async { "ok" }).route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "read",
            ))),
        );

        let req = Request::builder()
            .uri("/api/v1/quotas")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_MISSING_CLAIMS");
    }

    #[tokio::test]
    async fn test_rbac_sys_admin_read_allowed() {
        let app = Router::new().route(
            "/api/v1/quotas",
            get(|| async { "ok" }).route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "read",
            ))),
        );

        let claims = make_claims_with_roles(&["sys_admin"]);
        let req = make_request_with_claims("GET", "/api/v1/quotas", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rbac_sys_operator_write_allowed() {
        let app = Router::new().route(
            "/api/v1/quotas",
            put(|| async { "ok" }).route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "write",
            ))),
        );

        let claims = make_claims_with_roles(&["sys_operator"]);
        let req = make_request_with_claims("PUT", "/api/v1/quotas", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rbac_sys_auditor_read_allowed() {
        let app = Router::new().route(
            "/api/v1/quotas",
            get(|| async { "ok" }).route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "read",
            ))),
        );

        let claims = make_claims_with_roles(&["sys_auditor"]);
        let req = make_request_with_claims("GET", "/api/v1/quotas", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rbac_sys_auditor_write_denied() {
        let app = Router::new().route(
            "/api/v1/quotas",
            put(|| async { "ok" }).route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "write",
            ))),
        );

        let claims = make_claims_with_roles(&["sys_auditor"]);
        let req = make_request_with_claims("PUT", "/api/v1/quotas", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_PERMISSION_DENIED");
    }

    #[tokio::test]
    async fn test_rbac_sys_operator_admin_denied() {
        let app = Router::new().route(
            "/api/v1/quotas/:id",
            delete(|| async { StatusCode::NO_CONTENT }).route_layer(axum::middleware::from_fn(
                require_permission("quotas", "admin"),
            )),
        );

        let claims = make_claims_with_roles(&["sys_operator"]);
        let req = make_request_with_claims("DELETE", "/api/v1/quotas/123", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_rbac_sys_admin_delete_allowed() {
        let app = Router::new().route(
            "/api/v1/quotas/:id",
            delete(|| async { StatusCode::NO_CONTENT }).route_layer(axum::middleware::from_fn(
                require_permission("quotas", "admin"),
            )),
        );

        let claims = make_claims_with_roles(&["sys_admin"]);
        let req = make_request_with_claims("DELETE", "/api/v1/quotas/123", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_rbac_unknown_role_denied() {
        let app = Router::new().route(
            "/api/v1/quotas",
            get(|| async { "ok" }).route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "read",
            ))),
        );

        let claims = make_claims_with_roles(&["viewer"]);
        let req = make_request_with_claims("GET", "/api/v1/quotas", claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
