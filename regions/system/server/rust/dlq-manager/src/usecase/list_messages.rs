use std::sync::Arc;

use crate::domain::entity::DlqMessage;
use crate::domain::repository::DlqMessageRepository;

/// ListMessagesUseCase はトピック別の DLQ メッセージ一覧取得を担う。
pub struct ListMessagesUseCase {
    repo: Arc<dyn DlqMessageRepository>,
}

impl ListMessagesUseCase {
    pub fn new(repo: Arc<dyn DlqMessageRepository>) -> Self {
        Self { repo }
    }

    /// トピック別にメッセージ一覧を取得する。
    pub async fn execute(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)> {
        self.repo.find_by_topic(topic, page, page_size).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_list_messages_empty() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic().returning(|_, _, _| Ok((vec![], 0)));

        let uc = ListMessagesUseCase::new(Arc::new(mock));
        let (messages, total) = uc.execute("orders.dlq.v1", 1, 20).await.unwrap();
        assert!(messages.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_list_messages_with_results() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic().returning(|_, _, _| {
            let msg = DlqMessage::new(
                "orders.events.v1".to_string(),
                "failed".to_string(),
                serde_json::json!({}),
                3,
            );
            Ok((vec![msg], 1))
        });

        let uc = ListMessagesUseCase::new(Arc::new(mock));
        let (messages, total) = uc.execute("orders.dlq.v1", 1, 20).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(total, 1);
    }
}
