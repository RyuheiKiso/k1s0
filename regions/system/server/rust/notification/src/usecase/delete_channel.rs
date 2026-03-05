use std::sync::Arc;

use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, thiserror::Error)]
pub enum DeleteChannelError {
    #[error("channel not found: {0}")]
    NotFound(String),

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

    pub async fn execute(&self, id: &str) -> Result<(), DeleteChannelError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteChannelError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteChannelError::NotFound(id.to_string()));
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
        let result = uc.execute("ch_any").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteChannelUseCase::new(Arc::new(mock));
        let result = uc.execute("ch_missing").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteChannelError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
