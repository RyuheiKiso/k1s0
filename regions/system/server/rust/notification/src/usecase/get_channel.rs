use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetChannelError {
    #[error("channel not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetChannelUseCase {
    repo: Arc<dyn NotificationChannelRepository>,
}

impl GetChannelUseCase {
    pub fn new(repo: Arc<dyn NotificationChannelRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<NotificationChannel, GetChannelError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetChannelError::Internal(e.to_string()))?
            .ok_or(GetChannelError::NotFound(*id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationChannelRepository::new();
        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id;
        let return_channel = channel.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(move |_| Ok(Some(return_channel.clone())));

        let uc = GetChannelUseCase::new(Arc::new(mock));
        let result = uc.execute(&channel_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, channel_id);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetChannelUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetChannelError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
