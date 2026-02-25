use std::sync::Arc;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, Clone)]
pub struct CreateTemplateInput {
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTemplateError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateTemplateUseCase {
    repo: Arc<dyn NotificationTemplateRepository>,
}

impl CreateTemplateUseCase {
    pub fn new(repo: Arc<dyn NotificationTemplateRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CreateTemplateInput,
    ) -> Result<NotificationTemplate, CreateTemplateError> {
        let template = NotificationTemplate::new(
            input.name.clone(),
            input.channel_type.clone(),
            input.subject_template.clone(),
            input.body_template.clone(),
        );

        self.repo
            .create(&template)
            .await
            .map_err(|e| CreateTemplateError::Internal(e.to_string()))?;

        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_template_repository::MockNotificationTemplateRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateTemplateUseCase::new(Arc::new(mock));
        let input = CreateTemplateInput {
            name: "welcome".to_string(),
            channel_type: "email".to_string(),
            subject_template: Some("Welcome {{name}}".to_string()),
            body_template: "Hello {{name}}, welcome!".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let template = result.unwrap();
        assert_eq!(template.name, "welcome");
        assert_eq!(template.channel_type, "email");
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateTemplateUseCase::new(Arc::new(mock));
        let input = CreateTemplateInput {
            name: "fail".to_string(),
            channel_type: "sms".to_string(),
            subject_template: None,
            body_template: "test".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}
