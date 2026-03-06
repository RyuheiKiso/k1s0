use crate::domain::entity::change_log::ChangeLog;
use crate::domain::repository::change_log_repository::ChangeLogRepository;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::service::rule_engine_service::RuleEngineService;
use crate::domain::value_object::rule_result::RuleResult;
use crate::usecase::rule_evaluator::RuleEvaluator;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RecordMutationOutput {
    pub record: Value,
    pub warnings: Vec<RuleResult>,
}

#[derive(Debug)]
pub struct RecordValidationError {
    pub errors: Vec<RuleResult>,
    pub warnings: Vec<RuleResult>,
}

impl std::fmt::Display for RecordValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "validation failed")
    }
}

impl std::error::Error for RecordValidationError {}

pub struct CrudRecordsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    rule_repo: Arc<dyn ConsistencyRuleRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    change_log_repo: Arc<dyn ChangeLogRepository>,
    rule_evaluator: RuleEvaluator,
}

pub struct ListRecordsOutput {
    pub table_name: String,
    pub display_name: String,
    pub allow_create: bool,
    pub allow_update: bool,
    pub allow_delete: bool,
    pub records: Vec<Value>,
    pub total: i64,
}

impl CrudRecordsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        rule_repo: Arc<dyn ConsistencyRuleRepository>,
        record_repo: Arc<dyn DynamicRecordRepository>,
        change_log_repo: Arc<dyn ChangeLogRepository>,
        rule_engine: Arc<dyn RuleEngineService>,
    ) -> Self {
        let rule_evaluator = RuleEvaluator::new(
            table_repo.clone(),
            column_repo.clone(),
            rule_repo.clone(),
            record_repo.clone(),
            rule_engine.clone(),
        );
        Self {
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
            rule_evaluator,
        }
    }

    async fn evaluate_before_save_rules(
        &self,
        table: &crate::domain::entity::table_definition::TableDefinition,
        columns: &[crate::domain::entity::column_definition::ColumnDefinition],
        data: &Value,
    ) -> anyhow::Result<Vec<RuleResult>> {
        let rules = self
            .rule_repo
            .find_by_table_id(table.id, Some("before_save"))
            .await?;

        let mut results = Vec::new();
        for rule in rules.into_iter().filter(|rule| rule.is_active) {
            let result = self
                .rule_evaluator
                .evaluate_rule(&rule, table, columns, data)
                .await?
                .with_rule_info(rule.id.to_string(), rule.name.clone());
            results.push(result);
        }

        Ok(results)
    }

    fn fail_on_rule_errors(results: &[RuleResult]) -> anyhow::Result<Vec<RuleResult>> {
        let errors: Vec<RuleResult> = results
            .iter()
            .filter(|result| !result.passed && result.severity == "error")
            .cloned()
            .collect();
        let warnings: Vec<RuleResult> = results
            .iter()
            .filter(|result| result.severity == "warning")
            .cloned()
            .collect();

        if errors.is_empty() {
            Ok(warnings)
        } else {
            Err(anyhow::Error::new(RecordValidationError {
                errors,
                warnings,
            }))
        }
    }

    pub async fn list_records(
        &self,
        table_name: &str,
        page: i32,
        page_size: i32,
        sort: Option<&str>,
        filter: Option<&str>,
        search: Option<&str>,
        selected_columns: Option<&str>,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<ListRecordsOutput> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let column_defs = self.column_repo.find_by_table_id(table.id).await?;
        let (mut records, total) = self
            .record_repo
            .find_all(&table, &column_defs, page, page_size, sort, filter, search)
            .await?;

        if let Some(raw_columns) = selected_columns {
            let selected: std::collections::HashSet<String> = raw_columns
                .split(',')
                .map(|c| c.trim())
                .filter(|c| !c.is_empty())
                .map(ToString::to_string)
                .collect();
            if !selected.is_empty() {
                for record in &mut records {
                    if let Some(obj) = record.as_object_mut() {
                        obj.retain(|k, _| selected.contains(k));
                    }
                }
            }
        }

        Ok(ListRecordsOutput {
            table_name: table.name,
            display_name: table.display_name,
            allow_create: table.allow_create,
            allow_update: table.allow_update,
            allow_delete: table.allow_delete,
            records,
            total,
        })
    }

    pub async fn get_record(
        &self,
        table_name: &str,
        record_id: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<Value>> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        self.record_repo
            .find_by_id(&table, &columns, record_id)
            .await
    }

    pub async fn table_permissions(
        &self,
        table_name: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<(bool, bool, bool)> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        Ok((table.allow_create, table.allow_update, table.allow_delete))
    }

    pub async fn create_record(
        &self,
        table_name: &str,
        data: &Value,
        created_by: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<RecordMutationOutput> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        if !table.allow_create {
            anyhow::bail!("Create not allowed for table '{}'", table_name);
        }
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let warnings = Self::fail_on_rule_errors(
            &self
                .evaluate_before_save_rules(&table, &columns, data)
                .await?,
        )?;
        let record = self.record_repo.create(&table, &columns, data).await?;

        let log = ChangeLog {
            id: uuid::Uuid::new_v4(),
            target_table: table_name.to_string(),
            target_record_id: record
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            operation: "INSERT".to_string(),
            before_data: None,
            after_data: Some(record.clone()),
            changed_columns: None,
            changed_by: created_by.to_string(),
            change_reason: None,
            trace_id: None,
            domain_scope: domain_scope.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(RecordMutationOutput { record, warnings })
    }

    pub async fn update_record(
        &self,
        table_name: &str,
        record_id: &str,
        data: &Value,
        updated_by: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<RecordMutationOutput> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        if !table.allow_update {
            anyhow::bail!("Update not allowed for table '{}'", table_name);
        }
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let warnings = Self::fail_on_rule_errors(
            &self
                .evaluate_before_save_rules(&table, &columns, data)
                .await?,
        )?;

        let before = self
            .record_repo
            .find_by_id(&table, &columns, record_id)
            .await?;
        let record = self
            .record_repo
            .update(&table, &columns, record_id, data)
            .await?;

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
            domain_scope: domain_scope.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(RecordMutationOutput { record, warnings })
    }

    pub async fn delete_record(
        &self,
        table_name: &str,
        record_id: &str,
        deleted_by: &str,
    ) -> anyhow::Result<()> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;
        if !table.allow_delete {
            anyhow::bail!("Delete not allowed for table '{}'", table_name);
        }
        let columns = self.column_repo.find_by_table_id(table.id).await?;

        let before = self
            .record_repo
            .find_by_id(&table, &columns, record_id)
            .await?;
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
