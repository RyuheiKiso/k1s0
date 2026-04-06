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

    /// MEDIUM-RUST-001 監査対応: tenant_id を受け取りリポジトリに伝播して RLS を有効化する。
    #[allow(dead_code)]
    pub async fn execute(&self, tenant_id: &str) -> Result<Vec<NotificationChannel>, ListChannelsError> {
        self.repo
            .find_all(tenant_id)
            .await
            .map_err(|e| ListChannelsError::Internal(e.to_string()))
    }

    /// MEDIUM-RUST-001 監査対応: tenant_id を受け取りリポジトリに伝播して RLS を有効化する。
    pub async fn execute_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> Result<(Vec<NotificationChannel>, u64), ListChannelsError> {
        self.repo
            .find_all_paginated(tenant_id, page, page_size, channel_type, enabled_only)
            .await
            .map_err(|e| ListChannelsError::Internal(e.to_string()))
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
        mock.expect_find_all().returning(|_| Ok(vec![]));

        let uc = ListChannelsUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant_a").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_find_all()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ListChannelsUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant_a").await;
        assert!(result.is_err());
    }
}
