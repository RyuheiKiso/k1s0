use std::sync::Arc;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetChannelError {
    #[error("channel not found: {0}")]
    NotFound(String),

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

    /// MEDIUM-RUST-001 監査対応: `tenant_id` を受け取りリポジトリに伝播して RLS を有効化する。
    pub async fn execute(&self, id: &str, tenant_id: &str) -> Result<NotificationChannel, GetChannelError> {
        self.repo
            .find_by_id(id, tenant_id)
            .await
            .map_err(|e| GetChannelError::Internal(e.to_string()))?
            .ok_or_else(|| GetChannelError::NotFound(id.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            "system".to_string(),
            true,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        mock.expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id, _tenant_id| id == channel_id.as_str()
            })
            .returning(move |_, _| Ok(Some(return_channel.clone())));

        let uc = GetChannelUseCase::new(Arc::new(mock));
        let result = uc.execute(&channel_id, "tenant_a").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, channel_id);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetChannelUseCase::new(Arc::new(mock));
        let result = uc.execute("ch_missing", "tenant_a").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetChannelError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
