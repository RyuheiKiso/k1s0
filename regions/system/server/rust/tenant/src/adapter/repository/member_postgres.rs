use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::{ProvisioningJob, ProvisioningStatus, TenantMember};
use crate::domain::repository::MemberRepository;

pub struct MemberPostgresRepository {
    pool: Arc<PgPool>,
}

impl MemberPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    id: Uuid,
    tenant_id: Uuid,
    user_id: Uuid,
    role: String,
    joined_at: DateTime<Utc>,
}

impl From<MemberRow> for TenantMember {
    fn from(r: MemberRow) -> Self {
        TenantMember {
            id: r.id,
            tenant_id: r.tenant_id,
            user_id: r.user_id,
            role: r.role,
            joined_at: r.joined_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProvisioningJobRow {
    id: Uuid,
    tenant_id: Uuid,
    status: String,
    current_step: Option<String>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn provisioning_status_from_str(s: &str) -> ProvisioningStatus {
    match s {
        "running" => ProvisioningStatus::Running,
        "completed" => ProvisioningStatus::Completed,
        "failed" => ProvisioningStatus::Failed,
        _ => ProvisioningStatus::Pending,
    }
}

impl From<ProvisioningJobRow> for ProvisioningJob {
    fn from(r: ProvisioningJobRow) -> Self {
        ProvisioningJob {
            id: r.id,
            tenant_id: r.tenant_id,
            status: provisioning_status_from_str(&r.status),
            current_step: r.current_step,
            error_message: r.error_message,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl MemberRepository for MemberPostgresRepository {
    async fn find_by_tenant(&self, tenant_id: &Uuid) -> anyhow::Result<Vec<TenantMember>> {
        let rows: Vec<MemberRow> = sqlx::query_as(
            "SELECT id, tenant_id, user_id, role, joined_at \
             FROM tenant.tenant_members WHERE tenant_id = $1 ORDER BY joined_at ASC",
        )
        .bind(tenant_id)
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_member(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Option<TenantMember>> {
        let row: Option<MemberRow> = sqlx::query_as(
            "SELECT id, tenant_id, user_id, role, joined_at \
             FROM tenant.tenant_members WHERE tenant_id = $1 AND user_id = $2",
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn add(&self, member: &TenantMember) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO tenant.tenant_members (id, tenant_id, user_id, role, joined_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(member.id)
        .bind(member.tenant_id)
        .bind(member.user_id)
        .bind(&member.role)
        .bind(member.joined_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn remove(&self, tenant_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM tenant.tenant_members WHERE tenant_id = $1 AND user_id = $2",
        )
        .bind(tenant_id)
        .bind(user_id)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_job(&self, _job_id: &Uuid) -> anyhow::Result<Option<ProvisioningJob>> {
        // Provisioning jobs テーブルは将来のマイグレーションで作成予定。
        // 現時点では None を返す。
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provisioning_status_from_str_pending() {
        assert_eq!(
            provisioning_status_from_str("pending"),
            ProvisioningStatus::Pending
        );
    }

    #[test]
    fn test_provisioning_status_from_str_running() {
        assert_eq!(
            provisioning_status_from_str("running"),
            ProvisioningStatus::Running
        );
    }

    #[test]
    fn test_provisioning_status_from_str_completed() {
        assert_eq!(
            provisioning_status_from_str("completed"),
            ProvisioningStatus::Completed
        );
    }

    #[test]
    fn test_provisioning_status_from_str_failed() {
        assert_eq!(
            provisioning_status_from_str("failed"),
            ProvisioningStatus::Failed
        );
    }

    #[test]
    fn test_provisioning_status_from_str_unknown_defaults_pending() {
        assert_eq!(
            provisioning_status_from_str("unknown"),
            ProvisioningStatus::Pending
        );
    }

    #[test]
    fn test_member_row_to_entity() {
        let row = MemberRow {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: "admin".to_string(),
            joined_at: Utc::now(),
        };
        let member: TenantMember = row.into();
        assert_eq!(member.role, "admin");
    }
}
