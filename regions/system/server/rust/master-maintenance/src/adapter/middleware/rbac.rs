use crate::adapter::handler::error::AppError;
use crate::domain::entity::table_definition::TableDefinition;
use axum::{body::Body, http::Request, middleware::Next, response::Response};

#[allow(clippy::type_complexity)]
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
            &format!("Insufficient permissions for action: {action}"),
        ));
    }

    Ok(next.run(req).await)
}

#[must_use] 
pub fn has_action_permission(
    roles: &[String],
    action: &str,
    table: Option<&TableDefinition>,
) -> bool {
    if check_system_permission(roles, action) {
        return true;
    }

    let Some(table) = table else {
        return false;
    };

    if check_table_permission(roles, table, action) {
        return true;
    }

    table
        .domain_scope
        .as_deref()
        .is_some_and(|domain| check_domain_permission(roles, domain, action))
}

#[must_use] 
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

#[must_use] 
pub fn check_table_permission(roles: &[String], table: &TableDefinition, action: &str) -> bool {
    let required_roles = match action {
        "read" => &table.read_roles,
        "write" => &table.write_roles,
        "admin" => &table.admin_roles,
        _ => return false,
    };

    !required_roles.is_empty()
        && required_roles
            .iter()
            .any(|role| roles.iter().any(|r| r == role))
}

#[must_use] 
pub fn check_domain_permission(roles: &[String], domain: &str, action: &str) -> bool {
    for role in roles {
        // sys_admin は全ドメインアクセス可
        if role == "sys_admin" {
            return true;
        }

        if matches_domain_admin_role(role, domain) {
            return true;
        }
        if matches_domain_write_role(role, domain) && matches!(action, "read" | "write") {
            return true;
        }
        if matches_domain_read_role(role, domain) && action == "read" {
            return true;
        }
    }
    false
}

fn matches_domain_admin_role(role: &str, domain: &str) -> bool {
    role == format!("{domain}_admin") || role == format!("biz_{domain}_admin")
}

fn matches_domain_write_role(role: &str, domain: &str) -> bool {
    role == format!("{domain}_operator") || role == format!("biz_{domain}_manager")
}

fn matches_domain_read_role(role: &str, domain: &str) -> bool {
    role == format!("{domain}_auditor") || role == format!("biz_{domain}_viewer")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_table() -> TableDefinition {
        TableDefinition {
            id: Uuid::new_v4(),
            name: "departments".to_string(),
            schema_name: "business".to_string(),
            database_name: "default".to_string(),
            display_name: "Departments".to_string(),
            description: None,
            category: None,
            is_active: true,
            allow_create: true,
            allow_update: true,
            allow_delete: false,
            read_roles: vec!["table_departments_reader".to_string()],
            write_roles: vec!["table_departments_editor".to_string()],
            admin_roles: vec!["table_departments_admin".to_string()],
            sort_order: 0,
            created_by: "tester".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            domain_scope: Some("taskmanagement".to_string()),
        }
    }

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
        let roles = vec!["taskmanagement_admin".to_string()];
        assert!(check_domain_permission(&roles, "taskmanagement", "read"));
        assert!(check_domain_permission(&roles, "taskmanagement", "write"));
        assert!(check_domain_permission(&roles, "taskmanagement", "admin"));
        assert!(!check_domain_permission(&roles, "fa", "read"));
    }

    #[test]
    fn test_domain_operator_read_write() {
        let roles = vec!["fa_operator".to_string()];
        assert!(check_domain_permission(&roles, "fa", "read"));
        assert!(check_domain_permission(&roles, "fa", "write"));
        assert!(!check_domain_permission(&roles, "fa", "admin"));
        assert!(!check_domain_permission(&roles, "taskmanagement", "read"));
    }

    #[test]
    fn test_domain_auditor_read_only() {
        let roles = vec!["taskmanagement_auditor".to_string()];
        assert!(check_domain_permission(&roles, "taskmanagement", "read"));
        assert!(!check_domain_permission(&roles, "taskmanagement", "write"));
        assert!(!check_domain_permission(&roles, "taskmanagement", "admin"));
    }

    #[test]
    fn test_business_domain_roles_supported() {
        let roles = vec!["biz_taskmanagement_manager".to_string()];
        assert!(check_domain_permission(&roles, "taskmanagement", "read"));
        assert!(check_domain_permission(&roles, "taskmanagement", "write"));
        assert!(!check_domain_permission(&roles, "taskmanagement", "admin"));
    }

    #[test]
    fn test_sys_admin_accesses_all_domains() {
        let roles = vec!["sys_admin".to_string()];
        assert!(check_domain_permission(&roles, "taskmanagement", "read"));
        assert!(check_domain_permission(&roles, "fa", "admin"));
        assert!(check_domain_permission(&roles, "any_domain", "write"));
    }

    #[test]
    fn test_table_permission_grants_access() {
        let roles = vec!["table_departments_editor".to_string()];
        assert!(check_table_permission(&roles, &sample_table(), "write"));
        assert!(!check_table_permission(&roles, &sample_table(), "admin"));
    }

    #[test]
    fn test_has_action_permission_accepts_table_or_domain_roles() {
        let table = sample_table();
        let explicit_roles = vec!["table_departments_reader".to_string()];
        let domain_roles = vec!["biz_taskmanagement_viewer".to_string()];

        assert!(has_action_permission(&explicit_roles, "read", Some(&table)));
        assert!(has_action_permission(&domain_roles, "read", Some(&table)));
        assert!(!has_action_permission(
            &["plain_user".to_string()],
            "read",
            Some(&table)
        ));
    }
}
