use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetTemplateError {
    #[error("template not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetTemplateUseCase {
    repo: Arc<dyn NotificationTemplateRepository>,
}

impl GetTemplateUseCase {
    pub fn new(repo: Arc<dyn NotificationTemplateRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<NotificationTemplate, GetTemplateError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetTemplateError::Internal(e.to_string()))?
            .ok_or(GetTemplateError::NotFound(*id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_template_repository::MockNotificationTemplateRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationTemplateRepository::new();
        let template = NotificationTemplate::new(
            "welcome".to_string(),
            "email".to_string(),
            Some("Welcome".to_string()),
            "Hello!".to_string(),
        );
        let template_id = template.id;
        let return_template = template.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == template_id)
            .returning(move |_| Ok(Some(return_template.clone())));

        let uc = GetTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute(&template_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, template_id);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetTemplateError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
