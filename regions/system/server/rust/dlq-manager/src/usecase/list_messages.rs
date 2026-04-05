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

    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してからトピック別にメッセージ一覧を取得する。
    pub async fn execute(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)> {
        self.repo.find_by_topic(topic, page, page_size, tenant_id).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_list_messages_empty() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic()
            .returning(|_, _, _, _| Ok((vec![], 0)));

        let uc = ListMessagesUseCase::new(Arc::new(mock));
        let (messages, total) = uc.execute("orders.dlq.v1", 1, 20, "tenant-a").await.unwrap();
        assert!(messages.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_list_messages_with_results() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic().returning(|_, _, _, _| {
            let msg = DlqMessage::new(
                "orders.events.v1".to_string(),
                "failed".to_string(),
                serde_json::json!({}),
                3,
            );
            Ok((vec![msg], 1))
        });

        let uc = ListMessagesUseCase::new(Arc::new(mock));
        let (messages, total) = uc.execute("orders.dlq.v1", 1, 20, "tenant-a").await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_list_messages_pagination_params_passed() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic()
            .withf(|topic, page, page_size, _tenant_id| {
                topic == "payments.dlq.v1" && *page == 2 && *page_size == 5
            })
            .returning(|_, _, _, _| Ok((vec![], 0)));

        let uc = ListMessagesUseCase::new(Arc::new(mock));
        let result = uc.execute("payments.dlq.v1", 2, 5, "tenant-a").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_messages_repository_error_propagated() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("db connection failed")));

        let uc = ListMessagesUseCase::new(Arc::new(mock));
        let result = uc.execute("orders.dlq.v1", 1, 20, "tenant-a").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("db connection"));
    }
}
