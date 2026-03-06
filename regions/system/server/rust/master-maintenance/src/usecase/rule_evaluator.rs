use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::entity::consistency_rule::ConsistencyRule;
use crate::domain::entity::rule_condition::RuleCondition;
use crate::domain::entity::table_definition::TableDefinition;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::service::rule_engine_service::RuleEngineService;
use crate::domain::value_object::rule_result::RuleResult;
use serde_json::Value;
use std::sync::Arc;

pub struct RuleEvaluator {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    rule_repo: Arc<dyn ConsistencyRuleRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    rule_engine: Arc<dyn RuleEngineService>,
}

impl RuleEvaluator {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        rule_repo: Arc<dyn ConsistencyRuleRepository>,
        record_repo: Arc<dyn DynamicRecordRepository>,
        rule_engine: Arc<dyn RuleEngineService>,
    ) -> Self {
        Self {
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            rule_engine,
        }
    }

    pub async fn evaluate_rule(
        &self,
        rule: &ConsistencyRule,
        source_table: &TableDefinition,
        source_columns: &[ColumnDefinition],
        record: &Value,
    ) -> anyhow::Result<RuleResult> {
        match rule.rule_type.as_str() {
            "custom" => self.rule_engine.evaluate_rule(rule, record).await,
            "range" | "conditional" => self.evaluate_record_conditions(rule, record).await,
            "uniqueness" => {
                self.evaluate_uniqueness(rule, source_table, source_columns, record)
                    .await
            }
            "cross_table" => self.evaluate_cross_table(rule, record).await,
            _ => Ok(RuleResult::pass()),
        }
    }

    async fn evaluate_record_conditions(
        &self,
        rule: &ConsistencyRule,
        record: &Value,
    ) -> anyhow::Result<RuleResult> {
        let conditions = self.rule_repo.find_conditions_by_rule_id(rule.id).await?;
        let passed = evaluate_conditions(&conditions, record, None);
        Ok(build_result(rule, passed, Vec::new(), record))
    }

    async fn evaluate_uniqueness(
        &self,
        rule: &ConsistencyRule,
        source_table: &TableDefinition,
        source_columns: &[ColumnDefinition],
        record: &Value,
    ) -> anyhow::Result<RuleResult> {
        let conditions = self.rule_repo.find_conditions_by_rule_id(rule.id).await?;
        let (records, _) = self
            .record_repo
            .find_all(source_table, source_columns, 1, 10_000, None, None, None)
            .await?;
        let source_pk = primary_key_column(source_columns);
        let current_pk = source_pk
            .and_then(|pk| record.get(pk))
            .map(scalar_string)
            .filter(|value| !value.is_empty());

        let duplicates: Vec<String> = records
            .into_iter()
            .filter(|candidate| {
                let candidate_pk = source_pk
                    .and_then(|pk| candidate.get(pk))
                    .map(scalar_string)
                    .filter(|value| !value.is_empty());
                if current_pk.is_some() && candidate_pk == current_pk {
                    return false;
                }
                evaluate_conditions(&conditions, record, Some(candidate))
            })
            .filter_map(|candidate| {
                source_pk
                    .and_then(|pk| candidate.get(pk))
                    .map(scalar_string)
                    .filter(|value| !value.is_empty())
            })
            .collect();

        Ok(build_result(
            rule,
            duplicates.is_empty(),
            duplicates,
            record,
        ))
    }

    async fn evaluate_cross_table(
        &self,
        rule: &ConsistencyRule,
        record: &Value,
    ) -> anyhow::Result<RuleResult> {
        let conditions = self.rule_repo.find_conditions_by_rule_id(rule.id).await?;
        let target_table_id = conditions
            .iter()
            .find_map(|condition| condition.right_table_id)
            .ok_or_else(|| anyhow::anyhow!("cross_table rule requires right_table_id"))?;
        let target_table = self
            .table_repo
            .find_by_id(target_table_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("target table not found"))?;
        let target_columns = self.column_repo.find_by_table_id(target_table.id).await?;
        let (records, _) = self
            .record_repo
            .find_all(&target_table, &target_columns, 1, 10_000, None, None, None)
            .await?;
        let target_pk = primary_key_column(&target_columns);

        let matches: Vec<String> = records
            .iter()
            .filter(|candidate| evaluate_conditions(&conditions, record, Some(candidate)))
            .filter_map(|candidate| {
                target_pk
                    .and_then(|pk| candidate.get(pk))
                    .map(scalar_string)
                    .filter(|value| !value.is_empty())
            })
            .collect();

        Ok(build_result(rule, !matches.is_empty(), matches, record))
    }
}

fn build_result(
    rule: &ConsistencyRule,
    passed: bool,
    affected_record_ids: Vec<String>,
    record: &Value,
) -> RuleResult {
    if passed {
        RuleResult {
            rule_id: String::new(),
            rule_name: String::new(),
            passed: true,
            message: None,
            severity: rule.severity.clone(),
            affected_record_ids,
        }
    } else if rule.severity == "warning" {
        RuleResult {
            rule_id: String::new(),
            rule_name: String::new(),
            passed: true,
            message: Some(render_message(&rule.error_message_template, record)),
            severity: rule.severity.clone(),
            affected_record_ids,
        }
    } else {
        RuleResult {
            rule_id: String::new(),
            rule_name: String::new(),
            passed: false,
            message: Some(render_message(&rule.error_message_template, record)),
            severity: rule.severity.clone(),
            affected_record_ids,
        }
    }
}

