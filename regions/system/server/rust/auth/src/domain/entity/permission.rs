/// Action は RBAC で使用するアクションを表す。
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Action::Read),
            "write" => Ok(Action::Write),
            "delete" => Ok(Action::Delete),
            "admin" => Ok(Action::Admin),
            other => Err(anyhow::anyhow!("unknown action: '{other}'")),
        }
    }

    /// Action を文字列スライスに変換する。
    #[allow(dead_code)]
    #[must_use] 
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
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: Action,
    pub allowed_roles: Vec<String>,
}

impl Permission {
    /// 新しい Permission を生成する。
    #[allow(dead_code)]
    pub fn new(resource: impl Into<String>, action: Action, allowed_roles: Vec<String>) -> Self {
        Self {
            resource: resource.into(),
            action,
            allowed_roles,
        }
    }

    /// 指定されたロールがこのパーミッションを持つかを判定する。
    #[allow(dead_code)]
    #[must_use] 
    pub fn is_allowed_for_role(&self, role: &str) -> bool {
        self.allowed_roles.iter().any(|r| r == role)
    }

    /// 指定されたロールのいずれかがこのパーミッションを持つかを判定する。
    #[allow(dead_code)]
    #[must_use] 
    pub fn is_allowed_for_any_role(&self, roles: &[String]) -> bool {
        roles.iter().any(|role| self.is_allowed_for_role(role))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // テーブル駆動テスト: Action::from_str の正常系（大文字小文字含む）
    #[test]
    fn test_action_from_str_valid_cases() {
        let cases = vec![
            ("read", Action::Read),
            ("write", Action::Write),
            ("delete", Action::Delete),
            ("admin", Action::Admin),
            ("READ", Action::Read),
            ("Write", Action::Write),
            ("DELETE", Action::Delete),
            ("ADMIN", Action::Admin),
        ];
        for (input, expected) in cases {
            let result = Action::from_str(input).unwrap();
            assert_eq!(
                result, expected,
                "Action::from_str(\"{input}\") の結果が不正"
            );
        }
    }

    // テーブル駆動テスト: Action::from_str の異常系
    #[test]
    fn test_action_from_str_invalid_cases() {
        let invalid_inputs = vec!["unknown", "", "readwrite", "SUDO"];
        for input in invalid_inputs {
            let result = Action::from_str(input);
            assert!(
                result.is_err(),
                "Action::from_str(\"{input}\") はエラーであるべき"
            );
        }
    }

    // テーブル駆動テスト: Action::as_str の往復変換
    #[test]
    fn test_action_as_str_roundtrip() {
        let cases = vec![
            (Action::Read, "read"),
            (Action::Write, "write"),
            (Action::Delete, "delete"),
            (Action::Admin, "admin"),
        ];
        for (action, expected_str) in cases {
            assert_eq!(action.as_str(), expected_str);
            // 往復変換: as_str -> from_str で元に戻ることを検証
            let roundtrip = Action::from_str(action.as_str()).unwrap();
            assert_eq!(roundtrip, action);
        }
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

    // テーブル駆動テスト: is_allowed_for_role
    #[test]
    fn test_permission_is_allowed_for_role() {
        let perm = Permission::new(
            "users",
            Action::Read,
            vec!["sys_admin".to_string(), "sys_auditor".to_string()],
        );
        let cases = vec![
            ("sys_admin", true),
            ("sys_auditor", true),
            ("sys_operator", false),
            ("unknown", false),
        ];
        for (role, expected) in cases {
            assert_eq!(
                perm.is_allowed_for_role(role),
                expected,
                "is_allowed_for_role(\"{role}\") の結果が不正"
            );
        }
    }

    // テーブル駆動テスト: is_allowed_for_any_role
    #[test]
    fn test_permission_is_allowed_for_any_role() {
        let perm = Permission::new(
            "config",
            Action::Write,
            vec!["sys_admin".to_string(), "sys_operator".to_string()],
        );
        let cases: Vec<(Vec<String>, bool)> = vec![
            (vec!["sys_auditor".into(), "sys_operator".into()], true),
            (vec!["sys_auditor".into(), "user".into()], false),
            (vec![], false),
            (vec!["sys_admin".into()], true),
        ];
        for (roles, expected) in cases {
            assert_eq!(
                perm.is_allowed_for_any_role(&roles),
                expected,
                "is_allowed_for_any_role({roles:?}) の結果が不正"
            );
        }
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Read, Action::Read);
        assert_ne!(Action::Read, Action::Write);
        assert_ne!(Action::Write, Action::Delete);
        assert_ne!(Action::Delete, Action::Admin);
    }
}
