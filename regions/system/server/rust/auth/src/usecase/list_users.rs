use std::sync::Arc;

use crate::domain::entity::user::UserListResult;
use crate::domain::repository::UserRepository;

/// ListUsersError はユーザー一覧取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum ListUsersError {
    #[error("invalid page: {0}")]
    InvalidPage(String),

    #[error("invalid page_size: {0}")]
    InvalidPageSize(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// ListUsersParams はユーザー一覧取得のクエリパラメータを表す。
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ListUsersParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub search: Option<String>,
    pub enabled: Option<bool>,
}

/// ListUsersUseCase はユーザー一覧取得ユースケース。
pub struct ListUsersUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl ListUsersUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// ユーザー一覧を取得する。
    pub async fn execute(&self, params: &ListUsersParams) -> Result<UserListResult, ListUsersError> {
        let page = params.page.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(20);

        // バリデーション
        if page < 1 {
            return Err(ListUsersError::InvalidPage(
                "page must be >= 1".to_string(),
            ));
        }
        if page_size < 1 || page_size > 100 {
            return Err(ListUsersError::InvalidPageSize(
                "page_size must be between 1 and 100".to_string(),
            ));
        }

        self.user_repo
            .list(
                page,
                page_size,
                params.search.clone(),
                params.enabled,
            )
            .await
            .map_err(|e| ListUsersError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::user::{Pagination, User};
    use crate::domain::repository::user_repository::MockUserRepository;
    use std::collections::HashMap;

    fn make_test_users() -> Vec<User> {
        vec![
            User {
                id: "user-1".to_string(),
                username: "taro.yamada".to_string(),
                email: "taro@example.com".to_string(),
                first_name: "Taro".to_string(),
                last_name: "Yamada".to_string(),
                enabled: true,
                email_verified: true,
                created_at: chrono::Utc::now(),
                attributes: HashMap::new(),
            },
            User {
                id: "user-2".to_string(),
                username: "hanako.tanaka".to_string(),
                email: "hanako@example.com".to_string(),
                first_name: "Hanako".to_string(),
                last_name: "Tanaka".to_string(),
                enabled: true,
                email_verified: false,
                created_at: chrono::Utc::now(),
                attributes: HashMap::new(),
            },
        ]
    }

    #[tokio::test]
    async fn test_list_users_success_default_params() {
        let mut mock = MockUserRepository::new();
        let users = make_test_users();

        mock.expect_list()
            .withf(|page, page_size, search, enabled| {
                *page == 1 && *page_size == 20 && search.is_none() && enabled.is_none()
            })
            .returning(move |page, page_size, _, _| {
                Ok(UserListResult {
                    users: users.clone(),
                    pagination: Pagination {
                        total_count: 2,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let uc = ListUsersUseCase::new(Arc::new(mock));
        let result = uc.execute(&ListUsersParams::default()).await;
        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.users.len(), 2);
        assert_eq!(list.pagination.total_count, 2);
        assert_eq!(list.pagination.page, 1);
        assert_eq!(list.pagination.page_size, 20);
        assert!(!list.pagination.has_next);
    }

    #[tokio::test]
    async fn test_list_users_with_search() {
        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .withf(|_, _, search, _| *search == Some("taro".to_string()))
            .returning(|page, page_size, _, _| {
                Ok(UserListResult {
                    users: vec![User {
                        id: "user-1".to_string(),
                        username: "taro.yamada".to_string(),
                        email: "taro@example.com".to_string(),
                        first_name: "Taro".to_string(),
                        last_name: "Yamada".to_string(),
                        enabled: true,
                        email_verified: true,
                        created_at: chrono::Utc::now(),
                        attributes: HashMap::new(),
                    }],
                    pagination: Pagination {
                        total_count: 1,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let uc = ListUsersUseCase::new(Arc::new(mock));
        let params = ListUsersParams {
            search: Some("taro".to_string()),
            ..Default::default()
        };
        let result = uc.execute(&params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().users.len(), 1);
    }

    #[tokio::test]
    async fn test_list_users_with_pagination() {
        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .withf(|page, page_size, _, _| *page == 2 && *page_size == 10)
            .returning(|page, page_size, _, _| {
                Ok(UserListResult {
                    users: vec![],
                    pagination: Pagination {
                        total_count: 15,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let uc = ListUsersUseCase::new(Arc::new(mock));
        let params = ListUsersParams {
            page: Some(2),
            page_size: Some(10),
            ..Default::default()
        };
        let result = uc.execute(&params).await;
        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.pagination.page, 2);
        assert_eq!(list.pagination.page_size, 10);
    }

    #[tokio::test]
    async fn test_list_users_invalid_page() {
        let mock = MockUserRepository::new();
        let uc = ListUsersUseCase::new(Arc::new(mock));

        let params = ListUsersParams {
            page: Some(0),
            ..Default::default()
        };
        let result = uc.execute(&params).await;
        assert!(matches!(result.unwrap_err(), ListUsersError::InvalidPage(_)));
    }

    #[tokio::test]
    async fn test_list_users_invalid_page_size_too_large() {
        let mock = MockUserRepository::new();
        let uc = ListUsersUseCase::new(Arc::new(mock));

        let params = ListUsersParams {
            page_size: Some(101),
            ..Default::default()
        };
        let result = uc.execute(&params).await;
        assert!(matches!(result.unwrap_err(), ListUsersError::InvalidPageSize(_)));
    }

    #[tokio::test]
    async fn test_list_users_invalid_page_size_zero() {
        let mock = MockUserRepository::new();
        let uc = ListUsersUseCase::new(Arc::new(mock));

        let params = ListUsersParams {
            page_size: Some(0),
            ..Default::default()
        };
        let result = uc.execute(&params).await;
        assert!(matches!(result.unwrap_err(), ListUsersError::InvalidPageSize(_)));
    }
}
