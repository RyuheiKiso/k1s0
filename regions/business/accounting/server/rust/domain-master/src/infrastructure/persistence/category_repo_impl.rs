use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use crate::domain::repository::category_repository::CategoryRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CategoryPostgresRepository {
    pool: PgPool,
}

impl CategoryPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for CategoryPostgresRepository {
    async fn find_all(&self, active_only: bool) -> anyhow::Result<Vec<MasterCategory>> {
        let rows = if active_only {
            sqlx::query_as::<_, CategoryRow>(
                "SELECT * FROM domain_master.master_categories WHERE is_active = true ORDER BY sort_order, code",
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, CategoryRow>(
                "SELECT * FROM domain_master.master_categories ORDER BY sort_order, code",
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_code(&self, code: &str) -> anyhow::Result<Option<MasterCategory>> {
        let row = sqlx::query_as::<_, CategoryRow>(
            "SELECT * FROM domain_master.master_categories WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterCategory>> {
        let row = sqlx::query_as::<_, CategoryRow>(
            "SELECT * FROM domain_master.master_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create(
        &self,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"INSERT INTO domain_master.master_categories
               (code, display_name, description, validation_schema, is_active, sort_order, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(&input.code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.validation_schema)
        .bind(input.is_active.unwrap_or(true))
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(
        &self,
        code: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"UPDATE domain_master.master_categories SET
               display_name = COALESCE($2, display_name),
               description = COALESCE($3, description),
               validation_schema = COALESCE($4, validation_schema),
               is_active = COALESCE($5, is_active),
               sort_order = COALESCE($6, sort_order),
               updated_at = now()
               WHERE code = $1 RETURNING *"#,
        )
        .bind(code)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.validation_schema)
        .bind(input.is_active)
        .bind(input.sort_order)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, code: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM domain_master.master_categories WHERE code = $1")
            .bind(code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: Uuid,
    code: String,
    display_name: String,
    description: Option<String>,
    validation_schema: Option<serde_json::Value>,
    is_active: bool,
    sort_order: i32,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CategoryRow> for MasterCategory {
    fn from(row: CategoryRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            display_name: row.display_name,
            description: row.description,
            validation_schema: row.validation_schema,
            is_active: row.is_active,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
