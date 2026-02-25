use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::quota::{Period, QuotaPolicy, SubjectType};
use crate::domain::repository::QuotaPolicyRepository;

pub struct QuotaPolicyPostgresRepository {
    pool: Arc<PgPool>,
}

impl QuotaPolicyPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct QuotaPolicyRow {
    id: uuid::Uuid,
    name: String,
    subject_type: String,
    subject_id: String,
    quota_limit: i64,
    period: String,
    enabled: bool,
    alert_threshold_percent: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<QuotaPolicyRow> for QuotaPolicy {
    fn from(r: QuotaPolicyRow) -> Self {
        QuotaPolicy {
            id: r.id.to_string(),
            name: r.name,
            subject_type: SubjectType::from_str(&r.subject_type)
                .unwrap_or(SubjectType::Tenant),
            subject_id: r.subject_id,
            limit: r.quota_limit as u64,
            period: Period::from_str(&r.period).unwrap_or(Period::Daily),
            enabled: r.enabled,
            alert_threshold_percent: Some(r.alert_threshold_percent as u8),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl QuotaPolicyRepository for QuotaPolicyPostgresRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let row: Option<QuotaPolicyRow> = sqlx::query_as(
            "SELECT id, name, subject_type, subject_id, quota_limit, period, \
                    enabled, alert_threshold_percent, created_at, updated_at \
             FROM quota.quota_policies WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_all(
        &self,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<QuotaPolicy>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<QuotaPolicyRow> = sqlx::query_as(
            "SELECT id, name, subject_type, subject_id, quota_limit, period, \
                    enabled, alert_threshold_percent, created_at, updated_at \
             FROM quota.quota_policies ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM quota.quota_policies")
                .fetch_one(self.pool.as_ref())
                .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&policy.id)
            .or_else(|_| {
                // id が "quota_xxx" 形式の場合は新しい UUID を生成
                Ok::<uuid::Uuid, uuid::Error>(uuid::Uuid::new_v4())
            })?;

        sqlx::query(
            "INSERT INTO quota.quota_policies \
             (id, name, subject_type, subject_id, quota_limit, period, \
              enabled, alert_threshold_percent, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(uuid)
        .bind(&policy.name)
        .bind(policy.subject_type.as_str())
        .bind(&policy.subject_id)
        .bind(policy.limit as i64)
        .bind(policy.period.as_str())
        .bind(policy.enabled)
        .bind(policy.alert_threshold_percent.unwrap_or(80) as f64)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let uuid = uuid::Uuid::parse_str(&policy.id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        sqlx::query(
            "UPDATE quota.quota_policies \
             SET name = $2, subject_type = $3, subject_id = $4, quota_limit = $5, \
                 period = $6, enabled = $7, alert_threshold_percent = $8, updated_at = $9 \
             WHERE id = $1",
        )
        .bind(uuid)
        .bind(&policy.name)
        .bind(policy.subject_type.as_str())
        .bind(&policy.subject_id)
        .bind(policy.limit as i64)
        .bind(policy.period.as_str())
        .bind(policy.enabled)
        .bind(policy.alert_threshold_percent.unwrap_or(80) as f64)
        .bind(policy.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid UUID: {}", e))?;

        let result = sqlx::query("DELETE FROM quota.quota_policies WHERE id = $1")
            .bind(uuid)
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
