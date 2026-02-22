use async_trait::async_trait;

use crate::domain::entity::user::{User, UserListResult, UserRoles};

/// UserRepository はユーザー情報取得のためのリポジトリトレイト。
/// 実装は Keycloak Admin API を通じてユーザー情報を取得する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// ユーザー ID でユーザー情報を取得する。
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User>;

    /// ユーザー一覧を取得する。
    async fn list(
        &self,
        page: i32,
        page_size: i32,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult>;

    /// ユーザーのロール一覧を取得する。
    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_mock_user_repository_find_by_id() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .withf(|id| id == "user-1")
            .returning(|_| {
                Ok(User {
                    id: "user-1".to_string(),
                    username: "test.user".to_string(),
                    email: "test@example.com".to_string(),
                    first_name: "Test".to_string(),
                    last_name: "User".to_string(),
                    enabled: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                })
            });

        let result = mock.find_by_id("user-1").await.unwrap();
        assert_eq!(result.id, "user-1");
        assert_eq!(result.username, "test.user");
    }

    #[tokio::test]
    async fn test_mock_user_repository_list() {
        let mut mock = MockUserRepository::new();
        mock.expect_list().returning(|page, page_size, _, _| {
            Ok(UserListResult {
                users: vec![],
                pagination: crate::domain::entity::user::Pagination {
                    total_count: 0,
                    page,
                    page_size,
                    has_next: false,
                },
            })
        });

        let result = mock.list(1, 20, None, None).await.unwrap();
        assert_eq!(result.pagination.page, 1);
        assert_eq!(result.pagination.page_size, 20);
        assert!(result.users.is_empty());
    }
}
