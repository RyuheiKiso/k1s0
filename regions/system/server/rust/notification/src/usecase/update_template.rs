use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, Clone)]
pub struct UpdateTemplateInput {
    pub id: Uuid,
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTemplateError {
    #[error("template not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateTemplateUseCase {
    repo: Arc<dyn NotificationTemplateRepository>,
}

impl UpdateTemplateUseCase {
    pub fn new(repo: Arc<dyn NotificationTemplateRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &UpdateTemplateInput,
    ) -> Result<NotificationTemplate, UpdateTemplateError> {
        let mut template = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateTemplateError::Internal(e.to_string()))?
            .ok_or(UpdateTemplateError::NotFound(input.id))?;

        if let Some(ref name) = input.name {
            template.name = name.clone();
        }
        if let Some(ref subject) = input.subject_template {
            template.subject_template = Some(subject.clone());
        }
        if let Some(ref body) = input.body_template {
            template.body_template = body.clone();
        }
        template.updated_at = chrono::Utc::now();

        self.repo
            .update(&template)
            .await
            .map_err(|e| UpdateTemplateError::Internal(e.to_string()))?;

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
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateTemplateUseCase::new(Arc::new(mock));
        let input = UpdateTemplateInput {
            id: template_id,
            name: Some("updated-welcome".to_string()),
            subject_template: None,
            body_template: Some("Updated body".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-welcome");
        assert_eq!(updated.body_template, "Updated body");
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateTemplateUseCase::new(Arc::new(mock));
        let input = UpdateTemplateInput {
            id: Uuid::new_v4(),
            name: None,
            subject_template: None,
            body_template: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateTemplateError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
