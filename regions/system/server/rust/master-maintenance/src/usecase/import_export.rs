use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::entity::import_job::ImportJob;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::import_job_repository::ImportJobRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::value_object::rule_result::RuleResult;
use crate::usecase::crud_records::CrudRecordsUseCase;
use calamine::{open_workbook_auto_from_rs, Reader};
use rust_xlsxwriter::Workbook;
use serde_json::Map;
use serde_json::Value;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

const EXPORT_PAGE_SIZE: i32 = 500;

#[derive(Debug, Clone)]
pub struct ExportedFile {
    pub file_name: String,
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ExportRecordsOutput {
    pub table: String,
    pub format: String,
    pub total: i64,
    pub records: Vec<Value>,
    pub file: Option<ExportedFile>,
}

impl ExportRecordsOutput {
    pub fn as_json(&self) -> Value {
        serde_json::json!({
            "table": self.table,
            "format": self.format,
            "total": self.total,
            "records": self.records,
        })
    }
}

pub struct ImportExportUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    import_job_repo: Arc<dyn ImportJobRepository>,
    crud_records_uc: Arc<CrudRecordsUseCase>,
}

impl ImportExportUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        import_job_repo: Arc<dyn ImportJobRepository>,
        crud_records_uc: Arc<CrudRecordsUseCase>,
    ) -> Self {
        Self {
            table_repo,
            column_repo,
            import_job_repo,
            crud_records_uc,
        }
    }

    pub async fn import_records(
        &self,
        table_name: &str,
        data: &Value,
        started_by: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<ImportJob> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let records = self.parse_import_records(data, &columns)?;

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
            match self
                .crud_records_uc
                .create_record(table_name, record, started_by, domain_scope, None)
                .await
            {
                Ok(output) => {
                    processed += 1;
                    if !output.warnings.is_empty() {
                        errors.push(serde_json::json!({
                            "row": idx + 1,
                            "warnings": summarize_warnings(&output.warnings),
                        }));
                    }
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

    pub async fn export_records(
        &self,
        table_name: &str,
        format: Option<&str>,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<ExportRecordsOutput> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let export_columns: Vec<ColumnDefinition> = columns
            .into_iter()
            .filter(|column| column.is_visible_in_list)
            .collect();
        let selected_columns = (!export_columns.is_empty()).then(|| {
            export_columns
                .iter()
                .map(|column| column.column_name.clone())
                .collect::<Vec<_>>()
                .join(",")
        });
        let (records, total) = self
            .collect_export_records(table_name, selected_columns.as_deref(), domain_scope)
            .await?;

        let normalized_format = normalize_export_format(format)?;
        let file = match normalized_format {
            "json" => None,
            "csv" => Some(ExportedFile {
                file_name: format!("{}.csv", table_name),
                content_type: "text/csv; charset=utf-8",
                bytes: Self::export_as_csv(&export_columns, &records)?.into_bytes(),
            }),
            "xlsx" => Some(ExportedFile {
                file_name: format!("{}.xlsx", table_name),
                content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                bytes: Self::export_as_xlsx(&export_columns, &records)?,
            }),
            _ => unreachable!(),
        };

        Ok(ExportRecordsOutput {
            table: table_name.to_string(),
            format: normalized_format.to_string(),
            total,
            records,
            file,
        })
    }

    pub async fn get_import_job(&self, id: Uuid) -> anyhow::Result<Option<ImportJob>> {
        self.import_job_repo.find_by_id(id).await
    }

    pub async fn import_records_from_file(
        &self,
        table_name: &str,
        file_name: &str,
        content: &[u8],
        started_by: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<ImportJob> {
        let extension = file_name
            .rsplit('.')
            .next()
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| "csv".to_string());

        let data = match extension.as_str() {
            "csv" => serde_json::json!({
                "file_name": file_name,
                "format": "csv",
                "content": String::from_utf8(content.to_vec())?,
            }),
            "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" => serde_json::json!({
                "file_name": file_name,
                "records": self.parse_excel_records(content)?,
            }),
            other => anyhow::bail!("unsupported import file extension: {}", other),
        };

        self.import_records(table_name, &data, started_by, domain_scope)
            .await
    }

    fn parse_import_records(
        &self,
        data: &Value,
        columns: &[crate::domain::entity::column_definition::ColumnDefinition],
    ) -> anyhow::Result<Vec<Value>> {
        if let Some(records) = data.get("records").and_then(|v| v.as_array()) {
            return Ok(records.clone());
        }

        let format = data
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");
        match format {
            "csv" => {
                let content = data
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("'content' field must be a CSV string"))?;
                self.parse_csv_records(content, columns)
            }
            other => anyhow::bail!("unsupported import format: {}", other),
        }
    }

    fn parse_csv_records(
        &self,
        content: &str,
        columns: &[crate::domain::entity::column_definition::ColumnDefinition],
    ) -> anyhow::Result<Vec<Value>> {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let headers = reader.headers()?.clone();
        let mut records = Vec::new();

        for row in reader.records() {
            let row = row?;
            let mut object = Map::new();
            for (index, raw_value) in row.iter().enumerate() {
                let Some(header) = headers.get(index) else {
                    continue;
                };
                if raw_value.is_empty() {
                    object.insert(header.to_string(), Value::Null);
                    continue;
                }
                let value = columns
                    .iter()
                    .find(|column| column.column_name == header)
                    .map(|column| parse_scalar_value(raw_value, &column.data_type))
                    .transpose()?
                    .unwrap_or_else(|| Value::String(raw_value.to_string()));
                object.insert(header.to_string(), value);
            }
            records.push(Value::Object(object));
        }

        Ok(records)
    }

    fn parse_excel_records(&self, content: &[u8]) -> anyhow::Result<Vec<Value>> {
        let cursor = Cursor::new(content.to_vec());
        let mut workbook = open_workbook_auto_from_rs(cursor)?;
        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("workbook does not contain any sheets"))?;
        let range = workbook.worksheet_range(&sheet_name)?;
        let mut rows = range.rows();
        let headers: Vec<String> = rows
            .next()
            .ok_or_else(|| anyhow::anyhow!("worksheet is empty"))?
            .iter()
            .map(|cell| cell.to_string())
            .collect();

        if headers.is_empty() {
            anyhow::bail!("worksheet header is empty");
        }

        let mut records = Vec::new();
        for row in rows {
            let mut object = Map::new();
            for (index, cell) in row.iter().enumerate() {
                let Some(header) = headers.get(index) else {
                    continue;
                };
                object.insert(header.clone(), excel_cell_to_json(cell));
            }
            if object.values().all(Value::is_null) {
                continue;
            }
            records.push(Value::Object(object));
        }

        Ok(records)
    }

    fn export_as_csv(columns: &[ColumnDefinition], records: &[Value]) -> anyhow::Result<String> {
        let ordered_columns: Vec<&str> = columns
            .iter()
            .map(|column| column.column_name.as_str())
            .collect();
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record(&ordered_columns)?;

        for record in records {
            let object = record
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("record must be a JSON object"))?;
            let row: Vec<String> = ordered_columns
                .iter()
                .map(|column| stringify_csv_value(object.get(*column).unwrap_or(&Value::Null)))
                .collect();
            writer.write_record(row)?;
        }

        let bytes = writer.into_inner()?;
        Ok(String::from_utf8(bytes)?)
    }

    async fn collect_export_records(
        &self,
        table_name: &str,
        selected_columns: Option<&str>,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)> {
        let mut page = 1;
        let mut total = 0;
        let mut records = Vec::new();

        loop {
            let result = self
                .crud_records_uc
                .list_records(
                    table_name,
                    page,
                    EXPORT_PAGE_SIZE,
                    None,
                    None,
                    None,
                    selected_columns,
                    domain_scope,
                )
                .await?;

            if page == 1 {
                total = result.total;
            }

            if result.records.is_empty() {
                break;
            }

            records.extend(result.records);

            if records.len() as i64 >= total {
                break;
            }

            page += 1;
        }

        Ok((records, total))
    }

    fn export_as_xlsx(columns: &[ColumnDefinition], records: &[Value]) -> anyhow::Result<Vec<u8>> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        for (column_index, column) in columns.iter().enumerate() {
            worksheet.write_string(0, column_index as u16, &column.column_name)?;
        }

        for (row_index, record) in records.iter().enumerate() {
            let object = record
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("record must be a JSON object"))?;
            for (column_index, column) in columns.iter().enumerate() {
                worksheet.write_string(
                    (row_index + 1) as u32,
                    column_index as u16,
                    stringify_csv_value(object.get(&column.column_name).unwrap_or(&Value::Null)),
                )?;
            }
        }

        Ok(workbook.save_to_buffer()?)
    }
}

