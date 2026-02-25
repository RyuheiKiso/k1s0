use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::repository::AccessLogRepository;

/// AccessLogPostgresRepository は PostgreSQL を使った AccessLogRepository の実装。
pub struct AccessLogPostgresRepository {
    pool: Arc<PgPool>,
}

impl AccessLogPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl AccessAction {
    /// SQL 保存用の文字列表現を返す。
    fn as_str(&self) -> &'static str {
        match self {
            AccessAction::Read => "read",
            AccessAction::Write => "write",
            AccessAction::Delete => "delete",
            AccessAction::List => "list",
        }
    }
}

#[async_trait]
impl AccessLogRepository for AccessLogPostgresRepository {
    async fn record(&self, log: &SecretAccessLog) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO vault.access_logs \
             (id, key_path, action, actor_id, ip_address, success, error_msg, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(log.id)
        .bind(&log.path)
        .bind(log.action.as_str())
        .bind(log.subject.as_deref().unwrap_or(""))
        .bind(log.ip_address.as_deref())
        .bind(log.success)
        .bind(log.error_msg.as_deref())
        .bind(log.created_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_action_as_str() {
        assert_eq!(AccessAction::Read.as_str(), "read");
        assert_eq!(AccessAction::Write.as_str(), "write");
        assert_eq!(AccessAction::Delete.as_str(), "delete");
        assert_eq!(AccessAction::List.as_str(), "list");
    }

    #[test]
    fn test_access_log_fields() {
        let log = SecretAccessLog::new(
            "app/db/password".to_string(),
            AccessAction::Read,
            Some("user-1".to_string()),
            true,
        );

        assert_eq!(log.path, "app/db/password");
        assert_eq!(log.action.as_str(), "read");
        assert_eq!(log.subject, Some("user-1".to_string()));
        assert!(log.success);
    }
}
