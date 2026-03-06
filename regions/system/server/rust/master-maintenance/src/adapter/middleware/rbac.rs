use crate::adapter::handler::error::AppError;
use axum::{body::Body, http::Request, middleware::Next, response::Response};

pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    Request<Body>,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>>
       + Clone {
    move |req, next| Box::pin(rbac_check(req, next, resource, action))
}

async fn rbac_check(
    req: Request<Body>,
    next: Next,
    _resource: &str,
    action: &str,
) -> Result<Response, AppError> {
    let claims = req.extensions().get::<k1s0_auth::Claims>().ok_or_else(|| {
        AppError::unauthorized("SYS_MM_MISSING_CLAIMS", "Missing authentication claims")
    })?;

    let roles = claims.realm_roles();

    if !check_system_permission(roles, action) {
        return Err(AppError::forbidden(
            "SYS_AUTH_PERMISSION_DENIED",
            &format!("Insufficient permissions for action: {}", action),
        ));
    }

    Ok(next.run(req).await)
}

pub fn check_system_permission(roles: &[String], action: &str) -> bool {
    for role in roles {
        match role.as_str() {
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
        }
    }
    false
}

pub fn check_domain_permission(roles: &[String], domain: &str, action: &str) -> bool {
    for role in roles {
        // sys_admin は全ドメインアクセス可
        if role == "sys_admin" {
            return true;
        }
        // ドメイン固有ロール: {domain}_admin, {domain}_operator, {domain}_auditor
        match role.as_str() {
            r if r == format!("{}_admin", domain) => return true,
            r if r == format!("{}_operator", domain) => {
                if matches!(action, "read" | "write") {
                    return true;
                }
            }
            r if r == format!("{}_auditor", domain) => {
                if action == "read" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_admin_all_allowed() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_system_permission(&roles, "read"));
        assert!(check_system_permission(&roles, "write"));
        assert!(check_system_permission(&roles, "admin"));
    }

    #[test]
    fn test_sys_operator_read_write() {
        let roles = vec!["sys_operator".to_string()];
        assert!(check_system_permission(&roles, "read"));
        assert!(check_system_permission(&roles, "write"));
        assert!(!check_system_permission(&roles, "admin"));
    }

    #[test]
    fn test_sys_auditor_read_only() {
        let roles = vec!["sys_auditor".to_string()];
        assert!(check_system_permission(&roles, "read"));
        assert!(!check_system_permission(&roles, "write"));
        assert!(!check_system_permission(&roles, "admin"));
    }

    #[test]
    fn test_unknown_role() {
        let roles = vec!["user".to_string()];
        assert!(!check_system_permission(&roles, "read"));
    }

    #[test]
    fn test_domain_admin_all_allowed() {
        let roles = vec!["accounting_admin".to_string()];
        assert!(check_domain_permission(&roles, "accounting", "read"));
        assert!(check_domain_permission(&roles, "accounting", "write"));
        assert!(check_domain_permission(&roles, "accounting", "admin"));
        assert!(!check_domain_permission(&roles, "fa", "read"));
    }

    #[test]
    fn test_domain_operator_read_write() {
        let roles = vec!["fa_operator".to_string()];
        assert!(check_domain_permission(&roles, "fa", "read"));
        assert!(check_domain_permission(&roles, "fa", "write"));
        assert!(!check_domain_permission(&roles, "fa", "admin"));
        assert!(!check_domain_permission(&roles, "accounting", "read"));
    }

    #[test]
    fn test_domain_auditor_read_only() {
        let roles = vec!["accounting_auditor".to_string()];
        assert!(check_domain_permission(&roles, "accounting", "read"));
        assert!(!check_domain_permission(&roles, "accounting", "write"));
        assert!(!check_domain_permission(&roles, "accounting", "admin"));
    }

    #[test]
    fn test_sys_admin_accesses_all_domains() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_domain_permission(&roles, "accounting", "read"));
        assert!(check_domain_permission(&roles, "fa", "admin"));
        assert!(check_domain_permission(&roles, "any_domain", "write"));
    }
}
