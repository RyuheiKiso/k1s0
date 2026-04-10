use std::sync::Arc;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetTemplateError {
    #[error("template not found: {0}")]
    NotFound(String),

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

    /// テナントスコープでテンプレートを取得する
    pub async fn execute(&self, id: &str, tenant_id: &str) -> Result<NotificationTemplate, GetTemplateError> {
        self.repo
            .find_by_id(id, tenant_id)
            .await
            .map_err(|e| GetTemplateError::Internal(e.to_string()))?
            .ok_or_else(|| GetTemplateError::NotFound(id.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_template_repository::MockNotificationTemplateRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationTemplateRepository::new();
        let template = NotificationTemplate::new(
            "tenant_a".to_string(),
            "welcome".to_string(),
            "email".to_string(),
            Some("Welcome".to_string()),
            "Hello!".to_string(),
        );
        let template_id = template.id.clone();
        let return_template = template.clone();

        mock.expect_find_by_id()
            .withf({
                let template_id = template_id.clone();
                move |id, _tenant_id| id == template_id.as_str()
            })
            .returning(move |_, _| Ok(Some(return_template.clone())));

        let uc = GetTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute(&template_id, "tenant_a").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, template_id);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute("tpl_missing", "tenant_a").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetTemplateError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
