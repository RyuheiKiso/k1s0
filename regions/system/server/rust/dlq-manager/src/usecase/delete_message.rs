use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::DlqMessageRepository;

/// DeleteMessageUseCase は DLQ メッセージ削除を担う。
pub struct DeleteMessageUseCase {
    repo: Arc<dyn DlqMessageRepository>,
}

impl DeleteMessageUseCase {
    pub fn new(repo: Arc<dyn DlqMessageRepository>) -> Self {
        Self { repo }
    }

    /// DLQ メッセージを削除する。
    pub async fn execute(&self, id: Uuid) -> anyhow::Result<()> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_delete_message_success() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_delete().returning(|_| Ok(()));

        let uc = DeleteMessageUseCase::new(Arc::new(mock));
        assert!(uc.execute(Uuid::new_v4()).await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_message_error() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_delete()
            .returning(|_| Err(anyhow::anyhow!("database error")));

        let uc = DeleteMessageUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}
