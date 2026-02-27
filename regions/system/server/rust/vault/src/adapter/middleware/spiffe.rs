use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::entity::access_policy::SpiffeAccessPolicy;

/// SPIFFE 認可ミドルウェアが使用する共有状態。
#[derive(Clone)]
pub struct SpiffeAuthState {
    pub policies: Arc<Vec<SpiffeAccessPolicy>>,
}

/// SPIFFE ID ベースの認可ミドルウェア。
///
/// auth ミドルウェアが格納した `k1s0_auth::Claims` の `sub` フィールドを SPIFFE ID として使い、
/// リクエストパスに一致するポリシーでアクセスを制御する。
///
/// ポリシーが空の場合、またはパスに一致するポリシーがない場合はアクセスを許可する（permissive mode）。
pub async fn spiffe_auth_middleware(
    State(spiffe_state): State<SpiffeAuthState>,
    request: Request,
    next: Next,
) -> Response {
    // ポリシーが空の場合は全て許可
    if spiffe_state.policies.is_empty() {
        return next.run(request).await;
    }

    let raw_path = request.uri().path();
    let path = raw_path.strip_prefix('/').unwrap_or(raw_path);

    // パスに一致するポリシーを検索
    let matching_policies: Vec<_> = spiffe_state
        .policies
        .iter()
        .filter(|p| p.matches_path(path))
        .collect();

    // 一致するポリシーがなければ許可
    if matching_policies.is_empty() {
        return next.run(request).await;
    }

    // auth ミドルウェアが格納した Claims から sub (SPIFFE ID) を取得
    let spiffe_id = request
        .extensions()
        .get::<k1s0_auth::Claims>()
        .map(|claims| claims.sub.clone());

    if let Some(ref id) = spiffe_id {
        if matching_policies.iter().any(|p| p.is_allowed(id)) {
            return next.run(request).await;
        }
    }

    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": {
                "code": "SPIFFE_ACCESS_DENIED",
                "message": "Access denied by SPIFFE policy"
            }
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use chrono::Utc;
    use k1s0_auth::claims::{Audience, RealmAccess};
    use tower::ServiceExt;

    fn make_policy(pattern: &str, spiffe_ids: &[&str]) -> SpiffeAccessPolicy {
        SpiffeAccessPolicy {
            id: uuid::Uuid::new_v4(),
            secret_path_pattern: pattern.to_string(),
            allowed_spiffe_ids: spiffe_ids.iter().map(|s| s.to_string()).collect(),
            created_at: Utc::now(),
        }
    }

    fn make_claims(sub: &str) -> k1s0_auth::Claims {
        k1s0_auth::Claims {
            sub: sub.to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: Audience(vec!["k1s0-api".to_string()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: Some(RealmAccess {
                roles: vec!["sys_admin".to_string()],
            }),
            resource_access: None,
            tier_access: None,
        }
    }

    fn build_app(policies: Vec<SpiffeAccessPolicy>) -> Router {
        let state = SpiffeAuthState {
            policies: Arc::new(policies),
        };
        Router::new()
            .route("/api/v1/secrets/:key", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state,
                spiffe_auth_middleware,
            ))
    }

    #[tokio::test]
    async fn test_empty_policies_allows_all() {
        let app = build_app(vec![]);
        let req = Request::builder()
            .uri("/api/v1/secrets/db-password")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_no_matching_policy_allows() {
        let app = build_app(vec![make_policy(
            "other/path/*",
            &["spiffe://cluster/ns/default/sa/svc"],
        )]);
        let req = Request::builder()
            .uri("/api/v1/secrets/db-password")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_matching_policy_allowed_spiffe_id() {
        let spiffe_id = "spiffe://cluster/ns/default/sa/payment-service";
        let app = build_app(vec![make_policy(
            "api/v1/secrets/*",
            &[spiffe_id],
        )]);
        let mut req = Request::builder()
            .uri("/api/v1/secrets/db-password")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(make_claims(spiffe_id));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_matching_policy_denied_spiffe_id() {
        let app = build_app(vec![make_policy(
            "api/v1/secrets/*",
            &["spiffe://cluster/ns/default/sa/payment-service"],
        )]);
        let mut req = Request::builder()
            .uri("/api/v1/secrets/db-password")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(make_claims("spiffe://cluster/ns/default/sa/unknown-service"));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SPIFFE_ACCESS_DENIED");
    }

    #[tokio::test]
    async fn test_matching_policy_no_claims_denied() {
        let app = build_app(vec![make_policy(
            "api/v1/secrets/*",
            &["spiffe://cluster/ns/default/sa/payment-service"],
        )]);
        let req = Request::builder()
            .uri("/api/v1/secrets/db-password")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