fn normalize_export_format(format: Option<&str>) -> anyhow::Result<&'static str> {
    match format
        .unwrap_or("json")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "json" => Ok("json"),
        "csv" => Ok("csv"),
        "xlsx" | "xls" => Ok("xlsx"),
        other => anyhow::bail!("unsupported export format: {}", other),
    }
}

fn summarize_warnings(warnings: &[RuleResult]) -> Vec<Value> {
    warnings
        .iter()
        .map(|warning| {
            serde_json::json!({
                "rule_id": warning.rule_id,
                "rule_name": warning.rule_name,
                "message": warning.message,
                "severity": warning.severity,
            })
        })
        .collect()
}

fn parse_scalar_value(value: &str, data_type: &str) -> anyhow::Result<Value> {
    Ok(match data_type {
        "integer" => Value::Number(value.parse::<i64>()?.into()),
        "decimal" => serde_json::Number::from_f64(value.parse::<f64>()?)
            .map(Value::Number)
            .ok_or_else(|| anyhow::anyhow!("invalid decimal value: {}", value))?,
        "boolean" => Value::Bool(value.parse::<bool>()?),
        "jsonb" => serde_json::from_str(value)?,
        _ => Value::String(value.to_string()),
    })
}

fn stringify_csv_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        other => other.to_string(),
    }
}

