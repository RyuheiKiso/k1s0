use std::sync::Arc;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListChannelsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListChannelsUseCase {
    repo: Arc<dyn NotificationChannelRepository>,
}

impl ListChannelsUseCase {
    pub fn new(repo: Arc<dyn NotificationChannelRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<NotificationChannel>, ListChannelsError> {
        self.repo
            .find_all()
            .await
            .map_err(|e| ListChannelsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_find_all().returning(|| Ok(vec![]));

        let uc = ListChannelsUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_find_all()
            .returning(|| Err(anyhow::anyhow!("db error")));

        let uc = ListChannelsUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_err());
    }
}
