use std::collections::HashMap;

/// Role はユーザーのロール情報を表す。
/// Keycloak の realm_roles / client_roles に対応。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl Role {
    /// 新しい Role を生成する。
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
        }
    }
}

/// UserRoles はユーザーに割り当てられたロール一覧を表す。
/// Keycloak の realm_roles / client_roles に対応。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserRoles {
    pub user_id: String,
    pub realm_roles: Vec<Role>,
    pub client_roles: HashMap<String, Vec<Role>>,
}

impl UserRoles {
    /// 新しい UserRoles を生成する。
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
        }
    }

    /// realm_roles のロール名一覧を返す。
    pub fn realm_role_names(&self) -> Vec<&str> {
        self.realm_roles.iter().map(|r| r.name.as_str()).collect()
    }

    /// 指定クライアントの client_roles のロール名一覧を返す。
    pub fn client_role_names(&self, client: &str) -> Vec<&str> {
        self.client_roles
            .get(client)
            .map(|roles| roles.iter().map(|r| r.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// 指定されたレルムロールを持っているかを判定する。
    pub fn has_realm_role(&self, role_name: &str) -> bool {
        self.realm_roles.iter().any(|r| r.name == role_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_role(name: &str) -> Role {
        Role::new(
            format!("role-id-{}", name),
            name,
            format!("{} role", name),
        )
    }

    #[test]
    fn test_role_new() {
        let role = Role::new("role-uuid-1", "sys_admin", "System administrator");
        assert_eq!(role.id, "role-uuid-1");
        assert_eq!(role.name, "sys_admin");
        assert_eq!(role.description, "System administrator");
    }

    #[test]
    fn test_role_equality() {
        let role1 = Role::new("id-1", "sys_admin", "admin");
        let role2 = Role::new("id-1", "sys_admin", "admin");
        let role3 = Role::new("id-2", "sys_operator", "operator");
        assert_eq!(role1, role2);
        assert_ne!(role1, role3);
    }

    #[test]
    fn test_role_serialization_roundtrip() {
        let role = Role::new("role-uuid-1", "sys_admin", "System administrator");
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }

    #[test]
    fn test_user_roles_new() {
        let user_roles = UserRoles::new("user-uuid-1234");
        assert_eq!(user_roles.user_id, "user-uuid-1234");
        assert!(user_roles.realm_roles.is_empty());
        assert!(user_roles.client_roles.is_empty());
    }

    #[test]
    fn test_user_roles_realm_role_names() {
        let mut user_roles = UserRoles::new("user-uuid-1234");
        user_roles.realm_roles.push(sample_role("sys_admin"));
        user_roles.realm_roles.push(sample_role("sys_auditor"));

        let names = user_roles.realm_role_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"sys_admin"));
        assert!(names.contains(&"sys_auditor"));
    }

    #[test]
    fn test_user_roles_client_role_names() {
        let mut user_roles = UserRoles::new("user-uuid-1234");
        user_roles.client_roles.insert(
            "order-service".to_string(),
            vec![sample_role("read"), sample_role("write")],
        );

        let names = user_roles.client_role_names("order-service");
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"read"));
        assert!(names.contains(&"write"));

        let empty = user_roles.client_role_names("unknown-service");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_user_roles_has_realm_role() {
        let mut user_roles = UserRoles::new("user-uuid-1234");
        user_roles.realm_roles.push(sample_role("sys_admin"));

        assert!(user_roles.has_realm_role("sys_admin"));
        assert!(!user_roles.has_realm_role("sys_operator"));
    }

    #[test]
    fn test_user_roles_serialization_roundtrip() {
        let mut user_roles = UserRoles::new("user-uuid-1234");
        user_roles.realm_roles.push(sample_role("sys_admin"));
        user_roles.client_roles.insert(
            "my-service".to_string(),
            vec![sample_role("read")],
        );

        let json = serde_json::to_string(&user_roles).unwrap();
        let deserialized: UserRoles = serde_json::from_str(&json).unwrap();
        assert_eq!(user_roles, deserialized);
    }
}
