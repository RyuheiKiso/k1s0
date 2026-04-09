use std::sync::Arc;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;
use crate::domain::service::NotificationDomainService;

/// テンプレート作成の入力パラメータ。tenant_id でテナント分離を強制する。
#[derive(Debug, Clone)]
pub struct CreateTemplateInput {
    /// RLS テナント分離に使用するテナント識別子
    pub tenant_id: String,
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTemplateError {
    #[error("validation error: {0}")]
    Validation(String),

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
        if let Some(subject_template) = &input.subject_template {
            NotificationDomainService::validate_template_body(subject_template)
                .map_err(CreateTemplateError::Validation)?;
        }
        NotificationDomainService::validate_template_body(&input.body_template)
            .map_err(CreateTemplateError::Validation)?;

        // テナント ID を含めてテンプレートを生成する
        let template = NotificationTemplate::new(
            input.tenant_id.clone(),
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_template_repository::MockNotificationTemplateRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateTemplateUseCase::new(Arc::new(mock));
        let input = CreateTemplateInput {
            tenant_id: "tenant_a".to_string(),
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
            tenant_id: "tenant_a".to_string(),
            name: "fail".to_string(),
            channel_type: "sms".to_string(),
            subject_template: None,
            body_template: "test".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}
