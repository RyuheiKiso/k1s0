use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::entity::access_log::{AccessAction, SecretAccessLog};
use crate::domain::repository::AccessLogRepository;

/// `AccessLogPostgresRepository` は `PostgreSQL` を使った `AccessLogRepository` の実装。
pub struct AccessLogPostgresRepository {
    pool: Arc<PgPool>,
}

impl AccessLogPostgresRepository {
    #[must_use]
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

/// ADR-0109 対応: `key_path` の先頭セグメントからテナント ID を抽出する。
/// `key_path` は "{`tenant_id`}/..." の形式であること。
fn extract_tenant_id_from_path(path: &str) -> &str {
    path.split('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("system")
}

#[async_trait]
impl AccessLogRepository for AccessLogPostgresRepository {
    async fn record(&self, log: &SecretAccessLog) -> anyhow::Result<()> {
        // ADR-0109: tenant_id が明示指定されていればそれを使用し、
        // なければ key_path の先頭セグメントから抽出する
        let tenant_id = log
            .tenant_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| extract_tenant_id_from_path(&log.path));

        // CRITICAL-DB-001 / CRITICAL-RUST-001 監査対応: vault.access_logs は RLS FORCE が有効（migration 007）。
        // INSERT 前にセッション変数を設定してテナント分離ポリシーを満たす。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        // migration 007 で tenant_id NOT NULL DEFAULT 削除済み: INSERT に明示指定が必要
        sqlx::query(
            "INSERT INTO vault.access_logs \
             (id, tenant_id, key_path, action, actor_id, ip_address, success, error_msg, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(log.id)
        .bind(tenant_id)
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
    /// CRITICAL-DB-001 監査対応: `vault.access_logs` は RLS FORCE が有効（migration 007）。
    /// 監査ログ一覧は管理・運用目的で全テナントを横断するため
    /// `vault.list_access_logs_all_tenants（SECURITY` DEFINER 関数、migration 008）を使用する。
    async fn list(
        &self,
        after_id: Option<Uuid>,
        limit: u32,
    ) -> anyhow::Result<(Vec<SecretAccessLog>, Option<Uuid>)> {
        // limit+1 件取得して次ページの存在を確認し、カーソルを生成する
        let fetch_limit = i64::from(limit) + 1;

        // vault.list_access_logs_all_tenants は SECURITY DEFINER 関数（migration 008）。
        // FORCE ROW LEVEL SECURITY を持つ access_logs テーブルに対して
        // 全テナント横断の管理クエリを関数オーナー（DB オーナー）権限で実行する。
        let rows = sqlx::query(
            "SELECT id, key_path, action, actor_id, ip_address, success, error_msg, created_at \
             FROM vault.list_access_logs_all_tenants($1, $2)",
        )
        .bind(after_id)
        .bind(fetch_limit)
        .fetch_all(self.pool.as_ref())
        .await?;

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

    #[test]
    fn test_extract_tenant_id_from_path() {
        // ADR-0109 パス形式からのテナント ID 抽出を検証する
        assert_eq!(extract_tenant_id_from_path("tenant-1/app/db"), "tenant-1");
        assert_eq!(extract_tenant_id_from_path("system/config"), "system");
        // 空パスのフォールバックを検証する
        assert_eq!(extract_tenant_id_from_path(""), "system");
    }
}
