/// CheckPermissionInput はパーミッション確認の入力。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CheckPermissionInput {
    pub roles: Vec<String>,
    pub permission: String,
    pub resource: String,
}

/// CheckPermissionOutput はパーミッション確認の出力。
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckPermissionOutput {
    pub allowed: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
}

/// CheckPermissionUseCase はパーミッション確認ユースケース。
pub struct CheckPermissionUseCase;

impl CheckPermissionUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, input: &CheckPermissionInput) -> CheckPermissionOutput {
        for role in &input.roles {
            match role.as_str() {
                "sys_admin" => {
                    return CheckPermissionOutput {
                        allowed: true,
                        reason: String::new(),
                    }
                }
                "sys_operator" => {
                    if input.permission == "read" || input.permission == "write" {
                        return CheckPermissionOutput {
                            allowed: true,
                            reason: String::new(),
                        };
                    }
                }
                "sys_auditor" => {
                    if input.permission == "read" {
                        return CheckPermissionOutput {
                            allowed: true,
                            reason: String::new(),
                        };
                    }
                }
                _ => {}
            }
        }
        CheckPermissionOutput {
            allowed: false,
            reason: format!(
                "insufficient permissions: role(s) {:?} do not grant '{}' on '{}'",
                input.roles, input.permission, input.resource
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_admin_all_allowed() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec!["sys_admin".to_string()],
            permission: "delete".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input);
        assert!(output.allowed);
        assert!(output.reason.is_empty());
    }

    #[test]
    fn test_sys_operator_read_users() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec!["sys_operator".to_string()],
            permission: "read".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input);
        assert!(output.allowed);
        assert!(output.reason.is_empty());
    }

    #[test]
    fn test_sys_operator_admin_denied() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec!["sys_operator".to_string()],
            permission: "admin".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input);
        assert!(!output.allowed);
        assert!(output.reason.contains("insufficient permissions"));
    }

    #[test]
    fn test_sys_auditor_read_audit_logs() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec!["sys_auditor".to_string()],
            permission: "read".to_string(),
            resource: "audit_logs".to_string(),
        };
        let output = uc.execute(&input);
        assert!(output.allowed);
        assert!(output.reason.is_empty());
    }

    #[test]
    fn test_empty_roles_denied() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec![],
            permission: "read".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input);
        assert!(!output.allowed);
        assert!(output.reason.contains("insufficient permissions"));
    }

    #[test]
    fn test_unknown_role_denied() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec!["unknown_role".to_string()],
            permission: "read".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input);
        assert!(!output.allowed);
        assert!(output.reason.contains("insufficient permissions"));
        assert!(output.reason.contains("unknown_role"));
    }

    #[test]
    fn test_sys_operator_read_write() {
        let uc = CheckPermissionUseCase::new();

        // sys_operator can read
        let input_read = CheckPermissionInput {
            roles: vec!["sys_operator".to_string()],
            permission: "read".to_string(),
            resource: "config".to_string(),
        };
        let output = uc.execute(&input_read);
        assert!(output.allowed);
        assert!(output.reason.is_empty());

        // sys_operator can write
        let input_write = CheckPermissionInput {
            roles: vec!["sys_operator".to_string()],
            permission: "write".to_string(),
            resource: "config".to_string(),
        };
        let output = uc.execute(&input_write);
        assert!(output.allowed);
        assert!(output.reason.is_empty());

        // sys_operator cannot delete
        let input_delete = CheckPermissionInput {
            roles: vec!["sys_operator".to_string()],
            permission: "delete".to_string(),
            resource: "config".to_string(),
        };
        let output = uc.execute(&input_delete);
        assert!(!output.allowed);
    }

    #[test]
    fn test_user_role_denied() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            roles: vec!["user".to_string()],
            permission: "read".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input);
        assert!(!output.allowed);
        assert!(output.reason.contains("insufficient permissions"));
        assert!(output.reason.contains("user"));
    }
}
