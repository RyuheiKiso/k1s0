/// Action は RBAC で使用するアクションを表す。
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Read,
    Write,
    Delete,
    Admin,
}

impl Action {
    /// 文字列から Action を生成する。
    /// "read" / "write" / "delete" / "admin" を受け付ける。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Action::Read),
            "write" => Ok(Action::Write),
            "delete" => Ok(Action::Delete),
            "admin" => Ok(Action::Admin),
            other => Err(anyhow::anyhow!("unknown action: '{}'", other)),
        }
    }

    /// Action を文字列スライスに変換する。
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Delete => "delete",
            Action::Admin => "admin",
        }
    }
}

/// Permission は RBAC パーミッション定義を表す。
/// リソース x アクション のマトリクスをロールにマッピングする。
#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: Action,
    pub allowed_roles: Vec<String>,
}

impl Permission {
    /// 新しい Permission を生成する。
    pub fn new(resource: impl Into<String>, action: Action, allowed_roles: Vec<String>) -> Self {
        Self {
            resource: resource.into(),
            action,
            allowed_roles,
        }
    }

    /// 指定されたロールがこのパーミッションを持つかを判定する。
    pub fn is_allowed_for_role(&self, role: &str) -> bool {
        self.allowed_roles.iter().any(|r| r == role)
    }

    /// 指定されたロールのいずれかがこのパーミッションを持つかを判定する。
    pub fn is_allowed_for_any_role(&self, roles: &[String]) -> bool {
        roles.iter().any(|role| self.is_allowed_for_role(role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_from_str_read() {
        let action = Action::from_str("read").unwrap();
        assert_eq!(action, Action::Read);
    }

    #[test]
    fn test_action_from_str_write() {
        let action = Action::from_str("write").unwrap();
        assert_eq!(action, Action::Write);
    }

    #[test]
    fn test_action_from_str_delete() {
        let action = Action::from_str("delete").unwrap();
        assert_eq!(action, Action::Delete);
    }

    #[test]
    fn test_action_from_str_admin() {
        let action = Action::from_str("admin").unwrap();
        assert_eq!(action, Action::Admin);
    }

    #[test]
    fn test_action_from_str_case_insensitive() {
        assert_eq!(Action::from_str("READ").unwrap(), Action::Read);
        assert_eq!(Action::from_str("Write").unwrap(), Action::Write);
        assert_eq!(Action::from_str("DELETE").unwrap(), Action::Delete);
        assert_eq!(Action::from_str("ADMIN").unwrap(), Action::Admin);
    }

    #[test]
    fn test_action_from_str_unknown() {
        let result = Action::from_str("unknown");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown action: 'unknown'"));
    }

    #[test]
    fn test_action_as_str() {
        assert_eq!(Action::Read.as_str(), "read");
        assert_eq!(Action::Write.as_str(), "write");
        assert_eq!(Action::Delete.as_str(), "delete");
        assert_eq!(Action::Admin.as_str(), "admin");
    }

    #[test]
    fn test_permission_new() {
        let perm = Permission::new(
            "users",
            Action::Read,
            vec!["sys_admin".to_string(), "sys_auditor".to_string()],
        );
        assert_eq!(perm.resource, "users");
        assert_eq!(perm.action, Action::Read);
        assert_eq!(perm.allowed_roles.len(), 2);
    }

    #[test]
    fn test_permission_is_allowed_for_role() {
        let perm = Permission::new(
            "users",
            Action::Read,
            vec!["sys_admin".to_string(), "sys_auditor".to_string()],
        );
        assert!(perm.is_allowed_for_role("sys_admin"));
        assert!(perm.is_allowed_for_role("sys_auditor"));
        assert!(!perm.is_allowed_for_role("sys_operator"));
        assert!(!perm.is_allowed_for_role("unknown"));
    }

    #[test]
    fn test_permission_is_allowed_for_any_role() {
        let perm = Permission::new(
            "config",
            Action::Write,
            vec!["sys_admin".to_string(), "sys_operator".to_string()],
        );
        let matching_roles = vec!["sys_auditor".to_string(), "sys_operator".to_string()];
        assert!(perm.is_allowed_for_any_role(&matching_roles));

        let no_matching_roles = vec!["sys_auditor".to_string(), "user".to_string()];
        assert!(!perm.is_allowed_for_any_role(&no_matching_roles));

        let empty_roles: Vec<String> = vec![];
        assert!(!perm.is_allowed_for_any_role(&empty_roles));
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Read, Action::Read);
        assert_ne!(Action::Read, Action::Write);
        assert_ne!(Action::Write, Action::Delete);
        assert_ne!(Action::Delete, Action::Admin);
    }
}
