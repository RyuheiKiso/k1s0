use axum::{
    Json,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use k1s0_server_common::ErrorResponse;

use crate::adapter::handler::AppState;
use crate::domain::entity::claims::Claims;

fn error_response(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (status, Json(ErrorResponse::new(code, message.into()))).into_response()
}

/// 静的 RBAC: ロール名とリソース・アクションの組み合わせでパーミッションを判定する。
/// service-catalog 固有のロール定義:
/// - admin / sys_admin: すべてのリソースに対する全アクション
/// - sys_operator / service_manager: "services", "teams" に対する read/write
/// - user / sys_auditor / sys_viewer: "services", "teams" に対する read のみ
fn check_permission_static(roles: &[String], resource: &str, action: &str) -> bool {
    for role in roles {
        let allowed = match role.as_str() {
            "admin" | "sys_admin" => true,
            "sys_operator" | "service_manager" => {
                (resource == "services" || resource == "teams")
                    && (action == "read" || action == "write")
            }
            "user" | "sys_auditor" | "sys_viewer" => {
                (resource == "services" || resource == "teams") && action == "read"
            }
            _ => false,
        };
        if allowed {
            return true;
        }
    }
    false
}

/// make_rbac_middleware はリソースとアクションを受け取り、RBAC チェックを行うクロージャを返す。
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
                "SYS_SCAT_MISSING_CLAIMS",
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
            "SYS_SCAT_PERMISSION_DENIED",
            format!(
                "Insufficient permissions: action '{}' on resource '{}' is not allowed for the current roles.",
                action, resource
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
            "services",
            "read"
        ));
        assert!(check_permission_static(
            &["admin".to_string()],
            "services",
            "write"
        ));
        assert!(check_permission_static(
            &["admin".to_string()],
            "teams",
            "read"
        ));
        assert!(check_permission_static(
            &["admin".to_string()],
            "teams",
            "write"
        ));
    }

    #[test]
    fn test_operator_can_read_and_write() {
        assert!(check_permission_static(
            &["sys_operator".to_string()],
            "services",
            "read"
        ));
        assert!(check_permission_static(
            &["sys_operator".to_string()],
            "services",
            "write"
        ));
        assert!(check_permission_static(
            &["service_manager".to_string()],
            "teams",
            "write"
        ));
    }

    #[test]
    fn test_viewer_can_only_read() {
        assert!(check_permission_static(
            &["user".to_string()],
            "services",
            "read"
        ));
        assert!(!check_permission_static(
            &["user".to_string()],
            "services",
            "write"
        ));
        assert!(check_permission_static(
            &["sys_viewer".to_string()],
            "teams",
            "read"
        ));
        assert!(!check_permission_static(
            &["sys_viewer".to_string()],
            "teams",
            "write"
        ));
    }

    #[test]
    fn test_unknown_role_denied() {
        assert!(!check_permission_static(
            &["unknown_role".to_string()],
            "services",
            "read"
        ));
    }
}
