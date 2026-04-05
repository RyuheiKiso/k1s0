use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::DlqMessage;
use crate::domain::repository::DlqMessageRepository;

/// GetMessageUseCase は DLQ メッセージ詳細取得を担う。
pub struct GetMessageUseCase {
    repo: Arc<dyn DlqMessageRepository>,
}

impl GetMessageUseCase {
    pub fn new(repo: Arc<dyn DlqMessageRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してから ID で DLQ メッセージを取得する。
    pub async fn execute(&self, id: Uuid, tenant_id: &str) -> anyhow::Result<DlqMessage> {
        self.repo
            .find_by_id(id, tenant_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("dlq message not found: {}", id))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_get_message_found() {
        let msg = DlqMessage::new(
            "orders.events.v1".to_string(),
            "failed".to_string(),
            serde_json::json!({}),
            3,
        );
        let msg_id = msg.id;
        let msg_clone = msg.clone();

        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id()
            .returning(move |_, _| Ok(Some(msg_clone.clone())));

        let uc = GetMessageUseCase::new(Arc::new(mock));
        let result = uc.execute(msg_id, "tenant-a").await.unwrap();
        assert_eq!(result.id, msg_id);
    }

    #[tokio::test]
    async fn test_get_message_not_found() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetMessageUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4(), "tenant-a").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
