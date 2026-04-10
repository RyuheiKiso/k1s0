use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::quota::{Period, QuotaPolicy, SubjectType};
use crate::domain::repository::QuotaPolicyRepository;

/// `QuotaPolicyPostgresRepository` は `QuotaPolicyRepository` の `PostgreSQL` 実装。
/// CRITICAL-RUST-001 監査対応: 全 DB 操作前に RLS テナント分離のための
/// `set_config`('`app.current_tenant_id`', ...) を呼び出す。
pub struct QuotaPolicyPostgresRepository {
    pool: Arc<PgPool>,
}

impl QuotaPolicyPostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct QuotaPolicyRow {
    id: String,
    name: String,
    subject_type: String,
    subject_id: String,
    quota_limit: i64,
    period: String,
    enabled: bool,
    alert_threshold_percent: i16,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tenant_id: String,
}

impl From<QuotaPolicyRow> for QuotaPolicy {
    fn from(r: QuotaPolicyRow) -> Self {
        QuotaPolicy {
            id: r.id,
            name: r.name,
            subject_type: SubjectType::from_str(&r.subject_type).unwrap_or(SubjectType::Tenant),
            subject_id: r.subject_id,
            // LOW-008: 安全な型変換（quota_limit は非負であることが前提）
            limit: u64::try_from(r.quota_limit).unwrap_or(0),
            period: Period::from_str(&r.period).unwrap_or(Period::Daily),
            enabled: r.enabled,
            // LOW-008: 安全な型変換（alert_threshold_percent は 0-100 の範囲で u8 に収まる）
            alert_threshold_percent: Some(u8::try_from(r.alert_threshold_percent).unwrap_or(0)),
            created_at: r.created_at,
            updated_at: r.updated_at,
            tenant_id: r.tenant_id,
        }
    }
}

#[async_trait]
impl QuotaPolicyRepository for QuotaPolicyPostgresRepository {
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        // CRITICAL-RUST-001 監査対応: SELECT 前に RLS テナント分離のためセッション変数を設定する。
        // FORCE ROW LEVEL SECURITY が有効なため set_config を省略すると全行がブロックされる。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        let row: Option<QuotaPolicyRow> = sqlx::query_as(
            "SELECT id, name, subject_type, subject_id, quota_limit, period, \
                    enabled, alert_threshold_percent, created_at, updated_at, tenant_id \
             FROM quota.quota_policies WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_all(
        &self,
        page: u32,
        page_size: u32,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<QuotaPolicy>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        // CRITICAL-RUST-001 監査対応: SELECT 前に RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        let rows: Vec<QuotaPolicyRow> = sqlx::query_as(
            "SELECT id, name, subject_type, subject_id, quota_limit, period, \
                    enabled, alert_threshold_percent, created_at, updated_at, tenant_id \
             FROM quota.quota_policies ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota.quota_policies")
            .fetch_one(self.pool.as_ref())
            .await?;

        // LOW-008: 安全な型変換（COUNT(*) は非負であることが前提）
        Ok((rows.into_iter().map(Into::into).collect(), u64::try_from(count.0).unwrap_or(0)))
    }

    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        // set_config の第3引数 true は SET LOCAL（トランザクションスコープのみ有効）を意味する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&policy.tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        // tenant_id を $2 にバインドし、残りのフィールドを続ける
        sqlx::query(
            "INSERT INTO quota.quota_policies \
             (id, tenant_id, name, subject_type, subject_id, quota_limit, period, \
              enabled, alert_threshold_percent, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(&policy.id)
        .bind(&policy.tenant_id)
        .bind(&policy.name)
        .bind(policy.subject_type.as_str())
        .bind(&policy.subject_id)
        // LOW-008: 安全な型変換（quota limit は i64 範囲内が前提）
        .bind(i64::try_from(policy.limit).unwrap_or(i64::MAX))
        .bind(policy.period.as_str())
        .bind(policy.enabled)
        .bind(i16::from(policy.alert_threshold_percent.unwrap_or(80)))
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&policy.tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        sqlx::query(
            "UPDATE quota.quota_policies \
             SET name = $2, subject_type = $3, subject_id = $4, quota_limit = $5, \
                 period = $6, enabled = $7, alert_threshold_percent = $8, updated_at = $9 \
             WHERE id = $1",
        )
        .bind(&policy.id)
        .bind(&policy.name)
        .bind(policy.subject_type.as_str())
        .bind(&policy.subject_id)
        // LOW-008: 安全な型変換（quota limit は i64 範囲内が前提）
        .bind(i64::try_from(policy.limit).unwrap_or(i64::MAX))
        .bind(policy.period.as_str())
        .bind(policy.enabled)
        .bind(i16::from(policy.alert_threshold_percent.unwrap_or(80)))
        .bind(policy.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool> {
        // CRITICAL-RUST-001 監査対応: DELETE 前に RLS テナント分離のためセッション変数を設定する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        let result = sqlx::query("DELETE FROM quota.quota_policies WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
