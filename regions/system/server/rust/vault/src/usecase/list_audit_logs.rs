use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::access_log::SecretAccessLog;
use crate::domain::repository::AccessLogRepository;

/// LOW-12 監査対応: keyset ページネーション入力。
/// OFFSET ではなくカーソル（前ページの最後のアイテムの id）を使用する。
pub struct ListAuditLogsInput {
    /// 前ページの最後のアイテムの id。None の場合は先頭ページを返す。
    pub after_id: Option<Uuid>,
    pub limit: u32,
}

/// LOW-12 監査対応: keyset ページネーション出力。
/// `next_cursor` が Some の場合は次のページが存在し、その値を `after_id` に渡すことで取得できる。
#[derive(Debug)]
pub struct ListAuditLogsOutput {
    pub logs: Vec<SecretAccessLog>,
    /// 次ページの先頭となるカーソル（最後のアイテムの id）。None なら最終ページ。
    pub next_cursor: Option<Uuid>,
}

/// `ListAuditLogsUseCase` は監査ログ一覧取得ユースケース。
pub struct ListAuditLogsUseCase {
    repo: Arc<dyn AccessLogRepository>,
}

impl ListAuditLogsUseCase {
    pub fn new(repo: Arc<dyn AccessLogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListAuditLogsInput) -> anyhow::Result<ListAuditLogsOutput> {
        let (logs, next_cursor) = self.repo.list(input.after_id, input.limit).await?;
        Ok(ListAuditLogsOutput { logs, next_cursor })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::access_log::AccessAction;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;

    #[tokio::test]
    async fn test_list_audit_logs_success() {
        let mut mock = MockAccessLogRepository::new();
        // LOW-12 監査対応: keyset シグネチャ (after_id, limit) に対応
        mock.expect_list().returning(|_, _| {
            Ok((
                vec![SecretAccessLog::new(
                    "app/db".to_string(),
                    AccessAction::Read,
                    Some("user-1".to_string()),
                    true,
                )],
                None,
            ))
        });

        let uc = ListAuditLogsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&ListAuditLogsInput {
                after_id: None,
                limit: 20,
            })
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.logs.len(), 1);
        assert!(output.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_list_audit_logs_empty() {
        let mut mock = MockAccessLogRepository::new();
        mock.expect_list().returning(|_, _| Ok((vec![], None)));

        let uc = ListAuditLogsUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&ListAuditLogsInput {
                after_id: None,
                limit: 20,
            })
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.logs.is_empty());
        assert!(output.next_cursor.is_none());
    }
}
