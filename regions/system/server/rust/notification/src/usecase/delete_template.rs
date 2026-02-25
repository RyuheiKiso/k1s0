use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteTemplateError {
    #[error("template not found: {0}")]
    NotFound(Uuid),

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

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteTemplateError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteTemplateError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteTemplateError::NotFound(*id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_template_repository::MockNotificationTemplateRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteTemplateUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteTemplateError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
