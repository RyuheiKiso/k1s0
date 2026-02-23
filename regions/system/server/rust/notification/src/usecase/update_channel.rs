use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, Clone)]
pub struct UpdateChannelInput {
    pub id: Uuid,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateChannelError {
    #[error("channel not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateChannelUseCase {
    repo: Arc<dyn NotificationChannelRepository>,
}

impl UpdateChannelUseCase {
    pub fn new(repo: Arc<dyn NotificationChannelRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &UpdateChannelInput) -> Result<NotificationChannel, UpdateChannelError> {
        let mut channel = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateChannelError::Internal(e.to_string()))?
            .ok_or(UpdateChannelError::NotFound(input.id))?;

        if let Some(ref name) = input.name {
            channel.name = name.clone();
        }
        if let Some(enabled) = input.enabled {
            channel.enabled = enabled;
        }
        if let Some(ref config) = input.config {
            channel.config = config.clone();
        }
        channel.updated_at = chrono::Utc::now();

        self.repo
            .update(&channel)
            .await
            .map_err(|e| UpdateChannelError::Internal(e.to_string()))?;

        Ok(channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::notification_channel::NotificationChannel;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationChannelRepository::new();
        let channel = NotificationChannel::new(
            "email-channel".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id;
        let return_channel = channel.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(move |_| Ok(Some(return_channel.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateChannelUseCase::new(Arc::new(mock));
        let input = UpdateChannelInput {
            id: channel_id,
            name: Some("updated-channel".to_string()),
            enabled: Some(false),
            config: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-channel");
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationChannelRepository::new();
        let missing_id = Uuid::new_v4();
        mock.expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = UpdateChannelUseCase::new(Arc::new(mock));
        let input = UpdateChannelInput {
            id: missing_id,
            name: None,
            enabled: Some(true),
            config: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateChannelError::NotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
