/// AuthDomainService は RBAC 判定のドメインロジックを提供する。
/// check_permission.rs の inline RBAC ロジックをドメインサービスとして整理したもの。
pub struct AuthDomainService;

impl AuthDomainService {
    /// ロール一覧・リソース・アクションからパーミッションを確認する。
    ///
    /// - sys_admin   : 全アクションを許可
    /// - sys_operator: "read" / "write" を許可
    /// - sys_auditor : "read" のみ許可
    /// - その他      : 拒否
    pub fn check_permission(roles: &[String], resource: &str, action: &str) -> bool {
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
        let _ = resource; // 将来的にリソースごとの細かい制御に使用する
        false
    }

    /// 指定されたロールが sys_admin 権限を持つかを判定する。
    pub fn is_admin(roles: &[String]) -> bool {
        roles.iter().any(|r| r == "sys_admin")
    }

    /// 指定されたロールが sys_operator 以上の権限を持つかを判定する。
    pub fn is_operator_or_above(roles: &[String]) -> bool {
        roles
            .iter()
            .any(|r| r == "sys_admin" || r == "sys_operator")
    }

    /// 指定されたロールが sys_auditor 以上の権限を持つかを判定する。
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

    // --- check_permission tests ---

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
            "config",
            "write"
        ));
    }

    #[test]
    fn test_sys_operator_delete_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&["sys_operator"]),
            "users",
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
            "audit_logs",
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
    fn test_empty_roles_denied() {
        assert!(!AuthDomainService::check_permission(
            &roles(&[]),
            "users",
            "read"
        ));
    }

    #[test]
    fn test_multiple_roles_admin_wins() {
        // sys_auditor + sys_admin → admin を持つので delete も許可
        assert!(AuthDomainService::check_permission(
            &roles(&["sys_auditor", "sys_admin"]),
            "users",
            "delete"
        ));
    }

    // --- is_admin tests ---

    #[test]
    fn test_is_admin_true() {
        assert!(AuthDomainService::is_admin(&roles(&["sys_admin"])));
    }

    #[test]
    fn test_is_admin_false_for_operator() {
        assert!(!AuthDomainService::is_admin(&roles(&["sys_operator"])));
    }

    // --- is_operator_or_above tests ---

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

    // --- is_auditor_or_above tests ---

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
