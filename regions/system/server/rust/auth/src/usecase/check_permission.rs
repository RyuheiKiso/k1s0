use std::sync::Arc;

use crate::domain::repository::UserRepository;
use crate::domain::service::{AuthDomainService, RolePermissionTable};

/// `CheckPermissionInput` はパーミッション確認の入力。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CheckPermissionInput {
    #[serde(default)]
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub permission: String,
    pub resource: String,
}

/// `CheckPermissionOutput` はパーミッション確認の出力。
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckPermissionOutput {
    pub allowed: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
}

/// `CheckPermissionUseCase` はパーミッション確認ユースケース。
#[derive(Default)]
pub struct CheckPermissionUseCase {
    user_repo: Option<Arc<dyn UserRepository>>,
    role_permission_table: Option<Arc<RolePermissionTable>>,
}

impl CheckPermissionUseCase {
    #[allow(dead_code)]
    #[must_use] 
    pub fn new() -> Self {
        Self {
            user_repo: None,
            role_permission_table: None,
        }
    }

    pub fn with_user_repo(user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            user_repo: Some(user_repo),
            role_permission_table: None,
        }
    }

    pub fn with_user_repo_and_role_table(
        user_repo: Arc<dyn UserRepository>,
        role_permission_table: Arc<RolePermissionTable>,
    ) -> Self {
        Self {
            user_repo: Some(user_repo),
            role_permission_table: Some(role_permission_table),
        }
    }

    pub async fn execute(&self, input: &CheckPermissionInput) -> CheckPermissionOutput {
        let roles = self.resolve_roles(input).await;
        let allowed = if let Some(role_table) = &self.role_permission_table {
            role_table
                .check_permission(&roles, &input.resource, &input.permission)
                .await
                .unwrap_or_else(|| {
                    AuthDomainService::check_permission(&roles, &input.resource, &input.permission)
                })
        } else {
            AuthDomainService::check_permission(&roles, &input.resource, &input.permission)
        };

        if allowed {
            return CheckPermissionOutput {
                allowed: true,
                reason: String::new(),
            };
        }

        CheckPermissionOutput {
            allowed: false,
            reason: format!(
                "insufficient permissions: role(s) {:?} do not grant '{}' on '{}'",
                roles, input.permission, input.resource
            ),
        }
    }

    async fn resolve_roles(&self, input: &CheckPermissionInput) -> Vec<String> {
        let user_id = input
            .user_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());

        let Some(user_id) = user_id else {
            return input.roles.clone();
        };
        let Some(user_repo) = &self.user_repo else {
            return input.roles.clone();
        };

        match user_repo.get_roles(user_id).await {
            Ok(user_roles) => {
                let mut roles = Vec::new();
                roles.extend(user_roles.realm_roles.into_iter().map(|r| r.name));
                for client_roles in user_roles.client_roles.into_values() {
                    roles.extend(client_roles.into_iter().map(|r| r.name));
                }
                if roles.is_empty() {
                    input.roles.clone()
                } else {
                    roles
                }
            }
            Err(_) => input.roles.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::user::{Role, UserRoles};
    use crate::domain::repository::user_repository::MockUserRepository;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_sys_admin_all_allowed() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            user_id: None,
            roles: vec!["sys_admin".to_string()],
            permission: "delete".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input).await;
        assert!(output.allowed);
        assert!(output.reason.is_empty());
    }

    #[tokio::test]
    async fn test_sys_operator_admin_denied() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            user_id: None,
            roles: vec!["sys_operator".to_string()],
            permission: "admin".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input).await;
        assert!(!output.allowed);
    }

    #[tokio::test]
    async fn test_fallback_to_request_roles_when_user_id_empty() {
        let uc = CheckPermissionUseCase::new();
        let input = CheckPermissionInput {
            user_id: Some(String::new()),
            roles: vec!["sys_auditor".to_string()],
            permission: "read".to_string(),
            resource: "audit_logs".to_string(),
        };
        let output = uc.execute(&input).await;
        assert!(output.allowed);
    }

    #[tokio::test]
    async fn test_user_roles_loaded_by_user_id() {
        let mut mock_user_repo = MockUserRepository::new();
        mock_user_repo
            .expect_get_roles()
            .withf(|id| id == "user-001")
            .returning(|_| {
                Ok(UserRoles {
                    user_id: "user-001".to_string(),
                    realm_roles: vec![Role {
                        id: "r-1".to_string(),
                        name: "sys_admin".to_string(),
                        description: String::new(),
                    }],
                    client_roles: HashMap::new(),
                })
            });
        let uc = CheckPermissionUseCase::with_user_repo(Arc::new(mock_user_repo));
        let input = CheckPermissionInput {
            user_id: Some("user-001".to_string()),
            roles: vec!["user".to_string()],
            permission: "admin".to_string(),
            resource: "users".to_string(),
        };
        let output = uc.execute(&input).await;
        assert!(output.allowed);
    }
}
