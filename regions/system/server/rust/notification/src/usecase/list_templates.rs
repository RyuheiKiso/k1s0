use std::sync::Arc;

use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationTemplateRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListTemplatesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListTemplatesUseCase {
    repo: Arc<dyn NotificationTemplateRepository>,
}

impl ListTemplatesUseCase {
    pub fn new(repo: Arc<dyn NotificationTemplateRepository>) -> Self {
        Self { repo }
    }

    /// テナントスコープで全テンプレートを取得する
    #[allow(dead_code)]
    pub async fn execute(&self, tenant_id: &str) -> Result<Vec<NotificationTemplate>, ListTemplatesError> {
        self.repo
            .find_all(tenant_id)
            .await
            .map_err(|e| ListTemplatesError::Internal(e.to_string()))
    }

    /// テナントスコープでページネーション付きテンプレート一覧を取得する
    pub async fn execute_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
    ) -> Result<(Vec<NotificationTemplate>, u64), ListTemplatesError> {
        self.repo
            .find_all_paginated(tenant_id, page, page_size, channel_type)
            .await
            .map_err(|e| ListTemplatesError::Internal(e.to_string()))
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
        mock.expect_find_all().returning(|_| Ok(vec![]));

        let uc = ListTemplatesUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant_a").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationTemplateRepository::new();
        mock.expect_find_all()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ListTemplatesUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant_a").await;
        assert!(result.is_err());
    }
}
