//! レコードの CRUD 操作を担当する usecase。
//!
//! テーブル定義に基づく動的レコードの作成・読み取り・更新・削除を行う。
//! バリデーションルール評価と変更ログの記録も含む。

use crate::domain::entity::change_log::ChangeLog;
use crate::domain::error::MasterMaintenanceError;
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

/// レコード変更操作の出力型。変更後のレコードとバリデーション警告を含む。
#[derive(Debug, Clone)]
pub struct RecordMutationOutput {
    pub record: Value,
    pub warnings: Vec<RuleResult>,
}

/// レコードバリデーションエラー。ルール評価で失敗した errors と警告 warnings を保持する。
/// MasterMaintenanceError::RecordValidation に内包されて使用される。
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

/// レコード CRUD 操作を提供する usecase 構造体。
pub struct CrudRecordsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    rule_repo: Arc<dyn ConsistencyRuleRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    change_log_repo: Arc<dyn ChangeLogRepository>,
    rule_evaluator: RuleEvaluator,
}

/// レコード一覧取得の出力型。テーブルメタデータとページングされたレコードリストを含む。
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
    /// 依存リポジトリと rule_engine を注入して usecase を構築する。
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

    /// 保存前ルールを評価して RuleResult の一覧を返す。
    async fn evaluate_before_save_rules(
        &self,
        table: &crate::domain::entity::table_definition::TableDefinition,
        columns: &[crate::domain::entity::column_definition::ColumnDefinition],
        data: &Value,
    ) -> Result<Vec<RuleResult>, MasterMaintenanceError> {
        // before_save タイミングのルールのみを評価対象とする
        let rules = self
            .rule_repo
            .find_by_table_id(table.id, Some("before_save"))
            .await
            .map_err(MasterMaintenanceError::from)?;

        let mut results = Vec::new();
        for rule in rules.into_iter().filter(|rule| rule.is_active) {
            let result = self
                .rule_evaluator
                .evaluate_rule(&rule, table, columns, data)
                .await
                .map_err(MasterMaintenanceError::from)?
                .with_rule_info(rule.id.to_string(), rule.name.clone());
            results.push(result);
        }

        Ok(results)
    }

    /// ルール評価結果を検査し、エラーがあれば RecordValidation エラーを返す。
    /// エラーがない場合は警告リストを返す。
    fn fail_on_rule_errors(
        results: &[RuleResult],
    ) -> Result<Vec<RuleResult>, MasterMaintenanceError> {
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
            // 型安全なバリアントで RecordValidationError を内包する（C-04対応）
            Err(MasterMaintenanceError::RecordValidation(Box::new(
                RecordValidationError { errors, warnings },
            )))
        }
    }

    /// レコード一覧を取得する。selected_columns で返却カラムを絞り込める。
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
    ) -> Result<ListRecordsOutput, MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(table_name.to_string()))?;
        let column_defs = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        let (mut records, total) = self
            .record_repo
            .find_all(&table, &column_defs, page, page_size, sort, filter, search)
            .await
            .map_err(MasterMaintenanceError::from)?;

        // selected_columns が指定されている場合は指定カラムのみに絞り込む
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

    /// 単一レコードを ID で取得する。
    pub async fn get_record(
        &self,
        table_name: &str,
        record_id: &str,
        domain_scope: Option<&str>,
    ) -> Result<Option<Value>, MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(table_name.to_string()))?;
        let columns = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        self.record_repo
            .find_by_id(&table, &columns, record_id)
            .await
            .map(|record| {
                record.map(|value| {
                    filter_record_by_mode(&value, &columns, RecordVisibility::Form, None)
                })
            })
            .map_err(MasterMaintenanceError::from)
    }

    /// テーブルの操作許可フラグ（create/update/delete）を返す。
    pub async fn table_permissions(
        &self,
        table_name: &str,
        domain_scope: Option<&str>,
    ) -> Result<(bool, bool, bool), MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(table_name.to_string()))?;
        Ok((table.allow_create, table.allow_update, table.allow_delete))
    }

    /// レコードを新規作成する。バリデーションルールを評価してから保存する。
    pub async fn create_record(
        &self,
        table_name: &str,
        data: &Value,
        created_by: &str,
        domain_scope: Option<&str>,
        trace_id: Option<String>,
    ) -> Result<RecordMutationOutput, MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(table_name.to_string()))?;
        // create が許可されていない場合は OperationNotAllowed を返す
        if !table.allow_create {
            return Err(MasterMaintenanceError::OperationNotAllowed {
                table_name: table_name.to_string(),
                operation: "Create".to_string(),
            });
        }
        let columns = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        let warnings = Self::fail_on_rule_errors(
            &self
                .evaluate_before_save_rules(&table, &columns, data)
                .await?,
        )?;
        let record = self
            .record_repo
            .create(&table, &columns, data)
            .await
            .map_err(MasterMaintenanceError::from)?;

        // 変更ログを非同期で記録する（失敗しても無視する）
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

    /// レコードを更新する。バリデーションルールを評価してから保存する。
    pub async fn update_record(
        &self,
        table_name: &str,
        record_id: &str,
        data: &Value,
        updated_by: &str,
        domain_scope: Option<&str>,
        trace_id: Option<String>,
    ) -> Result<RecordMutationOutput, MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(table_name.to_string()))?;
        // update が許可されていない場合は OperationNotAllowed を返す
        if !table.allow_update {
            return Err(MasterMaintenanceError::OperationNotAllowed {
                table_name: table_name.to_string(),
                operation: "Update".to_string(),
            });
        }
        let columns = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        let warnings = Self::fail_on_rule_errors(
            &self
                .evaluate_before_save_rules(&table, &columns, data)
                .await?,
        )?;

        let before = self
            .record_repo
            .find_by_id(&table, &columns, record_id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        let record = self
            .record_repo
            .update(&table, &columns, record_id, data)
            .await
            .map_err(MasterMaintenanceError::from)?;

        // 変更ログを非同期で記録する（失敗しても無視する）
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

    /// レコードを削除する。delete が許可されていない場合はエラーを返す。
    pub async fn delete_record(
        &self,
        table_name: &str,
        record_id: &str,
        deleted_by: &str,
        domain_scope: Option<&str>,
        trace_id: Option<String>,
    ) -> Result<(), MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(table_name.to_string()))?;
        // delete が許可されていない場合は OperationNotAllowed を返す
        if !table.allow_delete {
            return Err(MasterMaintenanceError::OperationNotAllowed {
                table_name: table_name.to_string(),
                operation: "Delete".to_string(),
            });
        }
        let columns = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(MasterMaintenanceError::from)?;

        let before = self
            .record_repo
            .find_by_id(&table, &columns, record_id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        self.record_repo
            .delete(&table, record_id)
            .await
            .map_err(MasterMaintenanceError::from)?;

        // 変更ログを非同期で記録する（失敗しても無視する）
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

/// レコードの表示モード。一覧表示用と詳細フォーム用で表示カラムが異なる。
#[derive(Copy, Clone)]
enum RecordVisibility {
    List,
    Form,
}

/// 表示モードに基づいて許可カラム名のセットを返す。
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

/// レコードを表示モードと selected_columns でフィルタリングした新しい Value を返す。
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

/// 新規作成時に値が null でないカラム名のリストを返す。変更ログに記録する。
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

/// 更新時に before と after で値が異なるカラム名のリストを返す。変更ログに記録する。
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

/// 削除時に値が null でないカラム名のリストを返す。変更ログに記録する。
fn changed_columns_for_delete(
    before: Option<&Value>,
    columns: &[crate::domain::entity::column_definition::ColumnDefinition],
) -> Vec<String> {
    let Some(before_object) = before.and_then(Value::as_object) else {
        return Vec::new();
    };

    columns
        .iter()
        .filter_map(|column| {
            before_object
                .get(&column.column_name)
                .filter(|value| !value.is_null())
                .map(|_| column.column_name.clone())
        })
        .collect()
}
