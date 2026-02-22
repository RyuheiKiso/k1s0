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

    /// IDで DLQ メッセージを取得する。見つからなければエラー。
    pub async fn execute(&self, id: Uuid) -> anyhow::Result<DlqMessage> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("dlq message not found: {}", id))
    }
}

#[cfg(test)]
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
            .returning(move |_| Ok(Some(msg_clone.clone())));

        let uc = GetMessageUseCase::new(Arc::new(mock));
        let result = uc.execute(msg_id).await.unwrap();
        assert_eq!(result.id, msg_id);
    }

    #[tokio::test]
    async fn test_get_message_not_found() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetMessageUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
