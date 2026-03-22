//! RBAC ミドルウェア。
//!
//! project-master サービス固有のロールベースアクセス制御を提供する。
//! business tier の標準ロールに加えて taskmanagement 固有ロールをサポートする。
//!
//! ロールマッピング:
//! - `biz_taskmanagement_admin` / `biz_admin` / `sys_admin`: 全操作許可（read/write/admin）
//! - `biz_taskmanagement_manager` / `biz_operator`: read と write を許可
//! - `biz_taskmanagement_viewer` / `biz_auditor`: read のみ許可

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

/// project-master 固有のロールマッピングでパーミッションを確認する。
/// business 標準ロールと taskmanagement 固有ロールの両方をサポートする。
fn check_taskmanagement_permission(roles: &[String], _resource: &str, action: &str) -> bool {
    for role in roles {
        match role.as_str() {
            // sys_admin は全ロールに対してスーパーセット権限を持つ
            "sys_admin" => return true,
            // biz_admin・biz_taskmanagement_admin は全操作を許可する
            "biz_admin" | "biz_taskmanagement_admin" => return true,
            // biz_operator・biz_taskmanagement_manager は read と write を許可する
            "biz_operator" | "biz_taskmanagement_manager" => {
                if matches!(action, "read" | "write") {
                    return true;
                }
            }
            // biz_auditor・biz_taskmanagement_viewer は read のみ許可する
            "biz_auditor" | "biz_taskmanagement_viewer" => {
                if action == "read" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// project-master RBAC チェックのコアロジック。
async fn rbac_check(req: Request<Body>, next: Next, resource: &str, action: &str) -> Response {
    // リクエスト拡張からClaimsを取得する。存在しない場合は401を返す
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "BIZ_AUTH_MISSING_CLAIMS",
                        "message": "Authentication is required. Please provide a valid Bearer token."
                    }
                })),
            )
                .into_response();
        }
    };

    // taskmanagement 固有のパーミッションチェックを実行する
    let roles = claims.realm_roles().to_vec();

    if check_taskmanagement_permission(&roles, resource, action) {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "BIZ_AUTH_PERMISSION_DENIED",
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // sys_admin が全操作を許可されることを確認する
    #[test]
    fn test_sys_admin_all_allowed() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // biz_taskmanagement_admin が全操作を許可されることを確認する
    #[test]
    fn test_biz_taskmanagement_admin_all_allowed() {
        let roles = vec!["biz_taskmanagement_admin".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // biz_admin が全操作を許可されることを確認する
    #[test]
    fn test_biz_admin_all_allowed() {
        let roles = vec!["biz_admin".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // biz_taskmanagement_manager が read と write のみ許可されることを確認する
    #[test]
    fn test_biz_taskmanagement_manager_read_write() {
        let roles = vec!["biz_taskmanagement_manager".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // biz_operator が read と write のみ許可されることを確認する
    #[test]
    fn test_biz_operator_read_write() {
        let roles = vec!["biz_operator".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // biz_taskmanagement_viewer が read のみ許可されることを確認する
    #[test]
    fn test_biz_taskmanagement_viewer_read_only() {
        let roles = vec!["biz_taskmanagement_viewer".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // biz_auditor が read のみ許可されることを確認する
    #[test]
    fn test_biz_auditor_read_only() {
        let roles = vec!["biz_auditor".to_string()];
        assert!(check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // 未知のロールはアクセスが拒否されることを確認する
    #[test]
    fn test_unknown_role_denied() {
        let roles = vec!["anonymous".to_string()];
        assert!(!check_taskmanagement_permission(&roles, "project-types", "read"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "write"));
        assert!(!check_taskmanagement_permission(&roles, "project-types", "admin"));
    }

    // ロールが空の場合にアクセスが拒否されることを確認する
    #[test]
    fn test_empty_roles_denied() {
        let roles: Vec<String> = vec![];
        assert!(!check_taskmanagement_permission(&roles, "project-types", "read"));
    }
}
