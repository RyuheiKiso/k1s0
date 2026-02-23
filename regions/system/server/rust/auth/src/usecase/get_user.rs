use std::sync::Arc;

use crate::domain::entity::user::User;
use crate::domain::repository::UserRepository;

/// GetUserError はユーザー取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetUserError {
    #[error("user not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetUserUseCase はユーザー情報取得ユースケース。
pub struct GetUserUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl GetUserUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// ユーザー ID でユーザー情報を取得する。
    pub async fn execute(&self, user_id: &str) -> Result<User, GetUserError> {
        self.user_repo.find_by_id(user_id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GetUserError::NotFound(user_id.to_string())
            } else {
                GetUserError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::user_repository::MockUserRepository;
    use std::collections::HashMap;

    fn make_test_user() -> User {
        User {
            id: "user-uuid-1234".to_string(),
            username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            first_name: "Taro".to_string(),
            last_name: "Yamada".to_string(),
            enabled: true,
            email_verified: true,
            created_at: chrono::Utc::now(),
            attributes: HashMap::from([(
                "department".to_string(),
                vec!["engineering".to_string()],
            )]),
        }
    }

    #[tokio::test]
    async fn test_get_user_success() {
        let mut mock = MockUserRepository::new();
        let user = make_test_user();
        let expected_user = user.clone();

        mock.expect_find_by_id()
            .withf(|id| id == "user-uuid-1234")
            .returning(move |_| Ok(user.clone()));

        let uc = GetUserUseCase::new(Arc::new(mock));
        let result = uc.execute("user-uuid-1234").await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.id, expected_user.id);
        assert_eq!(user.username, expected_user.username);
        assert_eq!(user.email, expected_user.email);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("user not found")));

        let uc = GetUserUseCase::new(Arc::new(mock));
        let result = uc.execute("nonexistent-user").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetUserError::NotFound(id) => assert_eq!(id, "nonexistent-user"),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_user_internal_error() {
        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("connection refused")));

        let uc = GetUserUseCase::new(Arc::new(mock));
        let result = uc.execute("user-1").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetUserError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }
}
