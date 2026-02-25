use std::sync::Arc;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListTemplatesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListTemplatesUseCase {
    repo: Arc<dyn NotificationTemplateRepository>,
}

impl ListTemplatesUseCase {
    pub fn new(repo: Arc<dyn NotificationTemplateRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<NotificationTemplate>, ListTemplatesError> {
        self.repo
            .find_all()
            .await
            .map_err(|e| ListTemplatesError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_template_repository::MockNotificationTemplateRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_find_all().returning(|| Ok(vec![]));

        let uc = ListTemplatesUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_find_all()
            .returning(|| Err(anyhow::anyhow!("db error")));

        let uc = ListTemplatesUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_err());
    }
}
