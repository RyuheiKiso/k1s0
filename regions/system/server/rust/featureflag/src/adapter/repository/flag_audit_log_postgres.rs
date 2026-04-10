use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::flag_audit_log::FlagAuditLog;
use crate::domain::repository::FlagAuditLogRepository;

pub struct FlagAuditLogPostgresRepository {
    pool: Arc<PgPool>,
}

impl FlagAuditLogPostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// `PostgreSQL` の行をマッピングするための内部構造体。
/// STATIC-CRITICAL-001 監査対応: `tenant_id` カラムを含む。
/// HIGH-005 対応: migration 006 で `tenant_id` が TEXT 型に変更されたため String 型を使用する。
#[derive(sqlx::FromRow)]
struct FlagAuditLogRow {
    id: Uuid,
    tenant_id: String,
    flag_id: Uuid,
    flag_key: String,
    action: String,
    before_json: Option<serde_json::Value>,
    after_json: Option<serde_json::Value>,
    changed_by: String,
    created_at: DateTime<Utc>,
}

impl From<FlagAuditLogRow> for FlagAuditLog {
    fn from(row: FlagAuditLogRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            flag_id: row.flag_id,
            flag_key: row.flag_key,
            action: row.action,
            before_json: row.before_json,
            after_json: row.after_json,
            changed_by: row.changed_by,
            trace_id: None,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl FlagAuditLogRepository for FlagAuditLogPostgresRepository {
    /// STATIC-CRITICAL-001 監査対応: `tenant_id` を含む監査ログを記録する。
    async fn create(&self, log: &FlagAuditLog) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO featureflag.flag_audit_logs \
             (id, tenant_id, flag_id, flag_key, action, before_json, after_json, changed_by, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(log.id)
        .bind(&log.tenant_id)
        .bind(log.flag_id)
        .bind(&log.flag_key)
        .bind(&log.action)
        .bind(&log.before_json)
        .bind(&log.after_json)
        .bind(&log.changed_by)
        .bind(log.created_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn list_by_flag_id(
        &self,
        flag_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<FlagAuditLog>> {
        let rows: Vec<FlagAuditLogRow> = sqlx::query_as(
            "SELECT id, tenant_id, flag_id, flag_key, action, before_json, after_json, changed_by, created_at \
             FROM featureflag.flag_audit_logs \
             WHERE flag_id = $1 \
             ORDER BY created_at DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(flag_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
