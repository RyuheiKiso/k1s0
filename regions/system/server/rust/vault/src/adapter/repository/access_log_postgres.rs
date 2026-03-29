use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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

fn action_from_str(s: &str) -> AccessAction {
    match s {
        "write" => AccessAction::Write,
        "delete" => AccessAction::Delete,
        "list" => AccessAction::List,
        _ => AccessAction::Read,
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

    /// LOW-12 監査対応: keyset ページネーションで OFFSET を廃止する。
    /// after_id が Some の場合、そのレコードの (created_at, id) より小さい行のみを取得する
    /// row value 比較を使用し、full table scan を回避する。
    /// after_id が None の場合は先頭ページを返す。
    async fn list(
        &self,
        after_id: Option<Uuid>,
        limit: u32,
    ) -> anyhow::Result<(Vec<SecretAccessLog>, Option<Uuid>)> {
        // limit+1 件取得して次ページの存在を確認し、カーソルを生成する
        let fetch_limit = limit as i64 + 1;
        let rows = if let Some(cursor_id) = after_id {
            // カーソルより古い（降順で後続の）レコードを keyset で取得する
            sqlx::query(
                "SELECT id, key_path, action, actor_id, ip_address, success, error_msg, created_at \
                 FROM vault.access_logs \
                 WHERE (created_at, id) < ( \
                     SELECT created_at, id FROM vault.access_logs WHERE id = $1 \
                 ) \
                 ORDER BY created_at DESC, id DESC \
                 LIMIT $2",
            )
            .bind(cursor_id)
            .bind(fetch_limit)
            .fetch_all(self.pool.as_ref())
            .await?
        } else {
            // 先頭ページ: created_at 降順で最新から取得する
            sqlx::query(
                "SELECT id, key_path, action, actor_id, ip_address, success, error_msg, created_at \
                 FROM vault.access_logs \
                 ORDER BY created_at DESC, id DESC \
                 LIMIT $1",
            )
            .bind(fetch_limit)
            .fetch_all(self.pool.as_ref())
            .await?
        };

        let mut logs: Vec<SecretAccessLog> = rows
            .into_iter()
            .map(|row| {
                let actor_id: String = row.get("actor_id");
                SecretAccessLog {
                    id: row.get("id"),
                    path: row.get("key_path"),
                    action: action_from_str(row.get("action")),
                    subject: if actor_id.is_empty() {
                        None
                    } else {
                        Some(actor_id)
                    },
                    tenant_id: None,
                    ip_address: row.get("ip_address"),
                    trace_id: None,
                    success: row.get("success"),
                    error_msg: row.get("error_msg"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        // limit+1 件取得できた場合は次ページが存在する: 末尾の 1 件を捨ててカーソルを設定する
        let next_cursor = if logs.len() > limit as usize {
            logs.pop();
            logs.last().map(|l| l.id)
        } else {
            None
        };

        Ok((logs, next_cursor))
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
