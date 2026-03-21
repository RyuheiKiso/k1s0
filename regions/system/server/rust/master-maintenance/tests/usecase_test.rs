#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_master_maintenance_server::domain::entity::change_log::ChangeLog;
use k1s0_master_maintenance_server::domain::entity::column_definition::{
    ColumnDefinition, CreateColumnDefinition,
};
use k1s0_master_maintenance_server::domain::entity::consistency_rule::ConsistencyRule;
use k1s0_master_maintenance_server::domain::entity::display_config::DisplayConfig;
use k1s0_master_maintenance_server::domain::entity::rule_condition::RuleCondition;
use k1s0_master_maintenance_server::domain::entity::table_definition::{
    CreateTableDefinition, TableDefinition, UpdateTableDefinition,
};
use k1s0_master_maintenance_server::domain::repository::change_log_repository::ChangeLogRepository;
use k1s0_master_maintenance_server::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use k1s0_master_maintenance_server::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;
use k1s0_master_maintenance_server::domain::repository::display_config_repository::DisplayConfigRepository;
use k1s0_master_maintenance_server::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use k1s0_master_maintenance_server::domain::repository::table_definition_repository::TableDefinitionRepository;
use k1s0_master_maintenance_server::domain::service::rule_engine_service::RuleEngineService;
use k1s0_master_maintenance_server::domain::value_object::domain_filter::DomainFilter;
use k1s0_master_maintenance_server::domain::value_object::rule_result::RuleResult;
use k1s0_master_maintenance_server::domain::entity::import_job::ImportJob;
use k1s0_master_maintenance_server::domain::entity::table_relationship::TableRelationship;
use k1s0_master_maintenance_server::domain::repository::import_job_repository::ImportJobRepository;
use k1s0_master_maintenance_server::domain::repository::table_relationship_repository::TableRelationshipRepository;
use k1s0_master_maintenance_server::domain::value_object::relationship_type::RelationshipType;
use k1s0_master_maintenance_server::infrastructure::schema::SchemaManager;

// ---------------------------------------------------------------------------
// In-memory stub: TableDefinitionRepository
// ---------------------------------------------------------------------------

struct StubTableDefinitionRepository {
    tables: RwLock<Vec<TableDefinition>>,
    should_fail: bool,
}

