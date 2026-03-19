use std::sync::Arc;

use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteTemplateError {
    #[error("template not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteTemplateUseCase {
    repo: Arc<dyn NotificationTemplateRepository>,
}

impl DeleteTemplateUseCase {
    pub fn new(repo: Arc<dyn NotificationTemplateRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DeleteTemplateError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteTemplateError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteTemplateError::NotFound(id.to_string()));
        }

        Ok(())
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
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute("tpl_any").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute("tpl_missing").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteTemplateError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
