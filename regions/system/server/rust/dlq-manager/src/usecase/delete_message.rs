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

    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してから DLQ メッセージを削除する。
    pub async fn execute(&self, id: Uuid, tenant_id: &str) -> anyhow::Result<()> {
        self.repo.delete(id, tenant_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_delete_message_success() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_delete().returning(|_, _| Ok(()));

        let uc = DeleteMessageUseCase::new(Arc::new(mock));
        assert!(uc.execute(Uuid::new_v4(), "tenant-a").await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_message_error() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("database error")));

        let uc = DeleteMessageUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4(), "tenant-a").await;
        assert!(result.is_err());
    }
}
