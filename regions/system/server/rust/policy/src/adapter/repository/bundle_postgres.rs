use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::domain::repository::PolicyBundleRepository;

/// `PostgreSQL` 実装の `PolicyBundleRepository`。
pub struct BundlePostgresRepository {
    pool: Arc<PgPool>,
}

impl BundlePostgresRepository {
    #[must_use] 
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
    tenant_id: String,
}

impl TryFrom<BundleRow> for PolicyBundle {
    type Error = anyhow::Error;

    fn try_from(r: BundleRow) -> anyhow::Result<Self> {
        let policy_ids: Vec<Uuid> = serde_json::from_value(r.policies)?;
        Ok(PolicyBundle {
            id: r.id,
            name: r.name,
            description: None,
            enabled: true,
            policy_ids,
            created_at: r.created_at,
            // policy_bundles テーブルに updated_at がないため created_at で代用
            updated_at: r.created_at,
            tenant_id: r.tenant_id,
        })
    }
}

#[async_trait]
impl PolicyBundleRepository for BundlePostgresRepository {
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<Option<PolicyBundle>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<BundleRow> = sqlx::query_as(
            "SELECT id, name, policies, created_at, tenant_id \
             FROM policy.policy_bundles WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<PolicyBundle>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows: Vec<BundleRow> = sqlx::query_as(
            "SELECT id, name, policies, created_at, tenant_id \
             FROM policy.policy_bundles ORDER BY created_at DESC",
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから INSERT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&bundle.tenant_id)
            .execute(&mut *tx)
            .await?;

        let policies_json = serde_json::to_value(&bundle.policy_ids)?;
        sqlx::query(
            "INSERT INTO policy.policy_bundles (id, name, policies, created_at, tenant_id) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(bundle.id)
        .bind(&bundle.name)
        .bind(&policies_json)
        .bind(bundle.created_at)
        .bind(&bundle.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから DELETE する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM policy.policy_bundles WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
