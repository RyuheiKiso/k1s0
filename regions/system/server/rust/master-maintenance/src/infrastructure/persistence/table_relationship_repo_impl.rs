use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entity::table_relationship::TableRelationship;
use crate::domain::repository::table_relationship_repository::TableRelationshipRepository;
use crate::domain::value_object::relationship_type::RelationshipType;

pub struct TableRelationshipPostgresRepository {
    pool: PgPool,
}

impl TableRelationshipPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TableRelationshipRepository for TableRelationshipPostgresRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<TableRelationship>> {
        let rows = sqlx::query_as::<_, TableRelationshipRow>(
            "SELECT * FROM master_maintenance.table_relationships ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableRelationship>> {
        let row = sqlx::query_as::<_, TableRelationshipRow>(
            "SELECT * FROM master_maintenance.table_relationships WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<TableRelationship>> {
        let rows = sqlx::query_as::<_, TableRelationshipRow>(
            "SELECT * FROM master_maintenance.table_relationships WHERE source_table_id = $1 OR target_table_id = $1 ORDER BY created_at"
        )
        .bind(table_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn create(&self, relationship: &TableRelationship) -> anyhow::Result<TableRelationship> {
        let row = sqlx::query_as::<_, TableRelationshipRow>(
            r#"INSERT INTO master_maintenance.table_relationships
               (id, source_table_id, source_column, target_table_id, target_column,
                relationship_type, display_name, is_cascade_delete)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#
        )
        .bind(relationship.id)
        .bind(relationship.source_table_id)
        .bind(&relationship.source_column)
        .bind(relationship.target_table_id)
        .bind(&relationship.target_column)
        .bind(relationship.relationship_type.to_string())
        .bind(&relationship.display_name)
        .bind(relationship.is_cascade_delete)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(&self, id: Uuid, relationship: &TableRelationship) -> anyhow::Result<TableRelationship> {
        let row = sqlx::query_as::<_, TableRelationshipRow>(
            r#"UPDATE master_maintenance.table_relationships SET
               source_column = $2,
               target_column = $3,
               relationship_type = $4,
               display_name = $5,
               is_cascade_delete = $6
               WHERE id = $1 RETURNING *"#
        )
        .bind(id)
        .bind(&relationship.source_column)
        .bind(&relationship.target_column)
        .bind(relationship.relationship_type.to_string())
        .bind(&relationship.display_name)
        .bind(relationship.is_cascade_delete)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM master_maintenance.table_relationships WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TableRelationshipRow {
    id: Uuid,
    source_table_id: Uuid,
    source_column: String,
    target_table_id: Uuid,
    target_column: String,
    relationship_type: String,
    display_name: Option<String>,
    is_cascade_delete: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TableRelationshipRow> for TableRelationship {
    fn from(row: TableRelationshipRow) -> Self {
        let rel_type = match row.relationship_type.as_str() {
            "one_to_one" => RelationshipType::OneToOne,
            "one_to_many" => RelationshipType::OneToMany,
            "many_to_one" => RelationshipType::ManyToOne,
            "many_to_many" => RelationshipType::ManyToMany,
            _ => RelationshipType::OneToMany,
        };
        Self {
            id: row.id,
            source_table_id: row.source_table_id,
            source_column: row.source_column,
            target_table_id: row.target_table_id,
            target_column: row.target_column,
            relationship_type: rel_type,
            display_name: row.display_name,
            is_cascade_delete: row.is_cascade_delete,
            created_at: row.created_at,
        }
    }
}
