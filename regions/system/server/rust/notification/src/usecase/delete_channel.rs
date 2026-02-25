use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteChannelError {
    #[error("channel not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteChannelUseCase {
    repo: Arc<dyn NotificationChannelRepository>,
}

impl DeleteChannelUseCase {
    pub fn new(repo: Arc<dyn NotificationChannelRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteChannelError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteChannelError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteChannelError::NotFound(*id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteChannelUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteChannelUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteChannelError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
