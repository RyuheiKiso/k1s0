use crate::domain::entity::table_definition::{
    CreateTableDefinition, TableDefinition, UpdateTableDefinition,
};
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::value_object::domain_filter::DomainFilter;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

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
    async fn find_all(
        &self,
        category: Option<&str>,
        active_only: bool,
        domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>> {
        let mut query =
            String::from("SELECT * FROM master_maintenance.table_definitions WHERE 1=1");
        if active_only {
            query.push_str(" AND is_active = true");
        }
        if let Some(cat) = category {
            query.push_str(&format!(" AND category = '{}'", cat));
        }
        match domain_filter {
            DomainFilter::All => {}
            DomainFilter::System => {
                query.push_str(" AND domain_scope IS NULL");
            }
            DomainFilter::Domain(domain) => {
                query.push_str(&format!(" AND domain_scope = '{}'", domain));
            }
        }
        query.push_str(" ORDER BY sort_order, name");

        let rows = sqlx::query_as::<_, TableDefinitionRow>(&query)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_name(&self, name: &str, domain_scope: Option<&str>) -> anyhow::Result<Option<TableDefinition>> {
        let row = if let Some(ds) = domain_scope {
            sqlx::query_as::<_, TableDefinitionRow>(
                "SELECT * FROM master_maintenance.table_definitions WHERE name = $1 AND domain_scope = $2",
            )
            .bind(name)
            .bind(ds)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, TableDefinitionRow>(
                "SELECT * FROM master_maintenance.table_definitions WHERE name = $1 AND domain_scope IS NULL",
            )
            .bind(name)
            .fetch_optional(&self.pool)
            .await?
        };
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        let row = sqlx::query_as::<_, TableDefinitionRow>(
            "SELECT * FROM master_maintenance.table_definitions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn create(
        &self,
        input: &CreateTableDefinition,
        created_by: &str,
    ) -> anyhow::Result<TableDefinition> {
        let row = sqlx::query_as::<_, TableDefinitionRow>(
            r#"INSERT INTO master_maintenance.table_definitions
               (name, schema_name, database_name, display_name, description, category,
                allow_create, allow_update, allow_delete, read_roles, write_roles, admin_roles,
                sort_order, created_by, domain_scope)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
               RETURNING *"#,
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
        .bind(input.read_roles.clone().unwrap_or_default())
        .bind(input.write_roles.clone().unwrap_or_default())
        .bind(input.admin_roles.clone().unwrap_or_default())
        .bind(input.sort_order.unwrap_or(0))
        .bind(created_by)
        .bind(&input.domain_scope)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(
        &self,
        name: &str,
        input: &UpdateTableDefinition,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<TableDefinition> {
        let row = if let Some(ds) = domain_scope {
            sqlx::query_as::<_, TableDefinitionRow>(
                r#"UPDATE master_maintenance.table_definitions SET
                   display_name = COALESCE($2, display_name),
                   description = COALESCE($3, description),
                   category = COALESCE($4, category),
                   is_active = COALESCE($5, is_active),
                   allow_create = COALESCE($6, allow_create),
                   allow_update = COALESCE($7, allow_update),
                   allow_delete = COALESCE($8, allow_delete),
                   read_roles = COALESCE($9, read_roles),
                   write_roles = COALESCE($10, write_roles),
                   admin_roles = COALESCE($11, admin_roles),
                   sort_order = COALESCE($12, sort_order),
                   updated_at = now()
                   WHERE name = $1 AND domain_scope = $13 RETURNING *"#,
            )
            .bind(name)
            .bind(&input.display_name)
            .bind(&input.description)
            .bind(&input.category)
            .bind(input.is_active)
            .bind(input.allow_create)
            .bind(input.allow_update)
            .bind(input.allow_delete)
            .bind(&input.read_roles)
            .bind(&input.write_roles)
            .bind(&input.admin_roles)
            .bind(input.sort_order)
            .bind(ds)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, TableDefinitionRow>(
                r#"UPDATE master_maintenance.table_definitions SET
                   display_name = COALESCE($2, display_name),
                   description = COALESCE($3, description),
                   category = COALESCE($4, category),
                   is_active = COALESCE($5, is_active),
                   allow_create = COALESCE($6, allow_create),
                   allow_update = COALESCE($7, allow_update),
                   allow_delete = COALESCE($8, allow_delete),
                   read_roles = COALESCE($9, read_roles),
                   write_roles = COALESCE($10, write_roles),
                   admin_roles = COALESCE($11, admin_roles),
                   sort_order = COALESCE($12, sort_order),
                   updated_at = now()
                   WHERE name = $1 AND domain_scope IS NULL RETURNING *"#,
            )
            .bind(name)
            .bind(&input.display_name)
            .bind(&input.description)
            .bind(&input.category)
            .bind(input.is_active)
            .bind(input.allow_create)
            .bind(input.allow_update)
            .bind(input.allow_delete)
            .bind(&input.read_roles)
            .bind(&input.write_roles)
            .bind(&input.admin_roles)
            .bind(input.sort_order)
            .fetch_one(&self.pool)
            .await?
        };
        Ok(row.into())
    }

    async fn delete(&self, name: &str, domain_scope: Option<&str>) -> anyhow::Result<()> {
        if let Some(ds) = domain_scope {
            sqlx::query("DELETE FROM master_maintenance.table_definitions WHERE name = $1 AND domain_scope = $2")
                .bind(name)
                .bind(ds)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("DELETE FROM master_maintenance.table_definitions WHERE name = $1 AND domain_scope IS NULL")
                .bind(name)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn find_domains(&self) -> anyhow::Result<Vec<(String, i64)>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT domain_scope, COUNT(*) as table_count
               FROM master_maintenance.table_definitions
               WHERE domain_scope IS NOT NULL
               GROUP BY domain_scope
               ORDER BY domain_scope"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
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
    read_roles: Vec<String>,
    write_roles: Vec<String>,
    admin_roles: Vec<String>,
    sort_order: i32,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    domain_scope: Option<String>,
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
            read_roles: row.read_roles,
            write_roles: row.write_roles,
            admin_roles: row.admin_roles,
            sort_order: row.sort_order,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            domain_scope: row.domain_scope,
        }
    }
}
