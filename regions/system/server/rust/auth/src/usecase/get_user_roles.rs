use std::sync::Arc;

use crate::domain::entity::user::UserRoles;
use crate::domain::repository::UserRepository;

/// GetUserRolesError はユーザーロール取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetUserRolesError {
    #[error("user not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetUserRolesUseCase はユーザーロール取得ユースケース。
pub struct GetUserRolesUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl GetUserRolesUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// ユーザー ID でロール一覧を取得する。
    pub async fn execute(&self, user_id: &str) -> Result<UserRoles, GetUserRolesError> {
        self.user_repo.get_roles(user_id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GetUserRolesError::NotFound(user_id.to_string())
            } else {
                GetUserRolesError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::user::Role;
    use crate::domain::repository::user_repository::MockUserRepository;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_execute_success() {
        let mut mock = MockUserRepository::new();
        mock.expect_get_roles()
            .withf(|id| id == "user-uuid-1234")
            .returning(|_| {
                Ok(UserRoles {
                    user_id: "user-uuid-1234".to_string(),
                    realm_roles: vec![
                        Role {
                            id: "role-1".to_string(),
                            name: "user".to_string(),
                            description: "General user".to_string(),
                        },
                        Role {
                            id: "role-2".to_string(),
                            name: "sys_auditor".to_string(),
                            description: "Auditor".to_string(),
                        },
                    ],
                    client_roles: HashMap::from([(
                        "order-service".to_string(),
                        vec![Role {
                            id: "role-3".to_string(),
                            name: "read".to_string(),
                            description: "Read access".to_string(),
                        }],
                    )]),
                })
            });

        let uc = GetUserRolesUseCase::new(Arc::new(mock));
        let result = uc.execute("user-uuid-1234").await;
        assert!(result.is_ok());

        let roles = result.unwrap();
        assert_eq!(roles.realm_roles.len(), 2);
        assert_eq!(roles.client_roles.get("order-service").unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_not_found() {
        let mut mock = MockUserRepository::new();
        mock.expect_get_roles()
            .returning(|_| Err(anyhow::anyhow!("user not found")));

        let uc = GetUserRolesUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent").await;
        assert!(matches!(
            result.unwrap_err(),
            GetUserRolesError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_execute_internal_error() {
        let mut mock = MockUserRepository::new();
        mock.expect_get_roles()
            .returning(|_| Err(anyhow::anyhow!("connection refused")));

        let uc = GetUserRolesUseCase::new(Arc::new(mock));
        let result = uc.execute("user-1").await;
        assert!(matches!(
            result.unwrap_err(),
            GetUserRolesError::Internal(_)
        ));
    }
}
