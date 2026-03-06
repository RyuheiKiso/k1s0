use crate::domain::entity::table_relationship::{CreateTableRelationship, TableRelationship};
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::table_relationship_repository::TableRelationshipRepository;
use crate::infrastructure::schema::PhysicalSchemaManager;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageRelationshipsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    relationship_repo: Arc<dyn TableRelationshipRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    schema_manager: Arc<PhysicalSchemaManager>,
}

impl ManageRelationshipsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        relationship_repo: Arc<dyn TableRelationshipRepository>,
        record_repo: Arc<dyn DynamicRecordRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        schema_manager: Arc<PhysicalSchemaManager>,
    ) -> Self {
        Self {
            table_repo,
            relationship_repo,
            record_repo,
            column_repo,
            schema_manager,
        }
    }

    pub async fn list_relationships(&self) -> anyhow::Result<Vec<TableRelationship>> {
        self.relationship_repo.find_all().await
    }

    pub async fn create_relationship(
        &self,
        input: &Value,
        _created_by: &str,
    ) -> anyhow::Result<TableRelationship> {
        let create_input: CreateTableRelationship = serde_json::from_value(input.clone())?;

        // Verify source table exists
        let source_table = self
            .table_repo
            .find_by_name(&create_input.source_table)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Source table '{}' not found", create_input.source_table)
            })?;

        // Verify target table exists
        let target_table = self
            .table_repo
            .find_by_name(&create_input.target_table)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("Target table '{}' not found", create_input.target_table)
            })?;

        self.ensure_column_exists(source_table.id, &create_input.source_column)
            .await?;
        self.ensure_column_exists(target_table.id, &create_input.target_column)
            .await?;

        let relationship = TableRelationship {
            id: Uuid::new_v4(),
            source_table_id: source_table.id,
            source_column: create_input.source_column,
            target_table_id: target_table.id,
            target_column: create_input.target_column,
            relationship_type: create_input.relationship_type,
            display_name: create_input.display_name,
            is_cascade_delete: create_input.is_cascade_delete.unwrap_or(false),
            created_at: chrono::Utc::now(),
        };

        self.schema_manager
            .create_relationship(&source_table, &target_table, &relationship)
            .await?;
        self.relationship_repo.create(&relationship).await
    }

    pub async fn update_relationship(
        &self,
        id: Uuid,
        input: &Value,
    ) -> anyhow::Result<TableRelationship> {
        let mut relationship = self
            .relationship_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Relationship not found"))?;
        let source_table = self
            .table_repo
            .find_by_id(relationship.source_table_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source table not found"))?;
        let target_table = self
            .table_repo
            .find_by_id(relationship.target_table_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Target table not found"))?;

        if let Some(display_name) = input.get("display_name").and_then(|v| v.as_str()) {
            relationship.display_name = Some(display_name.to_string());
        }
        if let Some(cascade) = input.get("is_cascade_delete").and_then(|v| v.as_bool()) {
            relationship.is_cascade_delete = cascade;
        }
        if let Some(rel_type) = input.get("relationship_type") {
            relationship.relationship_type = serde_json::from_value(rel_type.clone())?;
        }
        if let Some(source_column) = input.get("source_column").and_then(|v| v.as_str()) {
            relationship.source_column = source_column.to_string();
        }
        if let Some(target_column) = input.get("target_column").and_then(|v| v.as_str()) {
            relationship.target_column = target_column.to_string();
        }

        self.ensure_column_exists(source_table.id, &relationship.source_column)
            .await?;
        self.ensure_column_exists(target_table.id, &relationship.target_column)
            .await?;

        self.schema_manager
            .update_relationship(&source_table, &target_table, &relationship)
            .await?;
        self.relationship_repo.update(id, &relationship).await
    }

    pub async fn delete_relationship(&self, id: Uuid) -> anyhow::Result<()> {
        let relationship = self
            .relationship_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Relationship not found"))?;
        let source_table = self
            .table_repo
            .find_by_id(relationship.source_table_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source table not found"))?;
        self.schema_manager
            .delete_relationship(&source_table, relationship.id)
            .await?;
        self.relationship_repo.delete(id).await
    }

    pub async fn get_related_records(
        &self,
        table_name: &str,
        record_id: &str,
    ) -> anyhow::Result<Value> {
        // Get table definition
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let table_columns = self.column_repo.find_by_table_id(table.id).await?;
        let current_record = self
            .record_repo
            .find_by_id(&table, &table_columns, record_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Record '{}' not found", record_id))?;

        // Find relationships where this table is the source
        let relationships = self.relationship_repo.find_by_table_id(table.id).await?;

        let mut related_data = serde_json::Map::new();

        for rel in &relationships {
            let (lookup_table_id, lookup_column, current_value, related_table_id) =
                if rel.source_table_id == table.id {
                    (
                        rel.target_table_id,
                        rel.target_column.as_str(),
                        current_record.get(&rel.source_column),
                        rel.target_table_id,
                    )
                } else {
                    (
                        rel.source_table_id,
                        rel.source_column.as_str(),
                        current_record.get(&rel.target_column),
                        rel.source_table_id,
                    )
                };

            let Some(current_value) = current_value else {
                continue;
            };
            if current_value.is_null() {
                continue;
            }

            let related_table = self
                .table_repo
                .find_by_id(related_table_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Related table not found"))?;
            let related_columns = self.column_repo.find_by_table_id(lookup_table_id).await?;
            let filter = format!("{}:{}", lookup_column, scalar_filter_value(current_value));
            let (records, _total) = self
                .record_repo
                .find_all(
                    &related_table,
                    &related_columns,
                    1,
                    100,
                    None,
                    Some(&filter),
                    None,
                )
                .await?;

            related_data.insert(
                related_table.name.clone(),
                serde_json::json!({
                    "relationship_id": rel.id,
                    "relationship_type": rel.relationship_type,
                    "records": records,
                }),
            );
        }

        Ok(Value::Object(related_data))
    }

    async fn ensure_column_exists(&self, table_id: Uuid, column_name: &str) -> anyhow::Result<()> {
        self.column_repo
            .find_by_table_and_column(table_id, column_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", column_name))?;
        Ok(())
    }
}

fn scalar_filter_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        other => other.to_string(),
    }
}
