use crate::domain::entity::tenant_master_extension::{
    TenantMasterExtension, UpsertTenantMasterExtension,
};
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TenantExtensionPostgresRepository {
    pool: PgPool,
}

impl TenantExtensionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantExtensionRepository for TenantExtensionPostgresRepository {
    async fn find_by_tenant_and_item(
        &self,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<Option<TenantMasterExtension>> {
        let row = sqlx::query_as::<_, TenantExtensionRow>(
            "SELECT * FROM domain_master.tenant_master_extensions WHERE tenant_id = $1 AND item_id = $2",
        )
        .bind(tenant_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_tenant_and_category(
        &self,
        tenant_id: &str,
        category_id: Uuid,
    ) -> anyhow::Result<Vec<TenantMasterExtension>> {
        let rows = sqlx::query_as::<_, TenantExtensionRow>(
            r#"SELECT te.* FROM domain_master.tenant_master_extensions te
               INNER JOIN domain_master.master_items mi ON te.item_id = mi.id
               WHERE te.tenant_id = $1 AND mi.category_id = $2"#,
        )
        .bind(tenant_id)
        .bind(category_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn upsert(
        &self,
        tenant_id: &str,
        item_id: Uuid,
        input: &UpsertTenantMasterExtension,
    ) -> anyhow::Result<TenantMasterExtension> {
        let row = sqlx::query_as::<_, TenantExtensionRow>(
            r#"INSERT INTO domain_master.tenant_master_extensions
               (tenant_id, item_id, display_name_override, attributes_override, is_enabled)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (tenant_id, item_id) DO UPDATE SET
               display_name_override = COALESCE($3, domain_master.tenant_master_extensions.display_name_override),
               attributes_override = COALESCE($4, domain_master.tenant_master_extensions.attributes_override),
               is_enabled = COALESCE($5, domain_master.tenant_master_extensions.is_enabled),
               updated_at = now()
               RETURNING *"#,
        )
        .bind(tenant_id)
        .bind(item_id)
        .bind(&input.display_name_override)
        .bind(&input.attributes_override)
        .bind(input.is_enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, tenant_id: &str, item_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "DELETE FROM domain_master.tenant_master_extensions WHERE tenant_id = $1 AND item_id = $2",
        )
        .bind(tenant_id)
        .bind(item_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TenantExtensionRow {
    id: Uuid,
    tenant_id: String,
    item_id: Uuid,
    display_name_override: Option<String>,
    attributes_override: Option<serde_json::Value>,
    is_enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TenantExtensionRow> for TenantMasterExtension {
    fn from(row: TenantExtensionRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            item_id: row.item_id,
            display_name_override: row.display_name_override,
            attributes_override: row.attributes_override,
            is_enabled: row.is_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
