use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

/// PostgreSQL 実装の PolicyRepository。
pub struct PolicyPostgresRepository {
    pool: Arc<PgPool>,
}

impl PolicyPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    name: String,
    description: String,
    rego_content: String,
    package_path: String,
    bundle_id: Option<String>,
    enabled: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PolicyRow> for Policy {
    fn from(r: PolicyRow) -> Self {
        Policy {
            id: r.id,
            name: r.name,
            description: r.description,
            rego_content: r.rego_content,
            package_path: r.package_path,
            bundle_id: r.bundle_id,
            version: r.version as u32,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl PolicyRepository for PolicyPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>> {
        let row: Option<PolicyRow> = sqlx::query_as(
            "SELECT id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at \
             FROM policy.policies WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        let rows: Vec<PolicyRow> = sqlx::query_as(
            "SELECT id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at \
             FROM policy.policies ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO policy.policies \
             (id, name, description, rego_content, package_path, bundle_id, enabled, version, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(policy.id)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(&policy.rego_content)
        .bind(&policy.package_path)
        .bind(&policy.bundle_id)
        .bind(policy.enabled)
        .bind(policy.version as i32)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE policy.policies \
             SET description = $2, rego_content = $3, package_path = $4, bundle_id = $5, enabled = $6, version = $7, updated_at = $8 \
             WHERE id = $1",
        )
        .bind(policy.id)
        .bind(&policy.description)
        .bind(&policy.rego_content)
        .bind(&policy.package_path)
        .bind(&policy.bundle_id)
        .bind(policy.enabled)
        .bind(policy.version as i32)
        .bind(policy.updated_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM policy.policies WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let row: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM policy.policies WHERE name = $1)")
                .bind(name)
                .fetch_one(self.pool.as_ref())
                .await?;
        Ok(row.0)
    }
}
