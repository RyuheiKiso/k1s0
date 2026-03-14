use std::sync::Arc;

use chrono::Utc;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateAppInput {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateAppError {
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateAppUseCase {
    repo: Arc<dyn AppRepository>,
}

impl CreateAppUseCase {
    pub fn new(repo: Arc<dyn AppRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: CreateAppInput) -> Result<App, CreateAppError> {
        if input.name.trim().is_empty() {
            return Err(CreateAppError::ValidationError(
                "name must not be empty".to_string(),
            ));
        }
        if input.category.trim().is_empty() {
            return Err(CreateAppError::ValidationError(
                "category must not be empty".to_string(),
            ));
        }

        let now = Utc::now();
        let app = App {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name,
            description: input.description,
            category: input.category,
            icon_url: input.icon_url,
            created_at: now,
            updated_at: now,
        };

        self.repo
            .create(&app)
            .await
            .map_err(|e| CreateAppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    #[tokio::test]
    async fn test_create_app_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_create().returning(|app| Ok(app.clone()));

        let uc = CreateAppUseCase::new(Arc::new(mock));
        let input = CreateAppInput {
            name: "Test App".to_string(),
            description: Some("A test app".to_string()),
            category: "tools".to_string(),
            icon_url: None,
        };
        let result = uc.execute(input).await.unwrap();
        assert_eq!(result.name, "Test App");
        assert_eq!(result.category, "tools");
    }

    #[tokio::test]
    async fn test_create_app_empty_name() {
        let mock = MockAppRepository::new();
        let uc = CreateAppUseCase::new(Arc::new(mock));
        let input = CreateAppInput {
            name: "".to_string(),
            description: None,
            category: "tools".to_string(),
            icon_url: None,
        };
        let result = uc.execute(input).await;
        assert!(matches!(result, Err(CreateAppError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_create_app_empty_category() {
        let mock = MockAppRepository::new();
        let uc = CreateAppUseCase::new(Arc::new(mock));
        let input = CreateAppInput {
            name: "Test".to_string(),
            description: None,
            category: " ".to_string(),
            icon_url: None,
        };
        let result = uc.execute(input).await;
        assert!(matches!(result, Err(CreateAppError::ValidationError(_))));
    }
}
