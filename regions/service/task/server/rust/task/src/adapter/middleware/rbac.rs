//! RBAC ミドルウェアの再エクスポート。
//!
//! service tier のロールベースアクセス制御を提供する。
//! - GET    -> svc_viewer / svc_operator / svc_admin: read
//! - POST/PUT -> svc_operator / svc_admin: write
//! - DELETE -> svc_admin: admin

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use k1s0_auth::Claims;
use k1s0_server_common::middleware::rbac::{check_permission, Tier};

/// require_permission は resource/action ベースのアクセス制御ミドルウェアファクトリ。
/// auth_middleware の後に使用すること。
pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    Request<Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    // クローズャでリソース名とアクション名をキャプチャしてRBACチェックを実行する
    move |req: Request<Body>, next: Next| Box::pin(rbac_check(req, next, resource, action))
}

/// service tier 向けのRBACチェックのコアロジック。
/// sys_admin / svc_admin / svc_operator / svc_viewer のロールに基づいてアクセスを制御する。
async fn rbac_check(req: Request<Body>, next: Next, resource: &str, action: &str) -> Response {
    // リクエスト拡張からClaimsを取得する。存在しない場合は401を返す
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "SVC_AUTH_MISSING_CLAIMS",
                        "message": "Authentication is required. Please provide a valid Bearer token."
                    }
                })),
            )
                .into_response();
        }
    };

    // service tier でのパーミッションチェックを実行する
    let roles = claims.realm_roles().to_vec();

    if check_permission(Tier::Service, &roles, action) {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "SVC_AUTH_PERMISSION_DENIED",
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
