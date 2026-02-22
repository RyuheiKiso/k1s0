use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User は Keycloak ユーザーを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub attributes: HashMap<String, Vec<String>>,
}

/// Role はロールを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// UserRoles はユーザーに割り当てられたロール一覧を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserRoles {
    pub user_id: String,
    pub realm_roles: Vec<Role>,
    pub client_roles: HashMap<String, Vec<Role>>,
}

/// Pagination はページネーションパラメータを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pagination {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

/// UserListResult はユーザー一覧とページネーション結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserListResult {
    pub users: Vec<User>,
    pub pagination: Pagination,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: "user-uuid-1234".to_string(),
            username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            first_name: "Taro".to_string(),
            last_name: "Yamada".to_string(),
            enabled: true,
            email_verified: true,
            created_at: Utc::now(),
            attributes: HashMap::new(),
        };

        assert_eq!(user.id, "user-uuid-1234");
        assert_eq!(user.username, "taro.yamada");
        assert!(user.enabled);
        assert!(user.email_verified);
    }

    #[test]
    fn test_user_serialization_roundtrip() {
        let mut attrs = HashMap::new();
        attrs.insert("department".to_string(), vec!["engineering".to_string()]);

        let user = User {
            id: "user-uuid-1234".to_string(),
            username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            first_name: "Taro".to_string(),
            last_name: "Yamada".to_string(),
            enabled: true,
            email_verified: true,
            created_at: Utc::now(),
            attributes: attrs,
        };

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();
        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_role_creation() {
        let role = Role {
            id: "role-uuid-1".to_string(),
            name: "sys_admin".to_string(),
            description: "System administrator".to_string(),
        };

        assert_eq!(role.name, "sys_admin");
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination {
            total_count: 150,
            page: 1,
            page_size: 20,
            has_next: true,
        };

        assert_eq!(pagination.total_count, 150);
        assert!(pagination.has_next);
    }
}
