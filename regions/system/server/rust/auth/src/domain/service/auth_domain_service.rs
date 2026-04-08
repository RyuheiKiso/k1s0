pub struct AuthDomainService;

impl AuthDomainService {
    /// Role x resource x action RBAC check.
    #[must_use] 
    pub fn check_permission(roles: &[String], resource: &str, action: &str) -> bool {
        let resource = resource.to_ascii_lowercase();
        let action = action.to_ascii_lowercase();

        roles
            .iter()
            .any(|role| Self::role_allows(role, &resource, &action))
    }

    fn role_allows(role: &str, resource: &str, action: &str) -> bool {
        match role {
            "sys_admin" => matches!(action, "read" | "write" | "delete" | "admin"),
            "sys_operator" => match resource {
                "users" | "auth_config" => action == "read" || action == "write",
                "audit_logs" | "api_keys" => action == "read" || action == "write",
                _ => false,
            },
            "sys_auditor" => match resource {
                "users" | "auth_config" | "audit_logs" | "api_keys" => action == "read",
                _ => false,
            },
            _ => false,
        }
    }

    #[allow(dead_code)]
    #[must_use] 
    pub fn is_admin(roles: &[String]) -> bool {
        roles.iter().any(|r| r == "sys_admin")
    }

    #[allow(dead_code)]
    #[must_use] 
    pub fn is_operator_or_above(roles: &[String]) -> bool {
        roles
            .iter()
            .any(|r| r == "sys_admin" || r == "sys_operator")
    }

    #[allow(dead_code)]
    #[must_use] 
    pub fn is_auditor_or_above(roles: &[String]) -> bool {
        roles
            .iter()
            .any(|r| r == "sys_admin" || r == "sys_operator" || r == "sys_auditor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roles(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_sys_admin_read_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_admin"]),
            "users",
            "read"
        ));
    }

    #[test]
    fn test_sys_admin_write_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_admin"]),
            "users",
            "write"
        ));
    }

    #[test]
    fn test_sys_admin_delete_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_admin"]),
            "users",
            "delete"
        ));
    }

    #[test]
    fn test_sys_admin_admin_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_admin"]),
            "users",
            "admin"
        ));
    }

    #[test]
    fn test_sys_operator_read_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "users",
            "read"
        ));
    }

    #[test]
    fn test_sys_operator_write_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "audit_logs",
            "write"
        ));
    }

    #[test]
    fn test_sys_operator_write_allowed_for_users() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "users",
            "write"
        ));
    }

    #[test]
    fn test_sys_operator_delete_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "api_keys",
            "delete"
        ));
    }

    #[test]
    fn test_sys_operator_admin_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "users",
            "admin"
        ));
    }

    #[test]
    fn test_sys_auditor_read_allowed() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_auditor"]),
            "audit_logs",
            "read"
        ));
    }

    #[test]
    fn test_sys_auditor_write_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["sys_auditor"]),
            "api_keys",
            "write"
        ));
    }

    #[test]
    fn test_sys_auditor_delete_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["sys_auditor"]),
            "audit_logs",
            "delete"
        ));
    }

    #[test]
    fn test_unknown_role_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["unknown_role"]),
            "users",
            "read"
        ));
    }

    #[test]
    fn test_unknown_resource_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "unknown",
            "read"
        ));
    }

    #[test]
    fn test_empty_roles_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&[]),
            "users",
            "read"
        ));
    }

    #[test]
    fn test_multiple_roles_admin_wins() {
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_auditor", "sys_admin"]),
            "users",
            "delete"
        ));
    }

    #[test]
    fn test_is_admin_true() {
        assert!(AuthDomainService::is_admin(&roles(&["sys_admin"])));
    }

    #[test]
    fn test_is_admin_false_for_operator() {
        assert!(!AuthDomainService::is_admin(&roles(&["sys_operator"])));
    }

    #[test]
    fn test_is_operator_or_above_admin() {
        assert!(AuthDomainService::is_operator_or_above(&roles(&[
            "sys_admin"
        ])));
    }

    #[test]
    fn test_is_operator_or_above_operator() {
        assert!(AuthDomainService::is_operator_or_above(&roles(&[
            "sys_operator"
        ])));
    }

    #[test]
    fn test_is_operator_or_above_auditor_false() {
        assert!(!AuthDomainService::is_operator_or_above(&roles(&[
            "sys_auditor"
        ])));
    }

    #[test]
    fn test_is_auditor_or_above_admin() {
        assert!(AuthDomainService::is_auditor_or_above(&roles(&[
            "sys_admin"
        ])));
    }

    #[test]
    fn test_is_auditor_or_above_operator() {
        assert!(AuthDomainService::is_auditor_or_above(&roles(&[
            "sys_operator"
        ])));
    }

    #[test]
    fn test_is_auditor_or_above_auditor() {
        assert!(AuthDomainService::is_auditor_or_above(&roles(&[
            "sys_auditor"
        ])));
    }

    #[test]
    fn test_is_auditor_or_above_unknown_false() {
        assert!(!AuthDomainService::is_auditor_or_above(&roles(&["user"])));
    }
}