fn excel_cell_to_json(cell: &calamine::Data) -> Value {
    match cell {
        calamine::Data::Empty => Value::Null,
        calamine::Data::Int(value) => Value::Number((*value).into()),
        calamine::Data::Float(value) => serde_json::Number::from_f64(*value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        calamine::Data::String(value) => Value::String(value.clone()),
        calamine::Data::Bool(value) => Value::Bool(*value),
        calamine::Data::DateTimeIso(value) | calamine::Data::DurationIso(value) => {
            Value::String(value.clone())
        }
        calamine::Data::DateTime(value) => Value::String(value.to_string()),
        calamine::Data::Error(value) => Value::String(value.to_string()),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::column_definition::ColumnDefinition;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_column(name: &str) -> ColumnDefinition {
        ColumnDefinition {
            id: Uuid::new_v4(),
            table_id: Uuid::new_v4(),
            column_name: name.to_string(),
            display_name: name.to_string(),
            data_type: "text".to_string(),
            is_primary_key: false,
            is_nullable: true,
            is_unique: false,
            default_value: None,
            max_length: None,
            min_value: None,
            max_value: None,
            regex_pattern: None,
            display_order: 0,
            is_searchable: false,
            is_sortable: false,
            is_filterable: false,
            is_visible_in_list: true,
            is_visible_in_form: true,
            is_readonly: false,
            input_type: "text".to_string(),
            select_options: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn parse_scalar_value_handles_basic_types() {
        assert_eq!(
            parse_scalar_value("42", "integer").unwrap(),
            Value::Number(42.into())
        );
        assert_eq!(
            parse_scalar_value("true", "boolean").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            parse_scalar_value("12.5", "decimal").unwrap(),
            serde_json::json!(12.5)
        );
    }

    #[test]
    fn stringify_csv_value_serializes_objects_as_json() {
        let value = serde_json::json!({ "enabled": true });
        assert_eq!(stringify_csv_value(&value), r#"{"enabled":true}"#);
    }

    #[test]
    fn excel_cell_to_json_handles_primitive_cells() {
        assert_eq!(
            excel_cell_to_json(&calamine::Data::Int(7)),
            Value::Number(7.into())
        );
        assert_eq!(
            excel_cell_to_json(&calamine::Data::Bool(true)),
            Value::Bool(true)
        );
    }

    #[test]
    fn normalize_export_format_accepts_excel_aliases() {
        assert_eq!(normalize_export_format(Some("xlsx")).unwrap(), "xlsx");
        assert_eq!(normalize_export_format(Some("xls")).unwrap(), "xlsx");
        assert!(normalize_export_format(Some("pdf")).is_err());
    }

    #[test]
    fn export_as_xlsx_generates_readable_workbook() {
        let columns = vec![sample_column("id"), sample_column("name")];
        let records = vec![serde_json::json!({
            "id": "dept-1",
            "name": "Platform",
        })];

        let bytes = ImportExportUseCase::export_as_xlsx(&columns, &records).unwrap();
        let mut workbook = open_workbook_auto_from_rs(Cursor::new(bytes)).unwrap();
        let sheet_name = workbook.sheet_names().first().cloned().unwrap();
        let range = workbook.worksheet_range(&sheet_name).unwrap();
        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect();

        assert_eq!(rows[0], vec![String::from("id"), String::from("name")]);
        assert_eq!(
            rows[1],
            vec![String::from("dept-1"), String::from("Platform")]
        );
    }
}
