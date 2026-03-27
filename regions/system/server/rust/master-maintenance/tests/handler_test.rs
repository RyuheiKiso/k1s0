// Master-Maintenance サーバーの handler テスト
// axum-test を使って REST API エンドポイントの動作を確認する
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use async_trait::async_trait;
use axum_test::TestServer;
use chrono::Utc;
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_master_maintenance_server::adapter::handler::{router, AppState};
use k1s0_master_maintenance_server::domain::entity::change_log::ChangeLog;
use k1s0_master_maintenance_server::domain::entity::column_definition::{
    ColumnDefinition, CreateColumnDefinition,
};
use k1s0_master_maintenance_server::domain::entity::consistency_rule::ConsistencyRule;
use k1s0_master_maintenance_server::domain::entity::display_config::DisplayConfig;
use k1s0_master_maintenance_server::domain::entity::import_job::ImportJob;
use k1s0_master_maintenance_server::domain::entity::rule_condition::RuleCondition;
use k1s0_master_maintenance_server::domain::entity::table_definition::{
    CreateTableDefinition, TableDefinition, UpdateTableDefinition,
};
use k1s0_master_maintenance_server::domain::entity::table_relationship::TableRelationship;
use k1s0_master_maintenance_server::domain::repository::change_log_repository::ChangeLogRepository;
use k1s0_master_maintenance_server::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use k1s0_master_maintenance_server::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;
use k1s0_master_maintenance_server::domain::repository::display_config_repository::DisplayConfigRepository;
use k1s0_master_maintenance_server::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use k1s0_master_maintenance_server::domain::repository::import_job_repository::ImportJobRepository;
use k1s0_master_maintenance_server::domain::repository::table_definition_repository::TableDefinitionRepository;
use k1s0_master_maintenance_server::domain::repository::table_relationship_repository::TableRelationshipRepository;
use k1s0_master_maintenance_server::domain::service::rule_engine_service::RuleEngineService;
use k1s0_master_maintenance_server::domain::value_object::domain_filter::DomainFilter;
use k1s0_master_maintenance_server::domain::value_object::rule_result::RuleResult;
use k1s0_master_maintenance_server::infrastructure::schema::SchemaManager;
use k1s0_master_maintenance_server::usecase;

// ---------------------------------------------------------------------------
// Stub: In-memory TableDefinitionRepository
// ---------------------------------------------------------------------------

struct StubTableRepo {
    tables: RwLock<Vec<TableDefinition>>,
}

