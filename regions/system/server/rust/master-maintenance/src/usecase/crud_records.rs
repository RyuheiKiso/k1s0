use std::sync::Arc;
use serde_json::Value;
use crate::domain::entity::change_log::ChangeLog;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use crate::domain::repository::change_log_repository::ChangeLogRepository;

pub struct CrudRecordsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    change_log_repo: Arc<dyn ChangeLogRepository>,
}

impl CrudRecordsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        record_repo: Arc<dyn DynamicRecordRepository>,
        change_log_repo: Arc<dyn ChangeLogRepository>,
    ) -> Self {
        Self { table_repo, column_repo, record_repo, change_log_repo }
    }

    pub async fn list_records(
        &self,
        table_name: &str,
        page: i32,
        page_size: i32,
        sort: Option<&str>,
        filter: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        self.record_repo.find_all(&table, &columns, page, page_size, sort, filter, search).await
    }

    pub async fn get_record(&self, table_name: &str, record_id: &str) -> anyhow::Result<Option<Value>> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        self.record_repo.find_by_id(&table, &columns, record_id).await
    }

    pub async fn create_record(&self, table_name: &str, data: &Value, created_by: &str) -> anyhow::Result<Value> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let record = self.record_repo.create(&table, &columns, data).await?;

        let log = ChangeLog {
            id: uuid::Uuid::new_v4(),
            target_table: table_name.to_string(),
            target_record_id: record.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            operation: "INSERT".to_string(),
            before_data: None,
            after_data: Some(record.clone()),
            changed_columns: None,
            changed_by: created_by.to_string(),
            change_reason: None,
            trace_id: None,
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(record)
    }

    pub async fn update_record(&self, table_name: &str, record_id: &str, data: &Value, updated_by: &str) -> anyhow::Result<Value> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;

        let before = self.record_repo.find_by_id(&table, &columns, record_id).await?;
        let record = self.record_repo.update(&table, &columns, record_id, data).await?;

        let log = ChangeLog {
            id: uuid::Uuid::new_v4(),
            target_table: table_name.to_string(),
            target_record_id: record_id.to_string(),
            operation: "UPDATE".to_string(),
            before_data: before,
            after_data: Some(record.clone()),
            changed_columns: None,
            changed_by: updated_by.to_string(),
            change_reason: None,
            trace_id: None,
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(record)
    }

    pub async fn delete_record(&self, table_name: &str, record_id: &str, deleted_by: &str) -> anyhow::Result<()> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;

        let before = self.record_repo.find_by_id(&table, &columns, record_id).await?;
        self.record_repo.delete(&table, record_id).await?;

        let log = ChangeLog {
            id: uuid::Uuid::new_v4(),
            target_table: table_name.to_string(),
            target_record_id: record_id.to_string(),
            operation: "DELETE".to_string(),
            before_data: before,
            after_data: None,
            changed_columns: None,
            changed_by: deleted_by.to_string(),
            change_reason: None,
            trace_id: None,
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(())
    }
}
