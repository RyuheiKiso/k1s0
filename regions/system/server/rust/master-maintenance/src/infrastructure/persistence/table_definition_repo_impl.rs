use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entity::table_definition::{TableDefinition, CreateTableDefinition, UpdateTableDefinition};
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;

pub struct TableDefinitionPostgresRepository {
    pool: PgPool,
}

impl TableDefinitionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TableDefinitionRepository for TableDefinitionPostgresRepository {
    async fn find_all(&self, category: Option<&str>, active_only: bool) -> anyhow::Result<Vec<TableDefinition>> {
        let mut query = String::from("SELECT * FROM master_maintenance.table_definitions WHERE 1=1");
        if active_only {
            query.push_str(" AND is_active = true");
        }
        if let Some(cat) = category {
            query.push_str(&format!(" AND category = '{}'", cat));
        }
        query.push_str(" ORDER BY sort_order, name");

        let rows = sqlx::query_as::<_, TableDefinitionRow>(&query)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<TableDefinition>> {
        let row = sqlx::query_as::<_, TableDefinitionRow>(
            "SELECT * FROM master_maintenance.table_definitions WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        let row = sqlx::query_as::<_, TableDefinitionRow>(
            "SELECT * FROM master_maintenance.table_definitions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, input: &CreateTableDefinition, created_by: &str) -> anyhow::Result<TableDefinition> {
        let row = sqlx::query_as::<_, TableDefinitionRow>(
            r#"INSERT INTO master_maintenance.table_definitions
               (name, schema_name, database_name, display_name, description, category,
                allow_create, allow_update, allow_delete, sort_order, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *"#
        )
        .bind(&input.name)
        .bind(&input.schema_name)
        .bind(input.database_name.as_deref().unwrap_or("default"))
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.allow_create.unwrap_or(true))
        .bind(input.allow_update.unwrap_or(true))
        .bind(input.allow_delete.unwrap_or(false))
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(&self, name: &str, input: &UpdateTableDefinition) -> anyhow::Result<TableDefinition> {
        // Dynamic update - only set fields that are Some
        let row = sqlx::query_as::<_, TableDefinitionRow>(
            r#"UPDATE master_maintenance.table_definitions SET
               display_name = COALESCE($2, display_name),
               description = COALESCE($3, description),
               category = COALESCE($4, category),
               is_active = COALESCE($5, is_active),
               allow_create = COALESCE($6, allow_create),
               allow_update = COALESCE($7, allow_update),
               allow_delete = COALESCE($8, allow_delete),
               sort_order = COALESCE($9, sort_order),
               updated_at = now()
               WHERE name = $1 RETURNING *"#
        )
        .bind(name)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.is_active)
        .bind(input.allow_create)
        .bind(input.allow_update)
        .bind(input.allow_delete)
        .bind(input.sort_order)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, name: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM master_maintenance.table_definitions WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TableDefinitionRow {
    id: Uuid,
    name: String,
    schema_name: String,
    database_name: String,
    display_name: String,
    description: Option<String>,
    category: Option<String>,
    is_active: bool,
    allow_create: bool,
    allow_update: bool,
    allow_delete: bool,
    sort_order: i32,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TableDefinitionRow> for TableDefinition {
    fn from(row: TableDefinitionRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            schema_name: row.schema_name,
            database_name: row.database_name,
            display_name: row.display_name,
            description: row.description,
            category: row.category,
            is_active: row.is_active,
            allow_create: row.allow_create,
            allow_update: row.allow_update,
            allow_delete: row.allow_delete,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
