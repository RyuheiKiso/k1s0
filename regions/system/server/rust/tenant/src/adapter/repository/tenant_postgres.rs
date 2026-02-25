use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::{Tenant, TenantStatus};
use crate::domain::repository::TenantRepository;

pub struct TenantPostgresRepository {
    pool: Arc<PgPool>,
}

impl TenantPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TenantRow {
    id: Uuid,
    name: String,
    display_name: String,
    status: String,
    plan: String,
    created_at: DateTime<Utc>,
}

fn status_from_str(s: &str) -> TenantStatus {
    match s {
        "active" => TenantStatus::Active,
        "suspended" => TenantStatus::Suspended,
        "deleted" => TenantStatus::Deleted,
        _ => TenantStatus::Provisioning,
    }
}

impl From<TenantRow> for Tenant {
    fn from(r: TenantRow) -> Self {
        Tenant {
            id: r.id,
            name: r.name,
            display_name: r.display_name,
            status: status_from_str(&r.status),
            plan: r.plan,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl TenantRepository for TenantPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>> {
        let row: Option<TenantRow> = sqlx::query_as(
            "SELECT id, name, display_name, status, plan, created_at \
             FROM tenant.tenants WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>> {
        let row: Option<TenantRow> = sqlx::query_as(
            "SELECT id, name, display_name, status, plan, created_at \
             FROM tenant.tenants WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)> {
        let offset = ((page.max(1) - 1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<TenantRow> = sqlx::query_as(
            "SELECT id, name, display_name, status, plan, created_at \
             FROM tenant.tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenant.tenants")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0))
    }

    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO tenant.tenants (id, name, display_name, status, plan, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&tenant.display_name)
        .bind(tenant.status.as_str())
        .bind(&tenant.plan)
        .bind(tenant.created_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE tenant.tenants \
             SET display_name = $2, status = $3, plan = $4 \
             WHERE id = $1",
        )
        .bind(tenant.id)
        .bind(&tenant.display_name)
        .bind(tenant.status.as_str())
        .bind(&tenant.plan)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_str_provisioning() {
        assert_eq!(status_from_str("provisioning"), TenantStatus::Provisioning);
    }

    #[test]
    fn test_status_from_str_active() {
        assert_eq!(status_from_str("active"), TenantStatus::Active);
    }

    #[test]
    fn test_status_from_str_suspended() {
        assert_eq!(status_from_str("suspended"), TenantStatus::Suspended);
    }

    #[test]
    fn test_status_from_str_deleted() {
        assert_eq!(status_from_str("deleted"), TenantStatus::Deleted);
    }

    #[test]
    fn test_status_from_str_unknown_defaults_provisioning() {
        assert_eq!(status_from_str("unknown"), TenantStatus::Provisioning);
    }

    #[test]
    fn test_tenant_row_to_entity() {
        let row = TenantRow {
            id: Uuid::new_v4(),
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            status: "active".to_string(),
            plan: "professional".to_string(),
            created_at: Utc::now(),
        };
        let tenant: Tenant = row.into();
        assert_eq!(tenant.name, "acme-corp");
        assert_eq!(tenant.status, TenantStatus::Active);
        assert_eq!(tenant.plan, "professional");
    }
}