impl StubTableRepo {
    fn new() -> Self {
        Self {
            tables: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl TableDefinitionRepository for StubTableRepo {
    async fn find_all(
        &self,
        _category: Option<&str>,
        _active_only: bool,
        _domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>> {
        Ok(self.tables.read().await.clone())
    }

    async fn find_by_name(
        &self,
        name: &str,
        _domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<TableDefinition>> {
        Ok(self
            .tables
            .read()
            .await
            .iter()
            .find(|t| t.name == name)
            .cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        Ok(self
            .tables
            .read()
            .await
            .iter()
            .find(|t| t.id == id)
            .cloned())
    }

    async fn create(
        &self,
        input: &CreateTableDefinition,
        created_by: &str,
    ) -> anyhow::Result<TableDefinition> {
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
        self.tables.write().await.retain(|t| t.name != name);
        Ok(())
    }

    async fn find_domains(&self) -> anyhow::Result<Vec<(String, i64)>> {
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory ColumnDefinitionRepository
// ---------------------------------------------------------------------------

struct StubColumnRepo;

#[async_trait]
impl ColumnDefinitionRepository for StubColumnRepo {
    async fn find_by_table_id(&self, _table_id: Uuid) -> anyhow::Result<Vec<ColumnDefinition>> {
        Ok(vec![])
    }

    async fn find_by_table_and_column(
        &self,
        _table_id: Uuid,
        _column_name: &str,
    ) -> anyhow::Result<Option<ColumnDefinition>> {
        Ok(None)
    }

    async fn create_batch(
        &self,
        _table_id: Uuid,
        _inputs: &[CreateColumnDefinition],
    ) -> anyhow::Result<Vec<ColumnDefinition>> {
        Ok(vec![])
    }

    async fn update(
        &self,
        _table_id: Uuid,
        _column_name: &str,
        _input: &CreateColumnDefinition,
    ) -> anyhow::Result<ColumnDefinition> {
        anyhow::bail!("not found")
    }

    async fn delete(&self, _table_id: Uuid, _column_name: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory ChangeLogRepository
// ---------------------------------------------------------------------------

struct StubChangeLogRepo;

#[async_trait]
impl ChangeLogRepository for StubChangeLogRepo {
    async fn create(&self, log: &ChangeLog) -> anyhow::Result<ChangeLog> {
        Ok(log.clone())
    }

    async fn find_by_table(
        &self,
        _table_name: &str,
        _page: i32,
        _page_size: i32,
    ) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        Ok((vec![], 0))
    }

    async fn find_by_record(
        &self,
        _table_name: &str,
        _record_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        Ok((vec![], 0))
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory DynamicRecordRepository
// ---------------------------------------------------------------------------

struct StubDynamicRecordRepo;

#[async_trait]
impl DynamicRecordRepository for StubDynamicRecordRepo {
    async fn find_all(
        &self,
        _table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        _page: i32,
        _page_size: i32,
        _sort: Option<&str>,
        _filter: Option<&str>,
        _search: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)> {
        Ok((vec![], 0))
    }

    async fn find_by_id(
        &self,
        _table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        _record_id: &str,
    ) -> anyhow::Result<Option<Value>> {
        Ok(None)
    }

    async fn create(
        &self,
        _table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        data: &Value,
    ) -> anyhow::Result<Value> {
        Ok(data.clone())
    }

    async fn update(
        &self,
        _table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        _record_id: &str,
        data: &Value,
    ) -> anyhow::Result<Value> {
        Ok(data.clone())
    }

    async fn delete(&self, _table_def: &TableDefinition, _record_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory ConsistencyRuleRepository
// ---------------------------------------------------------------------------

struct StubRuleRepo;

#[async_trait]
impl ConsistencyRuleRepository for StubRuleRepo {
    async fn find_all(
        &self,
        _table_id: Option<Uuid>,
        _rule_type: Option<&str>,
        _severity: Option<&str>,
    ) -> anyhow::Result<Vec<ConsistencyRule>> {
        Ok(vec![])
    }

    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<ConsistencyRule>> {
        Ok(None)
    }

    async fn find_by_table_id(
        &self,
        _table_id: Uuid,
        _timing: Option<&str>,
    ) -> anyhow::Result<Vec<ConsistencyRule>> {
        Ok(vec![])
    }

    async fn create(
        &self,
        rule: &ConsistencyRule,
        _conditions: &[RuleCondition],
    ) -> anyhow::Result<ConsistencyRule> {
        Ok(rule.clone())
    }

    async fn update(&self, _id: Uuid, rule: &ConsistencyRule) -> anyhow::Result<ConsistencyRule> {
        Ok(rule.clone())
    }

    async fn replace_conditions(
        &self,
        _rule_id: Uuid,
        _conditions: &[RuleCondition],
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_conditions_by_rule_id(
        &self,
        _rule_id: Uuid,
    ) -> anyhow::Result<Vec<RuleCondition>> {
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// Stub: DisplayConfigRepository
// ---------------------------------------------------------------------------

struct StubDisplayConfigRepo;

#[async_trait]
impl DisplayConfigRepository for StubDisplayConfigRepo {
    async fn find_by_table_id(&self, _table_id: Uuid) -> anyhow::Result<Vec<DisplayConfig>> {
        Ok(vec![])
    }

    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<DisplayConfig>> {
        Ok(None)
    }

    async fn create(&self, config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        Ok(config.clone())
    }

    async fn update(&self, _id: Uuid, config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        Ok(config.clone())
    }

    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: RuleEngineService
// ---------------------------------------------------------------------------

struct StubRuleEngineService;

#[async_trait]
impl RuleEngineService for StubRuleEngineService {
    async fn evaluate_rule(
        &self,
        _rule: &ConsistencyRule,
        _record_data: &Value,
    ) -> anyhow::Result<RuleResult> {
        Ok(RuleResult::pass())
    }
}

// ---------------------------------------------------------------------------
// Stub: SchemaManager
// ---------------------------------------------------------------------------

struct StubSchemaManager;

#[async_trait]
impl SchemaManager for StubSchemaManager {
    async fn create_table(&self, _input: &CreateTableDefinition) -> anyhow::Result<()> {
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
// Stub: TableRelationshipRepository
// ---------------------------------------------------------------------------

struct StubRelationshipRepo;

#[async_trait]
impl TableRelationshipRepository for StubRelationshipRepo {
    async fn find_all(&self) -> anyhow::Result<Vec<TableRelationship>> {
        Ok(vec![])
    }

    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<TableRelationship>> {
        Ok(None)
    }

    async fn find_by_table_id(&self, _table_id: Uuid) -> anyhow::Result<Vec<TableRelationship>> {
        Ok(vec![])
    }

    async fn create(&self, relationship: &TableRelationship) -> anyhow::Result<TableRelationship> {
        Ok(relationship.clone())
    }

    async fn update(
        &self,
        _id: Uuid,
        relationship: &TableRelationship,
    ) -> anyhow::Result<TableRelationship> {
        Ok(relationship.clone())
    }

    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub: ImportJobRepository
// ---------------------------------------------------------------------------

struct StubImportJobRepo {
    jobs: RwLock<Vec<ImportJob>>,
}

impl StubImportJobRepo {
    fn new() -> Self {
        Self {
            jobs: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ImportJobRepository for StubImportJobRepo {
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
// Helper: AppState を構築する
// ---------------------------------------------------------------------------

fn build_state() -> AppState {
    let table_repo = Arc::new(StubTableRepo::new());
    let column_repo = Arc::new(StubColumnRepo);
    let change_log_repo = Arc::new(StubChangeLogRepo);
    let record_repo = Arc::new(StubDynamicRecordRepo);
    let rule_repo = Arc::new(StubRuleRepo);
    let display_config_repo = Arc::new(StubDisplayConfigRepo);
    let rule_engine = Arc::new(StubRuleEngineService);
    let schema_manager = Arc::new(StubSchemaManager);
    let relationship_repo = Arc::new(StubRelationshipRepo);
    let import_job_repo = Arc::new(StubImportJobRepo::new());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "master-maintenance-test",
    ));

    let crud_records_uc = Arc::new(usecase::crud_records::CrudRecordsUseCase::new(
        table_repo.clone(),
        column_repo.clone(),
        rule_repo.clone(),
        record_repo.clone(),
        change_log_repo.clone(),
        rule_engine.clone(),
    ));

    AppState {
        manage_tables_uc: Arc::new(
            usecase::manage_table_definitions::ManageTableDefinitionsUseCase::new(
                table_repo.clone(),
                column_repo.clone(),
                schema_manager.clone(),
            ),
        ),
        manage_columns_uc: Arc::new(
            usecase::manage_column_definitions::ManageColumnDefinitionsUseCase::new(
                table_repo.clone(),
                column_repo.clone(),
                schema_manager.clone(),
            ),
        ),
        crud_records_uc: crud_records_uc.clone(),
        manage_rules_uc: Arc::new(usecase::manage_rules::ManageRulesUseCase::new(
            table_repo.clone(),
            rule_repo.clone(),
        )),
        check_consistency_uc: Arc::new(usecase::check_consistency::CheckConsistencyUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            rule_repo.clone(),
            record_repo.clone(),
            rule_engine.clone(),
        )),
        get_audit_logs_uc: Arc::new(usecase::get_audit_logs::GetAuditLogsUseCase::new(
            change_log_repo.clone(),
        )),
        manage_relationships_uc: Arc::new(
            usecase::manage_relationships::ManageRelationshipsUseCase::new(
                table_repo.clone(),
                relationship_repo.clone(),
                record_repo.clone(),
                column_repo.clone(),
                schema_manager.clone(),
            ),
        ),
        manage_display_configs_uc: Arc::new(
            usecase::manage_display_configs::ManageDisplayConfigsUseCase::new(
                table_repo.clone(),
                display_config_repo.clone(),
            ),
        ),
        import_export_uc: Arc::new(usecase::import_export::ImportExportUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            import_job_repo.clone(),
            crud_records_uc.clone(),
        )),
        metrics,
        kafka_producer: None,
        auth_state: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// GET /healthz は 200 {"status":"ok"} を返す
#[tokio::test]
async fn healthz_returns_ok() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/healthz").await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["status"], "ok");
}

/// GET /readyz はスタブリポジトリで "ready" を返す
#[tokio::test]
async fn readyz_returns_ready() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/readyz").await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["status"], "ready");
}

/// GET /api/v1/tables は認証なしの場合 401 を返す（auth 保護確認）
#[tokio::test]
async fn list_tables_without_auth_returns_401() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/tables").await;
    resp.assert_status_unauthorized();
}

/// GET /api/v1/rules は認証なしの場合 401 を返す
#[tokio::test]
async fn list_rules_without_auth_returns_401() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/rules").await;
    resp.assert_status_unauthorized();
}

/// GET /api/v1/relationships は認証なしの場合 401 を返す
#[tokio::test]
async fn list_relationships_without_auth_returns_401() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/relationships").await;
    resp.assert_status_unauthorized();
}
