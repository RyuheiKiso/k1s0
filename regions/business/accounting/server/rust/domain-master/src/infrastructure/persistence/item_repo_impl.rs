use crate::domain::entity::master_item::{CreateMasterItem, MasterItem, UpdateMasterItem};
use crate::domain::repository::item_repository::ItemRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ItemPostgresRepository {
    pool: PgPool,
}

impl ItemPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ItemRepository for ItemPostgresRepository {
    async fn find_by_category(
        &self,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        let rows = if active_only {
            sqlx::query_as::<_, ItemRow>(
                "SELECT * FROM domain_master.master_items WHERE category_id = $1 AND is_active = true ORDER BY sort_order, code",
            )
            .bind(category_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ItemRow>(
                "SELECT * FROM domain_master.master_items WHERE category_id = $1 ORDER BY sort_order, code",
            )
            .bind(category_id)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_category_and_code(
        &self,
        category_id: Uuid,
        code: &str,
    ) -> anyhow::Result<Option<MasterItem>> {
        let row = sqlx::query_as::<_, ItemRow>(
            "SELECT * FROM domain_master.master_items WHERE category_id = $1 AND code = $2",
        )
        .bind(category_id)
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterItem>> {
        let row = sqlx::query_as::<_, ItemRow>(
            "SELECT * FROM domain_master.master_items WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create(
        &self,
        category_id: Uuid,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        let row = sqlx::query_as::<_, ItemRow>(
            r#"INSERT INTO domain_master.master_items
               (category_id, code, display_name, description, attributes,
                parent_item_id, effective_from, effective_until, sort_order, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#,
        )
        .bind(category_id)
        .bind(&input.code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.attributes)
        .bind(input.parent_item_id)
        .bind(input.effective_from)
        .bind(input.effective_until)
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(
        &self,
        id: Uuid,
        input: &UpdateMasterItem,
    ) -> anyhow::Result<MasterItem> {
        let row = sqlx::query_as::<_, ItemRow>(
            r#"UPDATE domain_master.master_items SET
               display_name = COALESCE($2, display_name),
               description = COALESCE($3, description),
               attributes = COALESCE($4, attributes),
               parent_item_id = COALESCE($5, parent_item_id),
               effective_from = COALESCE($6, effective_from),
               effective_until = COALESCE($7, effective_until),
               is_active = COALESCE($8, is_active),
               sort_order = COALESCE($9, sort_order),
               updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.attributes)
        .bind(input.parent_item_id)
        .bind(input.effective_from)
        .bind(input.effective_until)
        .bind(input.is_active)
        .bind(input.sort_order)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM domain_master.master_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    category_id: Uuid,
    code: String,
    display_name: String,
    description: Option<String>,
    attributes: Option<serde_json::Value>,
    parent_item_id: Option<Uuid>,
    effective_from: Option<chrono::DateTime<chrono::Utc>>,
    effective_until: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
    sort_order: i32,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ItemRow> for MasterItem {
    fn from(row: ItemRow) -> Self {
        Self {
            id: row.id,
            category_id: row.category_id,
            code: row.code,
            display_name: row.display_name,
            description: row.description,
            attributes: row.attributes,
            parent_item_id: row.parent_item_id,
            effective_from: row.effective_from,
            effective_until: row.effective_until,
            is_active: row.is_active,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
