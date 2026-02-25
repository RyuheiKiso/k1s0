use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::domain::repository::PolicyBundleRepository;

/// PostgreSQL 実装の PolicyBundleRepository。
pub struct BundlePostgresRepository {
    pool: Arc<PgPool>,
}

impl BundlePostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BundleRow {
    id: Uuid,
    name: String,
    policies: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl TryFrom<BundleRow> for PolicyBundle {
    type Error = anyhow::Error;

    fn try_from(r: BundleRow) -> anyhow::Result<Self> {
        let policy_ids: Vec<Uuid> = serde_json::from_value(r.policies)?;
        Ok(PolicyBundle {
            id: r.id,
            name: r.name,
            policy_ids,
            created_at: r.created_at,
            // policy_bundles テーブルに updated_at がないため created_at で代用
            updated_at: r.created_at,
        })
    }
}

#[async_trait]
impl PolicyBundleRepository for BundlePostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<PolicyBundle>> {
        let row: Option<BundleRow> = sqlx::query_as(
            "SELECT id, name, policies, created_at \
             FROM policy.policy_bundles WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;
        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>> {
        let rows: Vec<BundleRow> = sqlx::query_as(
            "SELECT id, name, policies, created_at \
             FROM policy.policy_bundles ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()> {
        let policies_json = serde_json::to_value(&bundle.policy_ids)?;
        sqlx::query(
            "INSERT INTO policy.policy_bundles (id, name, policies, created_at) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(bundle.id)
        .bind(&bundle.name)
        .bind(&policies_json)
        .bind(bundle.created_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM policy.policy_bundles WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
