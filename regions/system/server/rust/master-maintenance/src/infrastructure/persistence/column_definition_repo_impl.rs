use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entity::column_definition::{ColumnDefinition, CreateColumnDefinition};
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;

pub struct ColumnDefinitionPostgresRepository {
    pool: PgPool,
}

impl ColumnDefinitionPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ColumnDefinitionRepository for ColumnDefinitionPostgresRepository {
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<ColumnDefinition>> {
        let rows = sqlx::query_as::<_, ColumnDefinitionRow>(
            "SELECT * FROM master_maintenance.column_definitions WHERE table_id = $1 ORDER BY display_order, column_name"
        )
        .bind(table_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_table_and_column(&self, table_id: Uuid, column_name: &str) -> anyhow::Result<Option<ColumnDefinition>> {
        let row = sqlx::query_as::<_, ColumnDefinitionRow>(
            "SELECT * FROM master_maintenance.column_definitions WHERE table_id = $1 AND column_name = $2"
        )
        .bind(table_id)
        .bind(column_name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create_batch(&self, table_id: Uuid, columns: &[CreateColumnDefinition]) -> anyhow::Result<Vec<ColumnDefinition>> {
        let mut tx = self.pool.begin().await?;
        let mut results = Vec::with_capacity(columns.len());

        for (i, col) in columns.iter().enumerate() {
            let row = sqlx::query_as::<_, ColumnDefinitionRow>(
                r#"INSERT INTO master_maintenance.column_definitions
                   (table_id, column_name, display_name, data_type, is_primary_key, is_nullable,
                    is_unique, default_value, max_length, min_value, max_value, regex_pattern,
                    display_order, is_searchable, is_sortable, is_filterable,
                    is_visible_in_list, is_visible_in_form, is_readonly, input_type, select_options)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
                   RETURNING *"#
            )
            .bind(table_id)
            .bind(&col.column_name)
            .bind(&col.display_name)
            .bind(&col.data_type)
            .bind(col.is_primary_key.unwrap_or(false))
            .bind(col.is_nullable.unwrap_or(true))
            .bind(col.is_unique.unwrap_or(false))
            .bind(&col.default_value)
            .bind(col.max_length)
            .bind(col.min_value)
            .bind(col.max_value)
            .bind(&col.regex_pattern)
            .bind(col.display_order.unwrap_or(i as i32))
            .bind(col.is_searchable.unwrap_or(false))
            .bind(col.is_sortable.unwrap_or(true))
            .bind(col.is_filterable.unwrap_or(false))
            .bind(col.is_visible_in_list.unwrap_or(true))
            .bind(col.is_visible_in_form.unwrap_or(true))
            .bind(col.is_readonly.unwrap_or(false))
            .bind(col.input_type.as_deref().unwrap_or("text"))
            .bind(&col.select_options)
            .fetch_one(&mut *tx)
            .await?;
            results.push(row.into());
        }

        tx.commit().await?;
        Ok(results)
    }

    async fn update(&self, table_id: Uuid, column_name: &str, input: &CreateColumnDefinition) -> anyhow::Result<ColumnDefinition> {
        let row = sqlx::query_as::<_, ColumnDefinitionRow>(
            r#"UPDATE master_maintenance.column_definitions SET
               display_name = $3,
               data_type = $4,
               is_primary_key = COALESCE($5, is_primary_key),
               is_nullable = COALESCE($6, is_nullable),
               is_unique = COALESCE($7, is_unique),
               default_value = $8,
               max_length = $9,
               min_value = $10,
               max_value = $11,
               regex_pattern = $12,
               display_order = COALESCE($13, display_order),
               is_searchable = COALESCE($14, is_searchable),
               is_sortable = COALESCE($15, is_sortable),
               is_filterable = COALESCE($16, is_filterable),
               is_visible_in_list = COALESCE($17, is_visible_in_list),
               is_visible_in_form = COALESCE($18, is_visible_in_form),
               is_readonly = COALESCE($19, is_readonly),
               input_type = COALESCE($20, input_type),
               select_options = $21,
               updated_at = now()
               WHERE table_id = $1 AND column_name = $2 RETURNING *"#
        )
        .bind(table_id)
        .bind(column_name)
        .bind(&input.display_name)
        .bind(&input.data_type)
        .bind(input.is_primary_key)
        .bind(input.is_nullable)
        .bind(input.is_unique)
        .bind(&input.default_value)
        .bind(input.max_length)
        .bind(input.min_value)
        .bind(input.max_value)
        .bind(&input.regex_pattern)
        .bind(input.display_order)
        .bind(input.is_searchable)
        .bind(input.is_sortable)
        .bind(input.is_filterable)
        .bind(input.is_visible_in_list)
        .bind(input.is_visible_in_form)
        .bind(input.is_readonly)
        .bind(&input.input_type)
        .bind(&input.select_options)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, table_id: Uuid, column_name: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM master_maintenance.column_definitions WHERE table_id = $1 AND column_name = $2")
            .bind(table_id)
            .bind(column_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ColumnDefinitionRow {
    id: Uuid,
    table_id: Uuid,
    column_name: String,
    display_name: String,
    data_type: String,
    is_primary_key: bool,
    is_nullable: bool,
    is_unique: bool,
    default_value: Option<String>,
    max_length: Option<i32>,
    min_value: Option<f64>,
    max_value: Option<f64>,
    regex_pattern: Option<String>,
    display_order: i32,
    is_searchable: bool,
    is_sortable: bool,
    is_filterable: bool,
    is_visible_in_list: bool,
    is_visible_in_form: bool,
    is_readonly: bool,
    input_type: String,
    select_options: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ColumnDefinitionRow> for ColumnDefinition {
    fn from(row: ColumnDefinitionRow) -> Self {
        Self {
            id: row.id,
            table_id: row.table_id,
            column_name: row.column_name,
            display_name: row.display_name,
            data_type: row.data_type,
            is_primary_key: row.is_primary_key,
            is_nullable: row.is_nullable,
            is_unique: row.is_unique,
            default_value: row.default_value,
            max_length: row.max_length,
            min_value: row.min_value,
            max_value: row.max_value,
            regex_pattern: row.regex_pattern,
            display_order: row.display_order,
            is_searchable: row.is_searchable,
            is_sortable: row.is_sortable,
            is_filterable: row.is_filterable,
            is_visible_in_list: row.is_visible_in_list,
            is_visible_in_form: row.is_visible_in_form,
            is_readonly: row.is_readonly,
            input_type: row.input_type,
            select_options: row.select_options,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
