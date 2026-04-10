use axum::{body::Body, http::Request, middleware::Next, response::Response};

use crate::ServiceError;

/// Tier はロールチェックのスコープを表現する。
#[derive(Debug, Clone, Copy)]
pub enum Tier {
    /// System tier: `sys_admin` / `sys_operator` / `sys_auditor`
    System,
    /// Business tier: `biz_admin` / `biz_operator` / `biz_auditor` + `sys_admin` fallback
    Business,
    /// Service tier: `svc_admin` / `svc_operator` / `svc_viewer` + `sys_admin` fallback
    Service,
}

/// RBAC チェックの Future 型エイリアス
type RbacFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ServiceError>> + Send>>;

/// RBAC ミドルウェアを返す。axum の `from_fn` で使用する。
pub fn require_permission(
    tier: Tier,
    _resource: &'static str,
    action: &'static str,
) -> impl Fn(Request<Body>, Next) -> RbacFuture + Clone {
    move |req, next| Box::pin(rbac_check(req, next, tier, action))
}

async fn rbac_check(
    req: Request<Body>,
    next: Next,
    tier: Tier,
    action: &str,
) -> Result<Response, ServiceError> {
    let claims = req
        .extensions()
        .get::<k1s0_auth::Claims>()
        .ok_or_else(|| ServiceError::unauthorized("AUTH", "Missing authentication claims"))?;

    let roles = claims.realm_roles();

    if !check_permission(tier, roles, action) {
        return Err(ServiceError::forbidden(
            "AUTH",
            format!("Insufficient permissions for action: {action}"),
        ));
    }

    Ok(next.run(req).await)
}

