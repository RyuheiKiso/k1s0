use async_trait::async_trait;

use crate::error::OutboxError;
use crate::message::OutboxMessage;

/// OutboxStore はアウトボックスメッセージの永続化インターフェース。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OutboxStore: Send + Sync {
    /// メッセージをアウトボックステーブルに保存する。
    async fn save(&self, message: &OutboxMessage) -> Result<(), OutboxError>;

    /// 処理待ちのメッセージを一覧取得する（最大 limit 件）。
    async fn fetch_pending(&self, limit: u32) -> Result<Vec<OutboxMessage>, OutboxError>;

    /// メッセージのステータスを更新する。
    async fn update(&self, message: &OutboxMessage) -> Result<(), OutboxError>;

    /// 配信完了メッセージを削除する（保持期間超過後）。
    async fn delete_delivered(&self, older_than_days: u32) -> Result<u64, OutboxError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{OutboxMessage, OutboxStatus};

    #[tokio::test]
    async fn test_mock_save_success() {
        let mut mock = MockOutboxStore::new();
        mock.expect_save().returning(|_| Ok(()));

        let msg = OutboxMessage::new("test.topic", "key", serde_json::json!({}));
        let result = mock.save(&msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_fetch_pending() {
        let mut mock = MockOutboxStore::new();
        let msg = OutboxMessage::new("test.topic", "key", serde_json::json!({}));
        let expected = vec![msg];
        let expected_clone = expected.clone();

        mock.expect_fetch_pending()
            .withf(|&limit| limit == 10)
            .returning(move |_| Ok(expected_clone.clone()));

        let result = mock.fetch_pending(10).await;
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].status, OutboxStatus::Pending);
    }

    #[tokio::test]
    async fn test_mock_update() {
        let mut mock = MockOutboxStore::new();
        mock.expect_update().returning(|_| Ok(()));

        let mut msg = OutboxMessage::new("test.topic", "key", serde_json::json!({}));
        msg.mark_delivered();
        let result = mock.update(&msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_delete_delivered() {
        let mut mock = MockOutboxStore::new();
        mock.expect_delete_delivered()
            .withf(|&days| days == 30)
            .returning(|_| Ok(5));

        let result = mock.delete_delivered(30).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }
}
