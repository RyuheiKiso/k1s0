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
use std::collections::HashSet;
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

    #[allow(clippy::too_many_arguments)]
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
            let selected: HashSet<String> = raw_columns
                .split(',')
                .map(|c| c.trim())
                .filter(|c| !c.is_empty())
                .map(ToString::to_string)
                .collect();
            if !selected.is_empty() {
                for record in &mut records {
                    *record = filter_record_by_mode(
                        record,
                        &column_defs,
                        RecordVisibility::List,
                        Some(&selected),
                    );
                }
            } else {
                for record in &mut records {
                    *record =
                        filter_record_by_mode(record, &column_defs, RecordVisibility::List, None);
                }
            }
        } else {
            for record in &mut records {
                *record = filter_record_by_mode(record, &column_defs, RecordVisibility::List, None);
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
            .map(|record| {
                record.map(|value| {
                    filter_record_by_mode(&value, &columns, RecordVisibility::Form, None)
                })
            })
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
        trace_id: Option<String>,
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
            changed_columns: Some(changed_columns_for_create(&record, &columns)),
            changed_by: created_by.to_string(),
            change_reason: None,
            trace_id,
            domain_scope: domain_scope.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(RecordMutationOutput {
            record: filter_record_by_mode(&record, &columns, RecordVisibility::Form, None),
            warnings,
        })
    }

    pub async fn update_record(
        &self,
        table_name: &str,
        record_id: &str,
        data: &Value,
        updated_by: &str,
        domain_scope: Option<&str>,
        trace_id: Option<String>,
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
            before_data: before.clone(),
            after_data: Some(record.clone()),
            changed_columns: Some(changed_columns_for_update(
                before.as_ref(),
                &record,
                &columns,
            )),
            changed_by: updated_by.to_string(),
            change_reason: None,
            trace_id,
            domain_scope: domain_scope.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(RecordMutationOutput {
            record: filter_record_by_mode(&record, &columns, RecordVisibility::Form, None),
            warnings,
        })
    }

    pub async fn delete_record(
        &self,
        table_name: &str,
        record_id: &str,
        deleted_by: &str,
        domain_scope: Option<&str>,
        trace_id: Option<String>,
    ) -> anyhow::Result<()> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
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
            before_data: before.clone(),
            after_data: None,
            changed_columns: Some(changed_columns_for_delete(before.as_ref(), &columns)),
            changed_by: deleted_by.to_string(),
            change_reason: None,
            trace_id,
            domain_scope: domain_scope.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        let _ = self.change_log_repo.create(&log).await;

        Ok(())
    }
}

#[derive(Copy, Clone)]
enum RecordVisibility {
    List,
    Form,
}

fn allowed_columns(
    columns: &[crate::domain::entity::column_definition::ColumnDefinition],
    mode: RecordVisibility,
) -> HashSet<String> {
    columns
        .iter()
        .filter(|column| match mode {
            RecordVisibility::List => column.is_visible_in_list,
            RecordVisibility::Form => column.is_primary_key || column.is_visible_in_form,
        })
        .map(|column| column.column_name.clone())
        .collect()
}

fn filter_record_by_mode(
    record: &Value,
    columns: &[crate::domain::entity::column_definition::ColumnDefinition],
    mode: RecordVisibility,
    selected_columns: Option<&HashSet<String>>,
) -> Value {
    let Some(object) = record.as_object() else {
        return record.clone();
    };

    let allowed = allowed_columns(columns, mode);
    let filtered = object
        .iter()
        .filter(|(key, _)| allowed.contains(*key))
        .filter(|(key, _)| match selected_columns {
            Some(selected) => selected.contains(*key),
            None => true,
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

    Value::Object(filtered)
}

fn changed_columns_for_create(
    record: &Value,
    columns: &[crate::domain::entity::column_definition::ColumnDefinition],
) -> Vec<String> {
    let Some(object) = record.as_object() else {
        return Vec::new();
    };

    columns
        .iter()
        .filter_map(|column| {
            object
                .get(&column.column_name)
                .filter(|value| !value.is_null())
                .map(|_| column.column_name.clone())
        })
        .collect()
}

fn changed_columns_for_update(
    before: Option<&Value>,
    after: &Value,
    columns: &[crate::domain::entity::column_definition::ColumnDefinition],
) -> Vec<String> {
    let before_object = before.and_then(Value::as_object);
    let after_object = after.as_object();

    columns
        .iter()
        .filter_map(|column| {
            let before_value = before_object.and_then(|value| value.get(&column.column_name));
            let after_value = after_object.and_then(|value| value.get(&column.column_name));
            if before_value != after_value {
                Some(column.column_name.clone())
            } else {
                None
            }
        })
        .collect()
}

fn changed_columns_for_delete(
    before: Option<&Value>,
    columns: &[crate::domain::entity::column_definition::ColumnDefinition],
) -> Vec<String> {
    let Some(object) = before.and_then(Value::as_object) else {
        return Vec::new();
    };

    columns
        .iter()
        .filter_map(|column| {
            object
                .get(&column.column_name)
                .filter(|value| !value.is_null())
                .map(|_| column.column_name.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::column_definition::ColumnDefinition;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_column(
        column_name: &str,
        is_primary_key: bool,
        is_visible_in_list: bool,
        is_visible_in_form: bool,
    ) -> ColumnDefinition {
        ColumnDefinition {
            id: Uuid::new_v4(),
            table_id: Uuid::new_v4(),
            column_name: column_name.to_string(),
            display_name: column_name.to_string(),
            data_type: "text".to_string(),
            is_primary_key,
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
            is_visible_in_list,
            is_visible_in_form,
            is_readonly: false,
            input_type: "text".to_string(),
            select_options: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn list_visibility_filters_out_hidden_columns() {
        let record = serde_json::json!({
            "id": "dept-1",
            "name": "Platform",
            "secret_code": "hidden",
        });
        let columns = vec![
            sample_column("id", true, true, false),
            sample_column("name", false, true, true),
            sample_column("secret_code", false, false, false),
        ];

        let filtered = filter_record_by_mode(&record, &columns, RecordVisibility::List, None);

        assert_eq!(
            filtered,
            serde_json::json!({
                "id": "dept-1",
                "name": "Platform",
            })
        );
    }

    #[test]
    fn form_visibility_keeps_primary_key_and_selected_columns() {
        let record = serde_json::json!({
            "id": "dept-1",
            "name": "Platform",
            "secret_code": "hidden",
        });
        let columns = vec![
            sample_column("id", true, true, false),
            sample_column("name", false, true, true),
            sample_column("secret_code", false, false, false),
        ];
        let selected = HashSet::from([String::from("id")]);

        let filtered =
            filter_record_by_mode(&record, &columns, RecordVisibility::Form, Some(&selected));

        assert_eq!(filtered, serde_json::json!({ "id": "dept-1" }));
    }

    #[test]
    fn changed_columns_for_update_only_returns_modified_fields() {
        let before = serde_json::json!({
            "id": "dept-1",
            "name": "Platform",
            "status": "active",
        });
        let after = serde_json::json!({
            "id": "dept-1",
            "name": "Data",
            "status": "active",
        });
        let columns = vec![
            sample_column("id", true, true, false),
            sample_column("name", false, true, true),
            sample_column("status", false, true, true),
        ];

        let changed = changed_columns_for_update(Some(&before), &after, &columns);

        assert_eq!(changed, vec![String::from("name")]);
    }
}