/// ロールベースの権限チェック。Tier に応じてロールプレフィックスを切り替える。
#[must_use]
pub fn check_permission(tier: Tier, roles: &[String], action: &str) -> bool {
    for role in roles {
        match tier {
            Tier::System => match role.as_str() {
                "sys_admin" => return true,
                "sys_operator" => {
                    if matches!(action, "read" | "write") {
                        return true;
                    }
                }
                "sys_auditor" => {
                    if action == "read" {
                        return true;
                    }
                }
                _ => {}
            },
            Tier::Business => match role.as_str() {
                "sys_admin" | "biz_admin" => return true,
                "biz_operator" => {
                    if matches!(action, "read" | "write") {
                        return true;
                    }
                }
                "biz_auditor" => {
                    if action == "read" {
                        return true;
                    }
                }
                _ => {}
            },
            Tier::Service => match role.as_str() {
                "sys_admin" | "svc_admin" => return true,
                "svc_operator" => {
                    if matches!(action, "read" | "write") {
                        return true;
                    }
                }
                "svc_viewer" => {
                    if action == "read" {
                        return true;
                    }
                }
                _ => {}
            },
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- System tier tests ---

    // sys_admin がシステム層のすべてのアクションを許可されることを確認する。
    #[test]
    fn test_sys_admin_all_allowed() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_permission(Tier::System, &roles, "read"));
        assert!(check_permission(Tier::System, &roles, "write"));
        assert!(check_permission(Tier::System, &roles, "admin"));
    }

    // sys_operator がシステム層で read と write のみ許可されることを確認する。
    #[test]
    fn test_sys_operator_read_write() {
        let roles = vec!["sys_operator".to_string()];
        assert!(check_permission(Tier::System, &roles, "read"));
        assert!(check_permission(Tier::System, &roles, "write"));
        assert!(!check_permission(Tier::System, &roles, "admin"));
    }

    // sys_auditor がシステム層で read のみ許可されることを確認する。
    #[test]
    fn test_sys_auditor_read_only() {
        let roles = vec!["sys_auditor".to_string()];
        assert!(check_permission(Tier::System, &roles, "read"));
        assert!(!check_permission(Tier::System, &roles, "write"));
        assert!(!check_permission(Tier::System, &roles, "admin"));
    }

    // --- Business tier tests ---

    // biz_admin がビジネス層のすべてのアクションを許可されることを確認する。
    #[test]
    fn test_biz_admin_all_allowed() {
        let roles = vec!["biz_admin".to_string()];
        assert!(check_permission(Tier::Business, &roles, "read"));
        assert!(check_permission(Tier::Business, &roles, "write"));
        assert!(check_permission(Tier::Business, &roles, "admin"));
    }

    // biz_operator がビジネス層で read と write のみ許可されることを確認する。
    #[test]
    fn test_biz_operator_read_write() {
        let roles = vec!["biz_operator".to_string()];
        assert!(check_permission(Tier::Business, &roles, "read"));
        assert!(check_permission(Tier::Business, &roles, "write"));
        assert!(!check_permission(Tier::Business, &roles, "admin"));
    }

    // biz_auditor がビジネス層で read のみ許可されることを確認する。
    #[test]
    fn test_biz_auditor_read_only() {
        let roles = vec!["biz_auditor".to_string()];
        assert!(check_permission(Tier::Business, &roles, "read"));
        assert!(!check_permission(Tier::Business, &roles, "write"));
        assert!(!check_permission(Tier::Business, &roles, "admin"));
    }

    // sys_admin がビジネス層でもフォールバックとしてすべてのアクションを許可されることを確認する。
    #[test]
    fn test_sys_admin_fallback_in_business_tier() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_permission(Tier::Business, &roles, "read"));
        assert!(check_permission(Tier::Business, &roles, "write"));
        assert!(check_permission(Tier::Business, &roles, "admin"));
    }

    // --- Service tier tests ---

    // svc_admin がサービス層のすべてのアクションを許可されることを確認する。
    #[test]
    fn test_svc_admin_all_allowed() {
        let roles = vec!["svc_admin".to_string()];
        assert!(check_permission(Tier::Service, &roles, "read"));
        assert!(check_permission(Tier::Service, &roles, "write"));
        assert!(check_permission(Tier::Service, &roles, "admin"));
    }

    // svc_operator がサービス層で read と write のみ許可されることを確認する。
    #[test]
    fn test_svc_operator_read_write() {
        let roles = vec!["svc_operator".to_string()];
        assert!(check_permission(Tier::Service, &roles, "read"));
        assert!(check_permission(Tier::Service, &roles, "write"));
        assert!(!check_permission(Tier::Service, &roles, "admin"));
    }

    // svc_viewer がサービス層で read のみ許可されることを確認する。
    #[test]
    fn test_svc_viewer_read_only() {
        let roles = vec!["svc_viewer".to_string()];
        assert!(check_permission(Tier::Service, &roles, "read"));
        assert!(!check_permission(Tier::Service, &roles, "write"));
        assert!(!check_permission(Tier::Service, &roles, "admin"));
    }

    // sys_admin がサービス層でもフォールバックとしてすべてのアクションを許可されることを確認する。
    #[test]
    fn test_sys_admin_fallback_in_service_tier() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_permission(Tier::Service, &roles, "read"));
        assert!(check_permission(Tier::Service, &roles, "write"));
        assert!(check_permission(Tier::Service, &roles, "admin"));
    }

    // 未知のロールではすべての層でアクセスが拒否されることを確認する。
    #[test]
    fn test_unknown_role() {
        let roles = vec!["user".to_string()];
        assert!(!check_permission(Tier::System, &roles, "read"));
        assert!(!check_permission(Tier::Business, &roles, "read"));
        assert!(!check_permission(Tier::Service, &roles, "read"));
    }

    // ロールが空の場合にすべての層でアクセスが拒否されることを確認する。
    #[test]
    fn test_empty_roles() {
        let roles: Vec<String> = vec![];
        assert!(!check_permission(Tier::System, &roles, "read"));
        assert!(!check_permission(Tier::Business, &roles, "read"));
        assert!(!check_permission(Tier::Service, &roles, "read"));
    }
}
