use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use k1s0_server_common::ErrorResponse;

use crate::adapter::handler::AppState;
use crate::domain::entity::claims::Claims;

fn error_response(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (status, Json(ErrorResponse::new(code, message.into()))).into_response()
}

/// 静的 RBAC: ロール名とリソース・アクションの組み合わせでパーミッションを判定する。
/// app-registry 固有のロール定義:
/// - admin / `sys_admin`: すべてのリソースに対する全アクション
/// - publisher / `app_publisher` / `sys_operator`: "apps" リソースに対する read/write
/// - user / `sys_auditor` / `sys_viewer`: "apps" リソースに対する read のみ
fn check_permission_static(roles: &[String], resource: &str, action: &str) -> bool {
    for role in roles {
        let allowed = match role.as_str() {
            "admin" | "sys_admin" => true,
            "publisher" | "sys_operator" | "app_publisher" => {
                resource == "apps" && (action == "read" || action == "write")
            }
            "user" | "sys_auditor" | "sys_viewer" => resource == "apps" && action == "read",
            _ => false,
        };
        if allowed {
            return true;
        }
    }
    false
}

/// `make_rbac_middleware` はリソースとアクションを受け取り、RBAC チェックを行うクロージャを返す。
pub fn make_rbac_middleware(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    State<AppState>,
    Request<axum::body::Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |State(state): State<AppState>, req: Request<axum::body::Body>, next: Next| {
        Box::pin(rbac_check(state, req, next, resource, action))
    }
}

/// RBAC チェックのコアロジック。
pub async fn rbac_check(
    _state: AppState,
    req: Request<axum::body::Body>,
    next: Next,
    resource: &str,
    action: &str,
) -> Response {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "SYS_APPS_MISSING_CLAIMS",
                "Authentication is required. Please provide a valid Bearer token.",
            );
        }
    };

    let roles: Vec<String> = claims.realm_access.roles.clone();
    let allowed = check_permission_static(&roles, resource, action);

    if allowed {
        next.run(req).await
    } else {
        error_response(
            StatusCode::FORBIDDEN,
            "SYS_APPS_PERMISSION_DENIED",
            format!(
                "Insufficient permissions: action '{action}' on resource '{resource}' is not allowed for the current roles."
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_can_do_everything() {
        assert!(check_permission_static(
            &["admin".to_string()],
            "apps",
            "read"
        ));
        assert!(check_permission_static(
            &["admin".to_string()],
            "apps",
            "write"
        ));
        assert!(check_permission_static(
            &["admin".to_string()],
            "apps",
            "admin"
        ));
    }

    #[test]
    fn test_publisher_can_read_and_write() {
        assert!(check_permission_static(
            &["publisher".to_string()],
            "apps",
            "read"
        ));
        assert!(check_permission_static(
            &["publisher".to_string()],
            "apps",
            "write"
        ));
        assert!(!check_permission_static(
            &["publisher".to_string()],
            "apps",
            "delete"
        ));
    }

    #[test]
    fn test_viewer_can_only_read() {
        assert!(check_permission_static(
            &["user".to_string()],
            "apps",
            "read"
        ));
        assert!(!check_permission_static(
            &["user".to_string()],
            "apps",
            "write"
        ));
    }

    #[test]
    fn test_unknown_role_denied() {
        assert!(!check_permission_static(
            &["unknown_role".to_string()],
            "apps",
            "read"
        ));
    }
}
