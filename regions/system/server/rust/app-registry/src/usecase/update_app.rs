use std::sync::Arc;

use chrono::Utc;

use crate::domain::entity::app::App;
use crate::domain::repository::AppRepository;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateAppInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateAppError {
    #[error("app not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateAppUseCase {
    repo: Arc<dyn AppRepository>,
}

impl UpdateAppUseCase {
    pub fn new(repo: Arc<dyn AppRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: UpdateAppInput) -> Result<App, UpdateAppError> {
        if input.name.trim().is_empty() {
            return Err(UpdateAppError::ValidationError(
                "name must not be empty".to_string(),
            ));
        }

        let existing = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateAppError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateAppError::NotFound(input.id.clone()))?;

        let app = App {
            id: existing.id,
            name: input.name,
            description: input.description,
            category: input.category,
            icon_url: input.icon_url,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };

        self.repo
            .update(&app)
            .await
            .map_err(|e| UpdateAppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::app_repository::MockAppRepository;

    fn make_app() -> App {
        App {
            id: "app-1".to_string(),
            name: "Old Name".to_string(),
            description: None,
            category: "tools".to_string(),
            icon_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_update_app_success() {
        let mut mock = MockAppRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_app())));
        mock.expect_update().returning(|app| Ok(app.clone()));

        let uc = UpdateAppUseCase::new(Arc::new(mock));
        let input = UpdateAppInput {
            id: "app-1".to_string(),
            name: "New Name".to_string(),
            description: Some("Updated".to_string()),
            category: "utils".to_string(),
            icon_url: None,
        };
        let result = uc.execute(input).await.unwrap();
        assert_eq!(result.name, "New Name");
        assert_eq!(result.category, "utils");
    }

    #[tokio::test]
    async fn test_update_app_not_found() {
        let mut mock = MockAppRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateAppUseCase::new(Arc::new(mock));
        let input = UpdateAppInput {
            id: "missing".to_string(),
            name: "Name".to_string(),
            description: None,
            category: "tools".to_string(),
            icon_url: None,
        };
        let result = uc.execute(input).await;
        assert!(matches!(result, Err(UpdateAppError::NotFound(_))));
    }
}
