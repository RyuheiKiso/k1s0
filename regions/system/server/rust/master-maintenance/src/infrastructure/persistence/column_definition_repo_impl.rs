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
    async fn find_by_table_id(&self, _table_id: Uuid) -> anyhow::Result<Vec<ColumnDefinition>> {
        todo!("implement find_by_table_id")
    }
    async fn find_by_table_and_column(&self, _table_id: Uuid, _column_name: &str) -> anyhow::Result<Option<ColumnDefinition>> {
        todo!("implement find_by_table_and_column")
    }
    async fn create_batch(&self, _table_id: Uuid, _columns: &[CreateColumnDefinition]) -> anyhow::Result<Vec<ColumnDefinition>> {
        todo!("implement create_batch")
    }
    async fn update(&self, _table_id: Uuid, _column_name: &str, _input: &CreateColumnDefinition) -> anyhow::Result<ColumnDefinition> {
        todo!("implement update")
    }
    async fn delete(&self, _table_id: Uuid, _column_name: &str) -> anyhow::Result<()> {
        todo!("implement delete")
    }
}