fn evaluate_conditions(
    conditions: &[RuleCondition],
    current_record: &Value,
    candidate_record: Option<&Value>,
) -> bool {
    let mut iter = conditions.iter();
    let Some(first) = iter.next() else {
        return true;
    };

    let mut result = evaluate_condition(first, current_record, candidate_record);
    for condition in iter {
        let current = evaluate_condition(condition, current_record, candidate_record);
        match condition.logical_connector.as_deref().unwrap_or("AND") {
            "OR" => result = result || current,
            _ => result = result && current,
        }
    }
    result
}

fn evaluate_condition(
    condition: &RuleCondition,
    current_record: &Value,
    candidate_record: Option<&Value>,
) -> bool {
    let left = current_record
        .get(&condition.left_column)
        .unwrap_or(&Value::Null);
    let right = resolve_right_value(condition, current_record, candidate_record);

    match condition.operator.as_str() {
        "eq" => compare_eq(left, &right),
        "neq" => !compare_eq(left, &right),
        "gt" => compare_numeric(left, &right, |lhs, rhs| lhs > rhs),
        "gte" => compare_numeric(left, &right, |lhs, rhs| lhs >= rhs),
        "lt" => compare_numeric(left, &right, |lhs, rhs| lhs < rhs),
        "lte" => compare_numeric(left, &right, |lhs, rhs| lhs <= rhs),
        "regex" => right
            .as_str()
            .and_then(|pattern| regex::Regex::new(pattern).ok())
            .and_then(|regex| left.as_str().map(|value| regex.is_match(value)))
            .unwrap_or(false),
        "exists" => !right.is_null(),
        "not_exists" => right.is_null(),
        "in" => right
            .as_array()
            .map(|items| items.iter().any(|item| compare_eq(left, item)))
            .unwrap_or(false),
        "not_in" => right
            .as_array()
            .map(|items| items.iter().all(|item| !compare_eq(left, item)))
            .unwrap_or(false),
        "between" => right
            .as_array()
            .filter(|items| items.len() == 2)
            .map(|items| {
                compare_numeric(left, &items[0], |lhs, rhs| lhs >= rhs)
                    && compare_numeric(left, &items[1], |lhs, rhs| lhs <= rhs)
            })
            .unwrap_or(false),
        _ => false,
    }
}

fn resolve_right_value(
    condition: &RuleCondition,
    current_record: &Value,
    candidate_record: Option<&Value>,
) -> Value {
    if let Some(column) = condition.right_column.as_deref() {
        if let Some(candidate_record) = candidate_record {
            return candidate_record.get(column).cloned().unwrap_or(Value::Null);
        }
        return current_record.get(column).cloned().unwrap_or(Value::Null);
    }

    if let Some(raw_value) = condition.right_value.as_deref() {
        return parse_right_value(raw_value, current_record);
    }

    Value::Null
}

fn parse_right_value(raw_value: &str, current_record: &Value) -> Value {
    if let Some(field) = raw_value
        .strip_prefix("{current.")
        .and_then(|value| value.strip_suffix('}'))
    {
        return current_record.get(field).cloned().unwrap_or(Value::Null);
    }

    serde_json::from_str(raw_value).unwrap_or_else(|_| Value::String(raw_value.to_string()))
}

fn compare_eq(left: &Value, right: &Value) -> bool {
    if left == right {
        return true;
    }
    scalar_string(left) == scalar_string(right)
}

fn compare_numeric(left: &Value, right: &Value, predicate: impl Fn(f64, f64) -> bool) -> bool {
    let Some(lhs) = as_f64(left) else {
        return false;
    };
    let Some(rhs) = as_f64(right) else {
        return false;
    };
    predicate(lhs, rhs)
}

fn as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(string) => string.parse::<f64>().ok(),
        _ => None,
    }
}

fn scalar_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        other => other.to_string(),
    }
}

fn primary_key_column(columns: &[ColumnDefinition]) -> Option<&str> {
    columns
        .iter()
        .find(|column| column.is_primary_key)
        .map(|column| column.column_name.as_str())
}

fn render_message(template: &str, record: &Value) -> String {
    let mut rendered = template.to_string();
    if let Some(object) = record.as_object() {
        for (key, value) in object {
            rendered = rendered.replace(&format!("{{{}}}", key), &scalar_string(value));
            rendered = rendered.replace(&format!("{{current.{}}}", key), &scalar_string(value));
        }
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn condition(
        operator: &str,
        right_value: Option<&str>,
        right_column: Option<&str>,
    ) -> RuleCondition {
        RuleCondition {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            condition_order: 1,
            left_table_id: Uuid::new_v4(),
            left_column: "amount".to_string(),
            operator: operator.to_string(),
            right_table_id: None,
            right_column: right_column.map(ToString::to_string),
            right_value: right_value.map(ToString::to_string),
            logical_connector: Some("AND".to_string()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn evaluates_numeric_condition() {
        let current = serde_json::json!({ "amount": 12 });
        assert!(evaluate_condition(
            &condition("gte", Some("10"), None),
            &current,
            None
        ));
        assert!(!evaluate_condition(
            &condition("lt", Some("10"), None),
            &current,
            None
        ));
    }

    #[test]
    fn resolves_current_placeholders() {
        let current = serde_json::json!({ "amount": 12, "limit": 10 });
        assert!(!evaluate_condition(
            &condition("lte", Some("{current.limit}"), None),
            &current,
            None
        ));
    }

    #[test]
    fn renders_message_template() {
        let current = serde_json::json!({ "code": "ABC001" });
        assert_eq!(render_message("invalid {code}", &current), "invalid ABC001");
    }
}
