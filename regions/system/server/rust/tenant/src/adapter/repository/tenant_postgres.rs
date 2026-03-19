use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::{Plan, Tenant, TenantStatus};
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
    owner_id: Option<String>,
    settings: serde_json::Value,
    keycloak_realm: Option<String>,
    db_schema: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn status_from_str(s: &str) -> TenantStatus {
    match s {
        "active" => TenantStatus::Active,
        "suspended" => TenantStatus::Suspended,
        "deleted" => TenantStatus::Deleted,
        _ => TenantStatus::Provisioning,
    }
}

fn plan_from_str(s: &str) -> anyhow::Result<Plan> {
    s.parse::<Plan>()
        .map_err(|e| anyhow::anyhow!("invalid tenant plan in database: {}", e))
}

impl TryFrom<TenantRow> for Tenant {
    type Error = anyhow::Error;

    fn try_from(r: TenantRow) -> Result<Self, Self::Error> {
        Ok(Tenant {
            id: r.id,
            name: r.name,
            display_name: r.display_name,
            status: status_from_str(&r.status),
            plan: plan_from_str(&r.plan)?,
            owner_id: r.owner_id,
            settings: r.settings,
            keycloak_realm: r.keycloak_realm,
            db_schema: r.db_schema,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[async_trait]
impl TenantRepository for TenantPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>> {
        let row: Option<TenantRow> = sqlx::query_as(
            "SELECT id, name, display_name, status, plan, owner_id, settings, keycloak_realm, db_schema, created_at, updated_at \
             FROM tenant.tenants WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>> {
        let row: Option<TenantRow> = sqlx::query_as(
            "SELECT id, name, display_name, status, plan, owner_id, settings, keycloak_realm, db_schema, created_at, updated_at \
             FROM tenant.tenants WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)> {
        let offset = ((page.max(1) - 1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<TenantRow> = sqlx::query_as(
            "SELECT id, name, display_name, status, plan, owner_id, settings, keycloak_realm, db_schema, created_at, updated_at \
             FROM tenant.tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenant.tenants")
            .fetch_one(self.pool.as_ref())
            .await?;

        let tenants = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<Tenant>>>()?;
        Ok((tenants, count.0))
    }

    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO tenant.tenants (id, name, display_name, status, plan, owner_id, settings, keycloak_realm, db_schema, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&tenant.display_name)
        .bind(tenant.status.as_str())
        .bind(tenant.plan.as_str())
        .bind(&tenant.owner_id)
        .bind(&tenant.settings)
        .bind(&tenant.keycloak_realm)
        .bind(&tenant.db_schema)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE tenant.tenants \
             SET display_name = $2, status = $3, plan = $4, owner_id = $5, settings = $6, keycloak_realm = $7, db_schema = $8 \
             WHERE id = $1",
        )
        .bind(tenant.id)
        .bind(&tenant.display_name)
        .bind(tenant.status.as_str())
        .bind(tenant.plan.as_str())
        .bind(&tenant.owner_id)
        .bind(&tenant.settings)
        .bind(&tenant.keycloak_realm)
        .bind(&tenant.db_schema)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            owner_id: Some(Uuid::new_v4().to_string()),
            settings: serde_json::json!({}),
            keycloak_realm: Some("acme".to_string()),
            db_schema: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let tenant: Tenant = row.try_into().unwrap();
        assert_eq!(tenant.name, "acme-corp");
        assert_eq!(tenant.status, TenantStatus::Active);
        assert_eq!(tenant.plan, Plan::Professional);
        assert_eq!(tenant.keycloak_realm, Some("acme".to_string()));
        assert_eq!(tenant.db_schema, None);
    }
}
