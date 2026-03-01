use std::sync::Arc;
use serde_json::Value;
use uuid::Uuid;
use crate::domain::entity::import_job::ImportJob;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use crate::domain::repository::import_job_repository::ImportJobRepository;

pub struct ImportExportUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    import_job_repo: Arc<dyn ImportJobRepository>,
}

impl ImportExportUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        record_repo: Arc<dyn DynamicRecordRepository>,
        import_job_repo: Arc<dyn ImportJobRepository>,
    ) -> Self {
        Self {
            table_repo,
            column_repo,
            record_repo,
            import_job_repo,
        }
    }

    pub async fn import_records(
        &self,
        table_name: &str,
        data: &Value,
        started_by: &str,
    ) -> anyhow::Result<ImportJob> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        let columns = self.column_repo.find_by_table_id(table.id).await?;

        let records = data
            .get("records")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("'records' field must be a JSON array"))?;

        let total_rows = records.len() as i32;

        // Create import job with "processing" status
        let initial_job = ImportJob {
            id: Uuid::new_v4(),
            table_id: table.id,
            file_name: data
                .get("file_name")
                .and_then(|v| v.as_str())
                .unwrap_or("api_import")
                .to_string(),
            status: "processing".to_string(),
            total_rows,
            processed_rows: 0,
            error_rows: 0,
            error_details: None,
            started_by: started_by.to_string(),
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        let job = self.import_job_repo.create(&initial_job).await?;
        let mut processed = 0;
        let mut error_count = 0;
        let mut errors: Vec<Value> = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            match self.record_repo.create(&table, &columns, record).await {
                Ok(_) => {
                    processed += 1;
                }
                Err(e) => {
                    error_count += 1;
                    errors.push(serde_json::json!({
                        "row": idx + 1,
                        "error": e.to_string(),
                    }));
                }
            }
        }

        // Update job with results
        let mut completed_job = job;
        completed_job.processed_rows = processed;
        completed_job.error_rows = error_count;
        completed_job.status = if error_count == 0 {
            "completed".to_string()
        } else if processed == 0 {
            "failed".to_string()
        } else {
            "completed_with_errors".to_string()
        };
        if !errors.is_empty() {
            completed_job.error_details = Some(serde_json::json!(errors));
        }
        completed_job.completed_at = Some(chrono::Utc::now());

        self.import_job_repo
            .update(completed_job.id, &completed_job)
            .await
    }

    pub async fn export_records(&self, table_name: &str) -> anyhow::Result<Value> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        let columns = self.column_repo.find_by_table_id(table.id).await?;

        // Fetch all records (large page size)
        let (records, total) = self
            .record_repo
            .find_all(&table, &columns, 1, i32::MAX, None, None, None)
            .await?;

        Ok(serde_json::json!({
            "table": table_name,
            "total": total,
            "records": records,
        }))
    }

    pub async fn get_import_job(&self, id: Uuid) -> anyhow::Result<Option<ImportJob>> {
        self.import_job_repo.find_by_id(id).await
    }
}