impl StubTableDefinitionRepository {
    fn new() -> Self {
        Self {
            tables: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_tables(tables: Vec<TableDefinition>) -> Self {
        Self {
            tables: RwLock::new(tables),
            should_fail: false,
        }
    }

    #[allow(dead_code)]
    fn failing() -> Self {
        Self {
            tables: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl TableDefinitionRepository for StubTableDefinitionRepository {
    async fn find_all(
        &self,
        category: Option<&str>,
        active_only: bool,
        _domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let tables = self.tables.read().await;
        Ok(tables
            .iter()
            .filter(|t| {
                if active_only && !t.is_active {
                    return false;
                }
                if let Some(cat) = category {
                    return t.category.as_deref() == Some(cat);
                }
                true
            })
            .cloned()
            .collect())
    }

    async fn find_by_name(
        &self,
        name: &str,
        _domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<TableDefinition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let tables = self.tables.read().await;
        Ok(tables.iter().find(|t| t.name == name).cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let tables = self.tables.read().await;
        Ok(tables.iter().find(|t| t.id == id).cloned())
    }

    async fn create(
        &self,
        input: &CreateTableDefinition,
        created_by: &str,
    ) -> anyhow::Result<TableDefinition> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let now = Utc::now();
        let table = TableDefinition {
            id: Uuid::new_v4(),
            name: input.name.clone(),
            schema_name: input.schema_name.clone(),
            database_name: input.database_name.clone().unwrap_or_default(),
            display_name: input.display_name.clone(),
            description: input.description.clone(),
            category: input.category.clone(),
            is_active: true,
            allow_create: input.allow_create.unwrap_or(true),
            allow_update: input.allow_update.unwrap_or(true),
            allow_delete: input.allow_delete.unwrap_or(true),
            read_roles: input.read_roles.clone().unwrap_or_default(),
            write_roles: input.write_roles.clone().unwrap_or_default(),
            admin_roles: input.admin_roles.clone().unwrap_or_default(),
            sort_order: input.sort_order.unwrap_or(0),
            created_by: created_by.to_string(),
            created_at: now,
            updated_at: now,
            domain_scope: input.domain_scope.clone(),
        };
        self.tables.write().await.push(table.clone());
        Ok(table)
    }

    async fn update(
        &self,
        name: &str,
        input: &UpdateTableDefinition,
        _domain_scope: Option<&str>,
    ) -> anyhow::Result<TableDefinition> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut tables = self.tables.write().await;
        let table = tables
            .iter_mut()
            .find(|t| t.name == name)
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        if let Some(ref display_name) = input.display_name {
            table.display_name = display_name.clone();
        }
        if let Some(is_active) = input.is_active {
            table.is_active = is_active;
        }
        if let Some(allow_create) = input.allow_create {
            table.allow_create = allow_create;
        }
        if let Some(allow_update) = input.allow_update {
            table.allow_update = allow_update;
        }
        if let Some(allow_delete) = input.allow_delete {
            table.allow_delete = allow_delete;
        }
        table.updated_at = Utc::now();
        Ok(table.clone())
    }

    async fn delete(&self, name: &str, _domain_scope: Option<&str>) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut tables = self.tables.write().await;
        tables.retain(|t| t.name != name);
        Ok(())
    }

    async fn find_domains(&self) -> anyhow::Result<Vec<(String, i64)>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        Ok(vec![("default".to_string(), 1)])
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: ColumnDefinitionRepository
// ---------------------------------------------------------------------------

struct StubColumnDefinitionRepository {
    columns: RwLock<Vec<ColumnDefinition>>,
    should_fail: bool,
}

impl StubColumnDefinitionRepository {
    fn new() -> Self {
        Self {
            columns: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_columns(columns: Vec<ColumnDefinition>) -> Self {
        Self {
            columns: RwLock::new(columns),
            should_fail: false,
        }
    }
}

#[async_trait]
impl ColumnDefinitionRepository for StubColumnDefinitionRepository {
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<ColumnDefinition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let columns = self.columns.read().await;
        Ok(columns
            .iter()
            .filter(|c| c.table_id == table_id)
            .cloned()
            .collect())
    }

    async fn find_by_table_and_column(
        &self,
        table_id: Uuid,
        column_name: &str,
    ) -> anyhow::Result<Option<ColumnDefinition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let columns = self.columns.read().await;
        Ok(columns
            .iter()
            .find(|c| c.table_id == table_id && c.column_name == column_name)
            .cloned())
    }

    async fn create_batch(
        &self,
        table_id: Uuid,
        inputs: &[CreateColumnDefinition],
    ) -> anyhow::Result<Vec<ColumnDefinition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let now = Utc::now();
        let mut created = Vec::new();
        let mut columns = self.columns.write().await;
        for input in inputs {
            let col = make_column_from_input(table_id, input, now);
            columns.push(col.clone());
            created.push(col);
        }
        Ok(created)
    }

    async fn update(
        &self,
        table_id: Uuid,
        column_name: &str,
        _input: &CreateColumnDefinition,
    ) -> anyhow::Result<ColumnDefinition> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let columns = self.columns.read().await;
        columns
            .iter()
            .find(|c| c.table_id == table_id && c.column_name == column_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Column not found"))
    }

    async fn delete(&self, table_id: Uuid, column_name: &str) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut columns = self.columns.write().await;
        columns.retain(|c| !(c.table_id == table_id && c.column_name == column_name));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: ChangeLogRepository
// ---------------------------------------------------------------------------

struct StubChangeLogRepository {
    logs: RwLock<Vec<ChangeLog>>,
    should_fail: bool,
}

impl StubChangeLogRepository {
    fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_logs(logs: Vec<ChangeLog>) -> Self {
        Self {
            logs: RwLock::new(logs),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl ChangeLogRepository for StubChangeLogRepository {
    async fn create(&self, log: &ChangeLog) -> anyhow::Result<ChangeLog> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut logs = self.logs.write().await;
        logs.push(log.clone());
        Ok(log.clone())
    }

    async fn find_by_table(
        &self,
        table_name: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let logs = self.logs.read().await;
        let filtered: Vec<ChangeLog> = logs
            .iter()
            .filter(|l| l.target_table == table_name)
            .cloned()
            .collect();
        let total = filtered.len() as i64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let items = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((items, total))
    }

    async fn find_by_record(
        &self,
        table_name: &str,
        record_id: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let logs = self.logs.read().await;
        let filtered: Vec<ChangeLog> = logs
            .iter()
            .filter(|l| l.target_table == table_name && l.target_record_id == record_id)
            .cloned()
            .collect();
        let total = filtered.len() as i64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let items = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((items, total))
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: DynamicRecordRepository
// ---------------------------------------------------------------------------

struct StubDynamicRecordRepository {
    records: RwLock<Vec<(String, Value)>>,
    should_fail: bool,
}

impl StubDynamicRecordRepository {
    fn new() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    #[allow(dead_code)]
    fn with_records(table_name: &str, records: Vec<Value>) -> Self {
        let items = records
            .into_iter()
            .map(|r| (table_name.to_string(), r))
            .collect();
        Self {
            records: RwLock::new(items),
            should_fail: false,
        }
    }
}

#[async_trait]
impl DynamicRecordRepository for StubDynamicRecordRepository {
    async fn find_all(
        &self,
        table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        page: i32,
        page_size: i32,
        _sort: Option<&str>,
        _filter: Option<&str>,
        _search: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let records = self.records.read().await;
        let filtered: Vec<Value> = records
            .iter()
            .filter(|(name, _)| *name == table_def.name)
            .map(|(_, r)| r.clone())
            .collect();
        let total = filtered.len() as i64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let items = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((items, total))
    }

    async fn find_by_id(
        &self,
        table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        record_id: &str,
    ) -> anyhow::Result<Option<Value>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let records = self.records.read().await;
        Ok(records
            .iter()
            .filter(|(name, _)| *name == table_def.name)
            .find(|(_, r)| r.get("id").and_then(|v| v.as_str()) == Some(record_id))
            .map(|(_, r)| r.clone()))
    }

    async fn create(
        &self,
        table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        data: &Value,
    ) -> anyhow::Result<Value> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut record = data.clone();
        if record.get("id").is_none() {
            if let Some(obj) = record.as_object_mut() {
                obj.insert("id".to_string(), Value::String(Uuid::new_v4().to_string()));
            }
        }
        self.records
            .write()
            .await
            .push((table_def.name.clone(), record.clone()));
        Ok(record)
    }

    async fn update(
        &self,
        table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        record_id: &str,
        data: &Value,
    ) -> anyhow::Result<Value> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut records = self.records.write().await;
        if let Some((_, existing)) = records
            .iter_mut()
            .filter(|(name, _)| *name == table_def.name)
            .find(|(_, r)| r.get("id").and_then(|v| v.as_str()) == Some(record_id))
        {
            if let (Some(existing_obj), Some(new_obj)) =
                (existing.as_object_mut(), data.as_object())
            {
                for (k, v) in new_obj {
                    existing_obj.insert(k.clone(), v.clone());
                }
            }
            Ok(existing.clone())
        } else {
            anyhow::bail!("Record not found")
        }
    }

    async fn delete(&self, table_def: &TableDefinition, record_id: &str) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut records = self.records.write().await;
        records.retain(|(name, r)| {
            !(*name == table_def.name && r.get("id").and_then(|v| v.as_str()) == Some(record_id))
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: ConsistencyRuleRepository
// ---------------------------------------------------------------------------

struct StubConsistencyRuleRepository {
    rules: RwLock<Vec<ConsistencyRule>>,
    conditions: RwLock<Vec<RuleCondition>>,
    should_fail: bool,
}

impl StubConsistencyRuleRepository {
    fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            conditions: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_rules(rules: Vec<ConsistencyRule>) -> Self {
        Self {
            rules: RwLock::new(rules),
            conditions: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    #[allow(dead_code)]
    fn failing() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            conditions: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl ConsistencyRuleRepository for StubConsistencyRuleRepository {
    async fn find_all(
        &self,
        table_id: Option<Uuid>,
        rule_type: Option<&str>,
        severity: Option<&str>,
    ) -> anyhow::Result<Vec<ConsistencyRule>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let rules = self.rules.read().await;
        Ok(rules
            .iter()
            .filter(|r| {
                if let Some(tid) = table_id {
                    if r.source_table_id != tid {
                        return false;
                    }
                }
                if let Some(rt) = rule_type {
                    if r.rule_type != rt {
                        return false;
                    }
                }
                if let Some(sev) = severity {
                    if r.severity != sev {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ConsistencyRule>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let rules = self.rules.read().await;
        Ok(rules.iter().find(|r| r.id == id).cloned())
    }

    async fn find_by_table_id(
        &self,
        table_id: Uuid,
        timing: Option<&str>,
    ) -> anyhow::Result<Vec<ConsistencyRule>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let rules = self.rules.read().await;
        Ok(rules
            .iter()
            .filter(|r| {
                r.source_table_id == table_id
                    && timing.map(|t| r.evaluation_timing == t).unwrap_or(true)
            })
            .cloned()
            .collect())
    }

    async fn create(
        &self,
        rule: &ConsistencyRule,
        conditions: &[RuleCondition],
    ) -> anyhow::Result<ConsistencyRule> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        self.rules.write().await.push(rule.clone());
        self.conditions
            .write()
            .await
            .extend(conditions.iter().cloned());
        Ok(rule.clone())
    }

    async fn update(&self, id: Uuid, rule: &ConsistencyRule) -> anyhow::Result<ConsistencyRule> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.id == id) {
            *existing = rule.clone();
        }
        Ok(rule.clone())
    }

    async fn replace_conditions(
        &self,
        rule_id: Uuid,
        conditions: &[RuleCondition],
    ) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut conds = self.conditions.write().await;
        conds.retain(|c| c.rule_id != rule_id);
        conds.extend(conditions.iter().cloned());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.id != id);
        let mut conds = self.conditions.write().await;
        conds.retain(|c| c.rule_id != id);
        Ok(())
    }

    async fn find_conditions_by_rule_id(
        &self,
        rule_id: Uuid,
    ) -> anyhow::Result<Vec<RuleCondition>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let conds = self.conditions.read().await;
        Ok(conds
            .iter()
            .filter(|c| c.rule_id == rule_id)
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: DisplayConfigRepository
// ---------------------------------------------------------------------------

struct StubDisplayConfigRepository {
    configs: RwLock<Vec<DisplayConfig>>,
    should_fail: bool,
}

impl StubDisplayConfigRepository {
    fn new() -> Self {
        Self {
            configs: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_configs(configs: Vec<DisplayConfig>) -> Self {
        Self {
            configs: RwLock::new(configs),
            should_fail: false,
        }
    }

    #[allow(dead_code)]
    fn failing() -> Self {
        Self {
            configs: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl DisplayConfigRepository for StubDisplayConfigRepository {
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<DisplayConfig>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let configs = self.configs.read().await;
        Ok(configs
            .iter()
            .filter(|c| c.table_id == table_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DisplayConfig>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let configs = self.configs.read().await;
        Ok(configs.iter().find(|c| c.id == id).cloned())
    }

    async fn create(&self, config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        self.configs.write().await.push(config.clone());
        Ok(config.clone())
    }

    async fn update(&self, id: Uuid, config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut configs = self.configs.write().await;
        if let Some(existing) = configs.iter_mut().find(|c| c.id == id) {
            *existing = config.clone();
        }
        Ok(config.clone())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut configs = self.configs.write().await;
        configs.retain(|c| c.id != id);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: RuleEngineService
// ---------------------------------------------------------------------------

struct StubRuleEngineService {
    should_pass: bool,
}

impl StubRuleEngineService {
    fn passing() -> Self {
        Self { should_pass: true }
    }

    #[allow(dead_code)]
    fn failing_rules() -> Self {
        Self { should_pass: false }
    }
}

#[async_trait]
impl RuleEngineService for StubRuleEngineService {
    async fn evaluate_rule(
        &self,
        rule: &ConsistencyRule,
        _record_data: &Value,
    ) -> anyhow::Result<RuleResult> {
        if self.should_pass {
            Ok(RuleResult::pass())
        } else {
            Ok(RuleResult::fail(rule.error_message_template.clone()))
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: SchemaManager（物理スキーマ操作のno-opスタブ）
// ---------------------------------------------------------------------------

struct StubSchemaManager;

#[async_trait]
impl SchemaManager for StubSchemaManager {
    async fn create_table(
        &self,
        _input: &k1s0_master_maintenance_server::domain::entity::table_definition::CreateTableDefinition,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_table(&self, _table: &TableDefinition) -> anyhow::Result<()> {
        Ok(())
    }

    async fn add_columns(
        &self,
        _table: &TableDefinition,
        _columns: &[CreateColumnDefinition],
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update_column(
        &self,
        _table: &TableDefinition,
        _existing: &ColumnDefinition,
        _input: &CreateColumnDefinition,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_column(
        &self,
        _table: &TableDefinition,
        _column_name: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_relationship(
        &self,
        _source_table: &TableDefinition,
        _target_table: &TableDefinition,
        _relationship: &TableRelationship,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update_relationship(
        &self,
        _source_table: &TableDefinition,
        _target_table: &TableDefinition,
        _relationship: &TableRelationship,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_relationship(
        &self,
        _source_table: &TableDefinition,
        _relationship_id: Uuid,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: TableRelationshipRepository
// ---------------------------------------------------------------------------

struct StubTableRelationshipRepository {
    relationships: RwLock<Vec<TableRelationship>>,
}

impl StubTableRelationshipRepository {
    fn new() -> Self {
        Self {
            relationships: RwLock::new(Vec::new()),
        }
    }

    fn with_relationships(rels: Vec<TableRelationship>) -> Self {
        Self {
            relationships: RwLock::new(rels),
        }
    }
}

#[async_trait]
impl TableRelationshipRepository for StubTableRelationshipRepository {
    async fn find_all(&self) -> anyhow::Result<Vec<TableRelationship>> {
        Ok(self.relationships.read().await.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableRelationship>> {
        Ok(self
            .relationships
            .read()
            .await
            .iter()
            .find(|r| r.id == id)
            .cloned())
    }

    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<TableRelationship>> {
        Ok(self
            .relationships
            .read()
            .await
            .iter()
            .filter(|r| r.source_table_id == table_id || r.target_table_id == table_id)
            .cloned()
            .collect())
    }

    async fn create(&self, relationship: &TableRelationship) -> anyhow::Result<TableRelationship> {
        self.relationships.write().await.push(relationship.clone());
        Ok(relationship.clone())
    }

    async fn update(
        &self,
        id: Uuid,
        relationship: &TableRelationship,
    ) -> anyhow::Result<TableRelationship> {
        let mut rels = self.relationships.write().await;
        if let Some(existing) = rels.iter_mut().find(|r| r.id == id) {
            *existing = relationship.clone();
        }
        Ok(relationship.clone())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        self.relationships.write().await.retain(|r| r.id != id);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: ImportJobRepository
// ---------------------------------------------------------------------------

struct StubImportJobRepository {
    jobs: RwLock<Vec<ImportJob>>,
}

impl StubImportJobRepository {
    fn new() -> Self {
        Self {
            jobs: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ImportJobRepository for StubImportJobRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ImportJob>> {
        Ok(self.jobs.read().await.iter().find(|j| j.id == id).cloned())
    }

    async fn create(&self, job: &ImportJob) -> anyhow::Result<ImportJob> {
        self.jobs.write().await.push(job.clone());
        Ok(job.clone())
    }

    async fn update(&self, id: Uuid, job: &ImportJob) -> anyhow::Result<ImportJob> {
        let mut jobs = self.jobs.write().await;
        if let Some(existing) = jobs.iter_mut().find(|j| j.id == id) {
            *existing = job.clone();
        }
        Ok(job.clone())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_table(name: &str) -> TableDefinition {
    let now = Utc::now();
    TableDefinition {
        id: Uuid::new_v4(),
        name: name.to_string(),
        schema_name: "public".to_string(),
        database_name: "main".to_string(),
        display_name: name.to_string(),
        description: Some(format!("{} description", name)),
        category: Some("master".to_string()),
        is_active: true,
        allow_create: true,
        allow_update: true,
        allow_delete: true,
        read_roles: vec!["viewer".to_string()],
        write_roles: vec!["editor".to_string()],
        admin_roles: vec!["admin".to_string()],
        sort_order: 0,
        created_by: "system".to_string(),
        created_at: now,
        updated_at: now,
        domain_scope: None,
    }
}

fn make_table_readonly(name: &str) -> TableDefinition {
    let mut table = make_table(name);
    table.allow_create = false;
    table.allow_update = false;
    table.allow_delete = false;
    table
}

fn make_column(
    table_id: Uuid,
    column_name: &str,
    is_primary_key: bool,
    is_visible_in_list: bool,
    is_visible_in_form: bool,
) -> ColumnDefinition {
    let now = Utc::now();
    ColumnDefinition {
        id: Uuid::new_v4(),
        table_id,
        column_name: column_name.to_string(),
        display_name: column_name.to_string(),
        data_type: "text".to_string(),
        is_primary_key,
        is_nullable: !is_primary_key,
        is_unique: is_primary_key,
        default_value: None,
        max_length: None,
        min_value: None,
        max_value: None,
        regex_pattern: None,
        display_order: 0,
        is_searchable: true,
        is_sortable: true,
        is_filterable: true,
        is_visible_in_list,
        is_visible_in_form,
        is_readonly: false,
        input_type: "text".to_string(),
        select_options: None,
        created_at: now,
        updated_at: now,
    }
}

fn make_column_from_input(
    table_id: Uuid,
    input: &CreateColumnDefinition,
    now: chrono::DateTime<Utc>,
) -> ColumnDefinition {
    ColumnDefinition {
        id: Uuid::new_v4(),
        table_id,
        column_name: input.column_name.clone(),
        display_name: input.display_name.clone(),
        data_type: input.data_type.clone(),
        is_primary_key: input.is_primary_key.unwrap_or(false),
        is_nullable: input.is_nullable.unwrap_or(true),
        is_unique: input.is_unique.unwrap_or(false),
        default_value: input.default_value.clone(),
        max_length: input.max_length,
        min_value: input.min_value,
        max_value: input.max_value,
        regex_pattern: input.regex_pattern.clone(),
        display_order: input.display_order.unwrap_or(0),
        is_searchable: input.is_searchable.unwrap_or(false),
        is_sortable: input.is_sortable.unwrap_or(false),
        is_filterable: input.is_filterable.unwrap_or(false),
        is_visible_in_list: input.is_visible_in_list.unwrap_or(true),
        is_visible_in_form: input.is_visible_in_form.unwrap_or(true),
        is_readonly: input.is_readonly.unwrap_or(false),
        input_type: input.input_type.clone().unwrap_or("text".to_string()),
        select_options: input.select_options.clone(),
        created_at: now,
        updated_at: now,
    }
}

fn make_change_log(table: &str, record_id: &str, operation: &str) -> ChangeLog {
    ChangeLog {
        id: Uuid::new_v4(),
        target_table: table.to_string(),
        target_record_id: record_id.to_string(),
        operation: operation.to_string(),
        before_data: None,
        after_data: Some(serde_json::json!({"id": record_id})),
        changed_columns: Some(vec!["name".to_string()]),
        changed_by: "test-user".to_string(),
        change_reason: None,
        trace_id: None,
        domain_scope: None,
        created_at: Utc::now(),
    }
}

fn make_rule(name: &str, table_id: Uuid, rule_type: &str) -> ConsistencyRule {
    ConsistencyRule {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: Some(format!("{} rule", name)),
        rule_type: rule_type.to_string(),
        severity: "error".to_string(),
        is_active: true,
        source_table_id: table_id,
        evaluation_timing: "before_save".to_string(),
        error_message_template: format!("{} validation failed", name),
        zen_rule_json: None,
        created_by: "admin".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_display_config(table_id: Uuid, config_type: &str) -> DisplayConfig {
    DisplayConfig {
        id: Uuid::new_v4(),
        table_id,
        config_type: config_type.to_string(),
        config_json: serde_json::json!({"columns": ["id", "name"]}),
        is_default: false,
        created_by: "admin".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// ===========================================================================
// GetAuditLogsUseCase tests
// ===========================================================================

mod get_audit_logs {
    use super::*;
    use k1s0_master_maintenance_server::usecase::get_audit_logs::GetAuditLogsUseCase;

    #[tokio::test]
    async fn returns_table_logs() {
        let logs = vec![
            make_change_log("departments", "dept-1", "INSERT"),
            make_change_log("departments", "dept-2", "INSERT"),
            make_change_log("employees", "emp-1", "UPDATE"),
        ];
        let repo = Arc::new(StubChangeLogRepository::with_logs(logs));
        let uc = GetAuditLogsUseCase::new(repo);

        let (result, total) = uc.get_table_logs("departments", 1, 20, None).await.unwrap();
        assert_eq!(total, 2);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|l| l.target_table == "departments"));
    }

    #[tokio::test]
    async fn returns_record_logs() {
        let logs = vec![
            make_change_log("departments", "dept-1", "INSERT"),
            make_change_log("departments", "dept-1", "UPDATE"),
            make_change_log("departments", "dept-2", "INSERT"),
        ];
        let repo = Arc::new(StubChangeLogRepository::with_logs(logs));
        let uc = GetAuditLogsUseCase::new(repo);

        let (result, total) = uc
            .get_record_logs("departments", "dept-1", 1, 20, None)
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|l| l.target_record_id == "dept-1"));
    }

    #[tokio::test]
    async fn returns_empty_when_no_logs() {
        let repo = Arc::new(StubChangeLogRepository::new());
        let uc = GetAuditLogsUseCase::new(repo);

        let (result, total) = uc.get_table_logs("departments", 1, 20, None).await.unwrap();
        assert_eq!(total, 0);
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn pagination_works() {
        let logs: Vec<ChangeLog> = (0..7)
            .map(|i| make_change_log("departments", &format!("dept-{}", i), "INSERT"))
            .collect();
        let repo = Arc::new(StubChangeLogRepository::with_logs(logs));
        let uc = GetAuditLogsUseCase::new(repo);

        let (page1, total) = uc.get_table_logs("departments", 1, 3, None).await.unwrap();
        assert_eq!(total, 7);
        assert_eq!(page1.len(), 3);

        let (page3, _) = uc.get_table_logs("departments", 3, 3, None).await.unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn repository_error_propagates() {
        let repo = Arc::new(StubChangeLogRepository::failing());
        let uc = GetAuditLogsUseCase::new(repo);

        let result = uc.get_table_logs("departments", 1, 20, None).await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// ManageDisplayConfigsUseCase tests
// ===========================================================================

mod manage_display_configs {
    use super::*;
    use k1s0_master_maintenance_server::usecase::manage_display_configs::ManageDisplayConfigsUseCase;

    #[tokio::test]
    async fn list_configs_for_table() {
        let table = make_table("departments");
        let table_id = table.id;
        let configs = vec![
            make_display_config(table_id, "list"),
            make_display_config(table_id, "form"),
            make_display_config(Uuid::new_v4(), "list"), // different table
        ];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let config_repo = Arc::new(StubDisplayConfigRepository::with_configs(configs));
        let uc = ManageDisplayConfigsUseCase::new(table_repo, config_repo);

        let result = uc.list_display_configs("departments", None).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn create_config() {
        let table = make_table("departments");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let config_repo = Arc::new(StubDisplayConfigRepository::new());
        let uc = ManageDisplayConfigsUseCase::new(table_repo, config_repo.clone());

        let input = serde_json::json!({
            "config_type": "list",
            "config_json": {"columns": ["id", "name", "status"]},
            "is_default": true
        });
        let result = uc
            .create_display_config("departments", &input, "admin", None)
            .await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.config_type, "list");
        assert!(config.is_default);

        let stored = config_repo.configs.read().await;
        assert_eq!(stored.len(), 1);
    }

    #[tokio::test]
    async fn get_config_by_id() {
        let config = make_display_config(Uuid::new_v4(), "list");
        let config_id = config.id;

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let config_repo = Arc::new(StubDisplayConfigRepository::with_configs(vec![config]));
        let uc = ManageDisplayConfigsUseCase::new(table_repo, config_repo);

        let result = uc.get_display_config(config_id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, config_id);
    }

    #[tokio::test]
    async fn update_config() {
        let config = make_display_config(Uuid::new_v4(), "list");
        let config_id = config.id;

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let config_repo = Arc::new(StubDisplayConfigRepository::with_configs(vec![config]));
        let uc = ManageDisplayConfigsUseCase::new(table_repo, config_repo);

        let input = serde_json::json!({
            "config_type": "form",
            "is_default": true
        });
        let result = uc.update_display_config(config_id, &input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.config_type, "form");
        assert!(updated.is_default);
    }

    #[tokio::test]
    async fn delete_config() {
        let config = make_display_config(Uuid::new_v4(), "list");
        let config_id = config.id;

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let config_repo = Arc::new(StubDisplayConfigRepository::with_configs(vec![config]));
        let uc = ManageDisplayConfigsUseCase::new(table_repo, config_repo.clone());

        let result = uc.delete_display_config(config_id).await;
        assert!(result.is_ok());

        let stored = config_repo.configs.read().await;
        assert!(stored.is_empty());
    }

    #[tokio::test]
    async fn table_not_found_returns_error() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let config_repo = Arc::new(StubDisplayConfigRepository::new());
        let uc = ManageDisplayConfigsUseCase::new(table_repo, config_repo);

        let result = uc.list_display_configs("nonexistent", None).await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// ManageRulesUseCase tests
// ===========================================================================

mod manage_rules {
    use super::*;
    use k1s0_master_maintenance_server::usecase::manage_rules::ManageRulesUseCase;

    #[tokio::test]
    async fn list_rules_for_table() {
        let table = make_table("departments");
        let table_id = table.id;
        let rules = vec![
            make_rule("name-required", table_id, "range"),
            make_rule("code-unique", table_id, "uniqueness"),
            make_rule("other-rule", Uuid::new_v4(), "range"), // different table
        ];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(rules));
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let result = uc
            .list_rules(Some("departments"), None, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn list_all_rules_without_table_filter() {
        let rules = vec![
            make_rule("rule-1", Uuid::new_v4(), "range"),
            make_rule("rule-2", Uuid::new_v4(), "uniqueness"),
        ];

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(rules));
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let result = uc.list_rules(None, None, None, None).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn get_rule_by_id() {
        let rule = make_rule("test-rule", Uuid::new_v4(), "range");
        let rule_id = rule.id;

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(vec![rule]));
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let result = uc.get_rule(rule_id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-rule");
    }

    #[tokio::test]
    async fn get_rule_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let result = uc.get_rule(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn create_rule() {
        let table = make_table("departments");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let uc = ManageRulesUseCase::new(table_repo, rule_repo.clone());

        let input = serde_json::json!({
            "name": "name-not-empty",
            "description": "Name must not be empty",
            "rule_type": "range",
            "severity": "error",
            "source_table": "departments",
            "error_message_template": "Name is required for {name}",
            "conditions": [{
                "condition_order": 1,
                "left_column": "name",
                "operator": "neq",
                "right_value": "\"\""
            }]
        });
        let result = uc.create_rule(&input, "admin", None).await;
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.name, "name-not-empty");
        assert_eq!(rule.severity, "error");
        assert!(rule.is_active);

        let stored = rule_repo.rules.read().await;
        assert_eq!(stored.len(), 1);
    }

    #[tokio::test]
    async fn create_rule_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let input = serde_json::json!({
            "name": "test-rule",
            "rule_type": "range",
            "source_table": "nonexistent",
            "error_message_template": "failed"
        });
        let result = uc.create_rule(&input, "admin", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_rule() {
        let rule = make_rule("to-delete", Uuid::new_v4(), "range");
        let rule_id = rule.id;

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(vec![rule]));
        let uc = ManageRulesUseCase::new(table_repo, rule_repo.clone());

        let result = uc.delete_rule(rule_id).await;
        assert!(result.is_ok());

        let stored = rule_repo.rules.read().await;
        assert!(stored.is_empty());
    }

    #[tokio::test]
    async fn update_rule() {
        let rule = make_rule("original", Uuid::new_v4(), "range");
        let rule_id = rule.id;

        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(vec![rule]));
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let input = serde_json::json!({
            "name": "updated-name",
            "severity": "warning",
            "is_active": false
        });
        let result = uc.update_rule(rule_id, &input, None).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-name");
        assert_eq!(updated.severity, "warning");
        assert!(!updated.is_active);
    }

    #[tokio::test]
    async fn update_rule_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let uc = ManageRulesUseCase::new(table_repo, rule_repo);

        let input = serde_json::json!({"name": "updated"});
        let result = uc.update_rule(Uuid::new_v4(), &input, None).await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// CrudRecordsUseCase tests
// ===========================================================================

mod crud_records {
    use super::*;
    use k1s0_master_maintenance_server::usecase::crud_records::CrudRecordsUseCase;

    #[allow(clippy::type_complexity)]
    fn setup_crud() -> (
        TableDefinition,
        Vec<ColumnDefinition>,
        Arc<StubTableDefinitionRepository>,
        Arc<StubColumnDefinitionRepository>,
        Arc<StubConsistencyRuleRepository>,
        Arc<StubDynamicRecordRepository>,
        Arc<StubChangeLogRepository>,
    ) {
        let table = make_table("departments");
        let table_id = table.id;
        let columns = vec![
            make_column(table_id, "id", true, true, true),
            make_column(table_id, "name", false, true, true),
            make_column(table_id, "status", false, true, true),
        ];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![
            table.clone()
        ]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(
            columns.clone(),
        ));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());

        (
            table,
            columns,
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        )
    }

    fn make_crud_uc(
        table_repo: Arc<StubTableDefinitionRepository>,
        column_repo: Arc<StubColumnDefinitionRepository>,
        rule_repo: Arc<StubConsistencyRuleRepository>,
        record_repo: Arc<StubDynamicRecordRepository>,
        change_log_repo: Arc<StubChangeLogRepository>,
    ) -> CrudRecordsUseCase {
        let rule_engine: Arc<dyn RuleEngineService> = Arc::new(StubRuleEngineService::passing());
        CrudRecordsUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
            rule_engine,
        )
    }

    #[tokio::test]
    async fn list_records_success() {
        let (_table, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) =
            setup_crud();

        // Insert test records
        {
            let mut records = record_repo.records.write().await;
            records.push((
                "departments".to_string(),
                serde_json::json!({"id": "dept-1", "name": "Engineering", "status": "active"}),
            ));
            records.push((
                "departments".to_string(),
                serde_json::json!({"id": "dept-2", "name": "Marketing", "status": "active"}),
            ));
        }

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );
        let result = uc
            .list_records("departments", 1, 20, None, None, None, None, None)
            .await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.table_name, "departments");
        assert_eq!(output.total, 2);
        assert_eq!(output.records.len(), 2);
        assert!(output.allow_create);
        assert!(output.allow_update);
        assert!(output.allow_delete);
    }

    #[tokio::test]
    async fn list_records_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );
        let result = uc
            .list_records("nonexistent", 1, 20, None, None, None, None, None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_record_success() {
        let (_, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) = setup_crud();

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo.clone(),
            change_log_repo.clone(),
        );

        let data =
            serde_json::json!({"id": "dept-new", "name": "New Department", "status": "active"});
        let result = uc
            .create_record("departments", &data, "admin", None, None)
            .await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(
            output.record.get("name").and_then(|v| v.as_str()),
            Some("New Department")
        );

        // Verify record persisted
        let records = record_repo.records.read().await;
        assert_eq!(records.len(), 1);

        // Verify change log created
        let logs = change_log_repo.logs.read().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, "INSERT");
    }

    #[tokio::test]
    async fn create_record_not_allowed() {
        let table = make_table_readonly("departments");
        let table_id = table.id;
        let columns = vec![make_column(table_id, "id", true, true, true)];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(columns));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let data = serde_json::json!({"id": "dept-1"});
        let result = uc
            .create_record("departments", &data, "admin", None, None)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Create not allowed"));
    }

    #[tokio::test]
    async fn update_record_success() {
        let (_, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) = setup_crud();

        // Insert initial record
        {
            let mut records = record_repo.records.write().await;
            records.push((
                "departments".to_string(),
                serde_json::json!({"id": "dept-1", "name": "Engineering", "status": "active"}),
            ));
        }

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo.clone(),
        );

        let data = serde_json::json!({"name": "Platform Engineering"});
        let result = uc
            .update_record("departments", "dept-1", &data, "admin", None, None)
            .await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(
            output.record.get("name").and_then(|v| v.as_str()),
            Some("Platform Engineering")
        );

        // Verify change log
        let logs = change_log_repo.logs.read().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, "UPDATE");
    }

    #[tokio::test]
    async fn update_record_not_allowed() {
        let table = make_table_readonly("departments");
        let table_id = table.id;
        let columns = vec![make_column(table_id, "id", true, true, true)];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(columns));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let data = serde_json::json!({"name": "new name"});
        let result = uc
            .update_record("departments", "dept-1", &data, "admin", None, None)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Update not allowed"));
    }

    #[tokio::test]
    async fn delete_record_success() {
        let (_, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) = setup_crud();

        // Insert record to delete
        {
            let mut records = record_repo.records.write().await;
            records.push((
                "departments".to_string(),
                serde_json::json!({"id": "dept-1", "name": "Engineering", "status": "active"}),
            ));
        }

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo.clone(),
            change_log_repo.clone(),
        );

        let result = uc
            .delete_record("departments", "dept-1", "admin", None, None)
            .await;
        assert!(result.is_ok());

        // Verify record deleted
        let records = record_repo.records.read().await;
        assert!(records.is_empty());

        // Verify change log
        let logs = change_log_repo.logs.read().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, "DELETE");
    }

    #[tokio::test]
    async fn delete_record_not_allowed() {
        let table = make_table_readonly("departments");
        let table_id = table.id;
        let columns = vec![make_column(table_id, "id", true, true, true)];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(columns));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let result = uc
            .delete_record("departments", "dept-1", "admin", None, None)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Delete not allowed"));
    }

    #[tokio::test]
    async fn get_record_success() {
        let (_, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) = setup_crud();

        {
            let mut records = record_repo.records.write().await;
            records.push((
                "departments".to_string(),
                serde_json::json!({"id": "dept-1", "name": "Engineering", "status": "active"}),
            ));
        }

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let result = uc.get_record("departments", "dept-1", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn get_record_not_found() {
        let (_, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) = setup_crud();

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let result = uc.get_record("departments", "nonexistent", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn table_permissions() {
        let (_, _, table_repo, column_repo, rule_repo, record_repo, change_log_repo) = setup_crud();

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let (create, update, delete) = uc.table_permissions("departments", None).await.unwrap();
        assert!(create);
        assert!(update);
        assert!(delete);
    }

    #[tokio::test]
    async fn table_permissions_readonly() {
        let table = make_table_readonly("readonly_table");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());

        let uc = make_crud_uc(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
        );

        let (create, update, delete) = uc.table_permissions("readonly_table", None).await.unwrap();
        assert!(!create);
        assert!(!update);
        assert!(!delete);
    }
}

// ===========================================================================
// Full lifecycle test
// ===========================================================================

mod lifecycle {
    use super::*;
    use k1s0_master_maintenance_server::usecase::crud_records::CrudRecordsUseCase;
    use k1s0_master_maintenance_server::usecase::get_audit_logs::GetAuditLogsUseCase;
    use k1s0_master_maintenance_server::usecase::manage_display_configs::ManageDisplayConfigsUseCase;
    use k1s0_master_maintenance_server::usecase::manage_rules::ManageRulesUseCase;

    #[tokio::test]
    async fn full_crud_lifecycle_with_audit_trail() {
        // Setup
        let table = make_table("departments");
        let table_id = table.id;
        let columns = vec![
            make_column(table_id, "id", true, true, true),
            make_column(table_id, "name", false, true, true),
            make_column(table_id, "status", false, true, true),
        ];

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(columns));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());
        let rule_engine: Arc<dyn RuleEngineService> = Arc::new(StubRuleEngineService::passing());

        let crud_uc = CrudRecordsUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            rule_repo,
            record_repo.clone(),
            change_log_repo.clone(),
            rule_engine,
        );
        let audit_uc = GetAuditLogsUseCase::new(change_log_repo.clone());

        // 1. Create a record
        let data = serde_json::json!({
            "id": "dept-1",
            "name": "Engineering",
            "status": "active"
        });
        let create_result = crud_uc
            .create_record(
                "departments",
                &data,
                "admin",
                None,
                Some("trace-001".to_string()),
            )
            .await
            .unwrap();
        assert_eq!(
            create_result.record.get("name").and_then(|v| v.as_str()),
            Some("Engineering")
        );

        // 2. Get the record
        let record = crud_uc
            .get_record("departments", "dept-1", None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            record.get("name").and_then(|v| v.as_str()),
            Some("Engineering")
        );

        // 3. Update the record
        let update_data = serde_json::json!({"name": "Platform Engineering"});
        let update_result = crud_uc
            .update_record(
                "departments",
                "dept-1",
                &update_data,
                "admin",
                None,
                Some("trace-002".to_string()),
            )
            .await
            .unwrap();
        assert_eq!(
            update_result.record.get("name").and_then(|v| v.as_str()),
            Some("Platform Engineering")
        );

        // 4. Verify audit trail
        let (logs, total) = audit_uc
            .get_table_logs("departments", 1, 20, None)
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert_eq!(logs[0].operation, "INSERT");
        assert_eq!(logs[1].operation, "UPDATE");

        // 5. List records
        let list = crud_uc
            .list_records("departments", 1, 20, None, None, None, None, None)
            .await
            .unwrap();
        assert_eq!(list.total, 1);

        // 6. Delete the record
        let delete_result = crud_uc
            .delete_record("departments", "dept-1", "admin", None, None)
            .await;
        assert!(delete_result.is_ok());

        // 7. Verify audit trail has all operations
        let (all_logs, all_total) = audit_uc
            .get_table_logs("departments", 1, 20, None)
            .await
            .unwrap();
        assert_eq!(all_total, 3);
        assert_eq!(all_logs[2].operation, "DELETE");

        // 8. Verify record is deleted
        let deleted = crud_uc
            .get_record("departments", "dept-1", None)
            .await
            .unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn display_config_and_rules_lifecycle() {
        let table = make_table("employees");

        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table]));
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let config_repo = Arc::new(StubDisplayConfigRepository::new());

        // 1. Create a display config
        let config_uc = ManageDisplayConfigsUseCase::new(table_repo.clone(), config_repo.clone());
        let config_input = serde_json::json!({
            "config_type": "list",
            "config_json": {"columns": ["id", "name"]},
            "is_default": true
        });
        let config = config_uc
            .create_display_config("employees", &config_input, "admin", None)
            .await
            .unwrap();

        // 2. Verify config created
        let configs = config_uc
            .list_display_configs("employees", None)
            .await
            .unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].config_type, "list");

        // 3. Create a consistency rule
        let rule_uc = ManageRulesUseCase::new(table_repo.clone(), rule_repo.clone());
        let rule_input = serde_json::json!({
            "name": "name-required",
            "rule_type": "range",
            "source_table": "employees",
            "error_message_template": "Name is required"
        });
        let rule = rule_uc
            .create_rule(&rule_input, "admin", None)
            .await
            .unwrap();

        // 4. Verify rule created
        let rules = rule_uc
            .list_rules(Some("employees"), None, None, None)
            .await
            .unwrap();
        assert_eq!(rules.len(), 1);

        // 5. Update the config
        let update_input = serde_json::json!({
            "config_type": "form",
            "is_default": false
        });
        let updated_config = config_uc
            .update_display_config(config.id, &update_input)
            .await
            .unwrap();
        assert_eq!(updated_config.config_type, "form");

        // 6. Delete the rule
        rule_uc.delete_rule(rule.id).await.unwrap();
        let remaining = rule_uc
            .list_rules(Some("employees"), None, None, None)
            .await
            .unwrap();
        assert!(remaining.is_empty());

        // 7. Delete the config
        config_uc.delete_display_config(config.id).await.unwrap();
        let remaining_configs = config_uc
            .list_display_configs("employees", None)
            .await
            .unwrap();
        assert!(remaining_configs.is_empty());
    }
}

// ---------------------------------------------------------------------------
// リレーションシップ生成ヘルパー
// ---------------------------------------------------------------------------

fn make_relationship(
    source_table_id: Uuid,
    source_col: &str,
    target_table_id: Uuid,
    target_col: &str,
) -> TableRelationship {
    TableRelationship {
        id: Uuid::new_v4(),
        source_table_id,
        source_column: source_col.to_string(),
        target_table_id,
        target_column: target_col.to_string(),
        relationship_type: RelationshipType::OneToMany,
        display_name: None,
        is_cascade_delete: false,
        created_at: Utc::now(),
    }
}

// ===========================================================================
// ManageTableDefinitionsUseCase tests
// ===========================================================================

mod manage_table_definitions {
    use super::*;
    use k1s0_master_maintenance_server::domain::entity::table_definition::{
        CreateTableDefinition, UpdateTableDefinition,
    };
    use k1s0_master_maintenance_server::usecase::manage_table_definitions::ManageTableDefinitionsUseCase;

    /// テーブル一覧取得: 全テーブルを返す
    #[tokio::test]
    async fn list_tables_returns_all() {
        let t1 = make_table("orders");
        let t2 = make_table("products");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![
            t1.clone(),
            t2.clone(),
        ]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc
            .list_tables(None, false, &DomainFilter::All)
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    /// テーブル取得: 存在するテーブルはSomeを返す
    #[tokio::test]
    async fn get_table_found() {
        let table = make_table("customers");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc.get_table("customers", None).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "customers");
    }

    /// テーブル取得: 存在しないテーブルはNoneを返す
    #[tokio::test]
    async fn get_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc.get_table("nonexistent", None).await.unwrap();
        assert!(result.is_none());
    }

    /// テーブル作成: スキーマ作成とリポジトリ保存が完了する
    #[tokio::test]
    async fn create_table_success() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(
            table_repo.clone(),
            column_repo,
            schema_manager,
        );

        let input = CreateTableDefinition {
            name: "inventory".to_string(),
            schema_name: "public".to_string(),
            database_name: None,
            display_name: "Inventory".to_string(),
            description: None,
            category: None,
            allow_create: Some(true),
            allow_update: Some(true),
            allow_delete: Some(true),
            read_roles: None,
            write_roles: None,
            admin_roles: None,
            sort_order: None,
            domain_scope: None,
        };
        let table = uc.create_table(&input, "admin").await.unwrap();
        assert_eq!(table.name, "inventory");

        // リポジトリに格納されていることを確認
        let found = uc.get_table("inventory", None).await.unwrap();
        assert!(found.is_some());
    }

    /// テーブル削除: 存在するテーブルを削除できる
    #[tokio::test]
    async fn delete_table_success() {
        let table = make_table("temp_table");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(
            table_repo.clone(),
            column_repo,
            schema_manager,
        );

        uc.delete_table("temp_table", None).await.unwrap();
        let found = uc.get_table("temp_table", None).await.unwrap();
        assert!(found.is_none());
    }

    /// テーブル削除: 存在しないテーブルはエラーを返す
    #[tokio::test]
    async fn delete_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc.delete_table("ghost_table", None).await;
        assert!(result.is_err());
    }

    /// テーブル更新: 表示名を変更できる
    #[tokio::test]
    async fn update_table_display_name() {
        let table = make_table("products");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let update = UpdateTableDefinition {
            display_name: Some("商品マスタ".to_string()),
            description: None,
            category: None,
            is_active: None,
            allow_create: None,
            allow_update: None,
            allow_delete: None,
            read_roles: None,
            write_roles: None,
            admin_roles: None,
            sort_order: None,
        };
        let updated = uc.update_table("products", &update, None).await.unwrap();
        assert_eq!(updated.display_name, "商品マスタ");
    }

    /// ドメイン一覧取得: ドメインリストを返す
    #[tokio::test]
    async fn list_domains_returns_domain() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageTableDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc.list_domains().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "default");
    }
}

// ===========================================================================
// ManageColumnDefinitionsUseCase tests
// ===========================================================================

mod manage_column_definitions {
    use super::*;
    use k1s0_master_maintenance_server::usecase::manage_column_definitions::ManageColumnDefinitionsUseCase;

    /// カラム一覧取得: テーブルが存在すれば対応するカラムを返す
    #[tokio::test]
    async fn list_columns_success() {
        let table = make_table("employees");
        let col = make_column(table.id, "name", false, true, true);
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(vec![col.clone()]));
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageColumnDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc.list_columns("employees", None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].column_name, "name");
    }

    /// カラム一覧取得: テーブルが存在しない場合はエラーを返す
    #[tokio::test]
    async fn list_columns_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageColumnDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let result = uc.list_columns("nonexistent", None).await;
        assert!(result.is_err());
    }

    /// カラム追加: テーブルに新しいカラムを追加できる
    #[tokio::test]
    async fn create_columns_success() {
        let table = make_table("items");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageColumnDefinitionsUseCase::new(
            table_repo,
            column_repo.clone(),
            schema_manager,
        );

        let input = serde_json::json!({
            "columns": [{
                "column_name": "price",
                "display_name": "価格",
                "data_type": "integer"
            }]
        });
        let result = uc.create_columns("items", &input, None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].column_name, "price");
    }

    /// カラム削除: 既存カラムを削除できる
    #[tokio::test]
    async fn delete_column_success() {
        let table = make_table("orders");
        let col = make_column(table.id, "notes", false, true, true);
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(vec![col.clone()]));
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageColumnDefinitionsUseCase::new(
            table_repo,
            column_repo.clone(),
            schema_manager,
        );

        uc.delete_column("orders", "notes", None).await.unwrap();

        // カラムが削除されていることを確認
        let remaining = uc.list_columns("orders", None).await.unwrap();
        assert!(remaining.iter().all(|c| c.column_name != "notes"));
    }

    /// カラム更新: テーブルが存在しない場合はエラーを返す
    #[tokio::test]
    async fn update_column_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageColumnDefinitionsUseCase::new(table_repo, column_repo, schema_manager);

        let input = serde_json::json!({
            "column_name": "price",
            "display_name": "価格",
            "data_type": "integer"
        });
        let result = uc.update_column("nonexistent", "price", &input, None).await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// CheckConsistencyUseCase tests
// ===========================================================================

mod check_consistency {
    use super::*;
    use k1s0_master_maintenance_server::usecase::check_consistency::CheckConsistencyUseCase;

    /// ルール実行: 存在しないルールIDはエラーを返す
    #[tokio::test]
    async fn execute_rule_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let rule_engine = Arc::new(StubRuleEngineService::passing());
        let uc = CheckConsistencyUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            rule_engine,
        );

        let result = uc.execute_rule(Uuid::new_v4(), None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rule not found"));
    }

    /// 全ルールチェック: テーブルが存在しない場合はエラーを返す
    #[tokio::test]
    async fn check_all_rules_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let rule_engine = Arc::new(StubRuleEngineService::passing());
        let uc = CheckConsistencyUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            rule_engine,
        );

        let result = uc.check_all_rules("ghost_table", None).await;
        assert!(result.is_err());
    }

    /// 全ルールチェック: ルールが0件の場合は pass を返す
    #[tokio::test]
    async fn check_all_rules_no_rules_returns_pass() {
        let table = make_table("suppliers");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let rule_engine = Arc::new(StubRuleEngineService::passing());
        let uc = CheckConsistencyUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            rule_engine,
        );

        let results = uc.check_all_rules("suppliers", None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    /// ルール実行: レコードが0件の場合でもルール処理が完了し pass を返す
    #[tokio::test]
    async fn execute_rule_with_no_records_returns_pass() {
        let table = make_table("warehouses");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule = make_rule("test-rule", table.id, "field_presence");
        let rule_id = rule.id;
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(vec![rule]));
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let rule_engine = Arc::new(StubRuleEngineService::passing());
        let uc = CheckConsistencyUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            rule_engine,
        );

        let results = uc.execute_rule(rule_id, None).await.unwrap();
        // レコードがないので pass 結果が1件返る
        assert!(!results.is_empty());
        assert!(results[0].passed);
    }

    /// ルール絞り込みチェック: 指定したrule_idのみ評価される
    #[tokio::test]
    async fn check_rules_with_specific_ids() {
        let table = make_table("vendors");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let rule1 = make_rule("rule-a", table.id, "field_presence");
        let rule1_id = rule1.id.to_string();
        let rule2 = make_rule("rule-b", table.id, "field_presence");
        let rule_repo = Arc::new(StubConsistencyRuleRepository::with_rules(vec![
            rule1.clone(),
            rule2.clone(),
        ]));
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let rule_engine = Arc::new(StubRuleEngineService::passing());
        let uc = CheckConsistencyUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            rule_engine,
        );

        // rule-a だけを対象に実行（レコードなしなのでpass）
        let results = uc
            .check_rules("vendors", &[rule1_id], None)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.passed));
    }
}

// ===========================================================================
// ManageRelationshipsUseCase tests
// ===========================================================================

mod manage_relationships {
    use super::*;
    use k1s0_master_maintenance_server::usecase::manage_relationships::ManageRelationshipsUseCase;

    /// リレーションシップ一覧取得: 空の場合は空ベクを返す
    #[tokio::test]
    async fn list_relationships_empty() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rel_repo = Arc::new(StubTableRelationshipRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageRelationshipsUseCase::new(
            table_repo,
            rel_repo,
            record_repo,
            column_repo,
            schema_manager,
        );

        let result = uc.list_relationships().await.unwrap();
        assert!(result.is_empty());
    }

    /// リレーションシップ一覧取得: データがあれば全件返す
    #[tokio::test]
    async fn list_relationships_with_data() {
        let t1 = make_table("orders");
        let t2 = make_table("customers");
        let rel = make_relationship(t1.id, "customer_id", t2.id, "id");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![
            t1.clone(),
            t2.clone(),
        ]));
        let rel_repo = Arc::new(StubTableRelationshipRepository::with_relationships(vec![rel]));
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageRelationshipsUseCase::new(
            table_repo,
            rel_repo,
            record_repo,
            column_repo,
            schema_manager,
        );

        let result = uc.list_relationships().await.unwrap();
        assert_eq!(result.len(), 1);
    }

    /// リレーションシップ作成: ソーステーブルが存在しない場合はエラーを返す
    #[tokio::test]
    async fn create_relationship_source_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rel_repo = Arc::new(StubTableRelationshipRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageRelationshipsUseCase::new(
            table_repo,
            rel_repo,
            record_repo,
            column_repo,
            schema_manager,
        );

        let input = serde_json::json!({
            "source_table": "nonexistent",
            "source_column": "id",
            "target_table": "customers",
            "target_column": "id",
            "relationship_type": "many_to_one"
        });
        let result = uc.create_relationship(&input, "admin", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    /// リレーションシップ作成: ソース・ターゲットテーブルとカラムが存在する場合は成功する
    #[tokio::test]
    async fn create_relationship_success() {
        let t_orders = make_table("orders_rel");
        let t_customers = make_table("customers_rel");
        let col_orders = make_column(t_orders.id, "customer_id", false, true, true);
        let col_customers = make_column(t_customers.id, "id", true, true, true);
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![
            t_orders.clone(),
            t_customers.clone(),
        ]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::with_columns(vec![
            col_orders,
            col_customers,
        ]));
        let rel_repo = Arc::new(StubTableRelationshipRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageRelationshipsUseCase::new(
            table_repo,
            rel_repo.clone(),
            record_repo,
            column_repo,
            schema_manager,
        );

        let input = serde_json::json!({
            "source_table": "orders_rel",
            "source_column": "customer_id",
            "target_table": "customers_rel",
            "target_column": "id",
            "relationship_type": "many_to_one"
        });
        let rel = uc.create_relationship(&input, "admin", None).await.unwrap();
        assert_eq!(rel.source_column, "customer_id");
        assert_eq!(rel.target_column, "id");

        // リポジトリに格納されていることを確認
        let all = uc.list_relationships().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    /// リレーションシップ削除: 存在しないIDはエラーを返す
    #[tokio::test]
    async fn delete_relationship_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let rel_repo = Arc::new(StubTableRelationshipRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let schema_manager = Arc::new(StubSchemaManager);
        let uc = ManageRelationshipsUseCase::new(
            table_repo,
            rel_repo,
            record_repo,
            column_repo,
            schema_manager,
        );

        let result = uc.delete_relationship(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}

// ===========================================================================
// ImportExportUseCase tests
// ===========================================================================

mod import_export {
    use super::*;
    use k1s0_master_maintenance_server::usecase::crud_records::CrudRecordsUseCase;
    use k1s0_master_maintenance_server::usecase::import_export::ImportExportUseCase;

    /// CrudRecordsUseCase をスタブで構築するヘルパー
    fn build_crud_uc(
        table_repo: Arc<StubTableDefinitionRepository>,
        column_repo: Arc<StubColumnDefinitionRepository>,
    ) -> Arc<CrudRecordsUseCase> {
        let rule_repo = Arc::new(StubConsistencyRuleRepository::new());
        let record_repo = Arc::new(StubDynamicRecordRepository::new());
        let change_log_repo = Arc::new(StubChangeLogRepository::new());
        let rule_engine = Arc::new(StubRuleEngineService::passing());
        Arc::new(CrudRecordsUseCase::new(
            table_repo,
            column_repo,
            rule_repo,
            record_repo,
            change_log_repo,
            rule_engine,
        ))
    }

    /// テーブルが存在しない場合はエラーを返す
    #[tokio::test]
    async fn import_records_table_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let crud_uc = build_crud_uc(table_repo.clone(), column_repo.clone());
        let import_job_repo = Arc::new(StubImportJobRepository::new());
        let uc = ImportExportUseCase::new(table_repo, column_repo, import_job_repo, crud_uc);

        let data = serde_json::json!({"records": [{"name": "test"}]});
        let result = uc.import_records("ghost_table", &data, "admin", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    /// レコードを正常にインポートし、importJobのstatusがcompletedになる
    #[tokio::test]
    async fn import_records_success() {
        let table = make_table("parts");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let crud_uc = build_crud_uc(table_repo.clone(), column_repo.clone());
        let import_job_repo = Arc::new(StubImportJobRepository::new());
        let uc = ImportExportUseCase::new(
            table_repo,
            column_repo,
            import_job_repo,
            crud_uc,
        );

        let data = serde_json::json!({
            "records": [
                {"name": "bolt"},
                {"name": "nut"}
            ]
        });
        let job = uc
            .import_records("parts", &data, "admin", None)
            .await
            .unwrap();
        assert_eq!(job.status, "completed");
        assert_eq!(job.total_rows, 2);
        assert_eq!(job.processed_rows, 2);
        assert_eq!(job.error_rows, 0);
    }

    /// インポートジョブが見つからない場合はNoneを返す
    #[tokio::test]
    async fn get_import_job_not_found() {
        let table_repo = Arc::new(StubTableDefinitionRepository::new());
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let crud_uc = build_crud_uc(table_repo.clone(), column_repo.clone());
        let import_job_repo = Arc::new(StubImportJobRepository::new());
        let uc = ImportExportUseCase::new(table_repo, column_repo, import_job_repo, crud_uc);

        let result = uc.get_import_job(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    /// エクスポート: テーブルのレコードが0件でも正常に完了する
    #[tokio::test]
    async fn export_records_empty_table() {
        let table = make_table("archived_orders");
        let table_repo = Arc::new(StubTableDefinitionRepository::with_tables(vec![table.clone()]));
        let column_repo = Arc::new(StubColumnDefinitionRepository::new());
        let crud_uc = build_crud_uc(table_repo.clone(), column_repo.clone());
        let import_job_repo = Arc::new(StubImportJobRepository::new());
        let uc = ImportExportUseCase::new(table_repo, column_repo, import_job_repo, crud_uc);

        let result = uc
            .export_records("archived_orders", Some("json"), None)
            .await
            .unwrap();
        assert_eq!(result.table, "archived_orders");
        assert_eq!(result.total, 0);
        assert!(result.records.is_empty());
    }
}
