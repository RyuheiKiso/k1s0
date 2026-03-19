// マスターメンテナンスサーバーの統合テスト。
// router の初期化と基本的なエンドポイントの動作を検証する。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_master_maintenance_server::adapter::handler::{router, AppState};
// 認証状態の型をインポート（共通AuthStateを使用）
use k1s0_master_maintenance_server::adapter::middleware::auth::AuthState;
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
use k1s0_master_maintenance_server::infrastructure::schema::PhysicalSchemaManager;

// --- テストダブル: テーブル定義リポジトリ ---

/// テスト用のテーブル定義リポジトリ。全メソッドが空の結果を返す。
struct StubTableDefRepo;

#[async_trait]
impl TableDefinitionRepository for StubTableDefRepo {
    async fn find_all(
        &self,
        _category: Option<&str>,
        _active_only: bool,
        _domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>> {
        Ok(vec![])
    }
    async fn find_by_name(
        &self,
        _name: &str,
        _domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<TableDefinition>> {
        Ok(None)
    }
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        Ok(None)
    }
    async fn create(
        &self,
        _input: &CreateTableDefinition,
        _created_by: &str,
    ) -> anyhow::Result<TableDefinition> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(
        &self,
        _name: &str,
        _input: &UpdateTableDefinition,
        _domain_scope: Option<&str>,
    ) -> anyhow::Result<TableDefinition> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _name: &str, _domain_scope: Option<&str>) -> anyhow::Result<()> {
        Ok(())
    }
    async fn find_domains(&self) -> anyhow::Result<Vec<(String, i64)>> {
        Ok(vec![])
    }
}

// --- テストダブル: カラム定義リポジトリ ---

/// テスト用のカラム定義リポジトリ。全メソッドが空の結果を返す。
struct StubColumnDefRepo;

#[async_trait]
impl ColumnDefinitionRepository for StubColumnDefRepo {
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
        _columns: &[CreateColumnDefinition],
    ) -> anyhow::Result<Vec<ColumnDefinition>> {
        Ok(vec![])
    }
    async fn update(
        &self,
        _table_id: Uuid,
        _column_name: &str,
        _input: &CreateColumnDefinition,
    ) -> anyhow::Result<ColumnDefinition> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _table_id: Uuid, _column_name: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: 動的レコードリポジトリ ---

/// テスト用の動的レコードリポジトリ。全メソッドが空の結果を返す。
struct StubRecordRepo;

#[async_trait]
impl DynamicRecordRepository for StubRecordRepo {
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
        _data: &Value,
    ) -> anyhow::Result<Value> {
        Ok(Value::Null)
    }
    async fn update(
        &self,
        _table_def: &TableDefinition,
        _columns: &[ColumnDefinition],
        _record_id: &str,
        _data: &Value,
    ) -> anyhow::Result<Value> {
        Ok(Value::Null)
    }
    async fn delete(&self, _table_def: &TableDefinition, _record_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: 整合性ルールリポジトリ ---

/// テスト用の整合性ルールリポジトリ。全メソッドが空の結果を返す。
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
        _rule: &ConsistencyRule,
        _conditions: &[RuleCondition],
    ) -> anyhow::Result<ConsistencyRule> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(&self, _id: Uuid, _rule: &ConsistencyRule) -> anyhow::Result<ConsistencyRule> {
        anyhow::bail!("stub: not implemented")
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

// --- テストダブル: 変更ログリポジトリ ---

/// テスト用の変更ログリポジトリ。全メソッドが空の結果を返す。
struct StubChangeLogRepo;

#[async_trait]
impl ChangeLogRepository for StubChangeLogRepo {
    async fn create(&self, _log: &ChangeLog) -> anyhow::Result<ChangeLog> {
        anyhow::bail!("stub: not implemented")
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

// --- テストダブル: テーブルリレーションシップリポジトリ ---

/// テスト用のリレーションシップリポジトリ。全メソッドが空の結果を返す。
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
    async fn create(&self, _relationship: &TableRelationship) -> anyhow::Result<TableRelationship> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(
        &self,
        _id: Uuid,
        _relationship: &TableRelationship,
    ) -> anyhow::Result<TableRelationship> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: 表示設定リポジトリ ---

/// テスト用の表示設定リポジトリ。全メソッドが空の結果を返す。
struct StubDisplayConfigRepo;

#[async_trait]
impl DisplayConfigRepository for StubDisplayConfigRepo {
    async fn find_by_table_id(&self, _table_id: Uuid) -> anyhow::Result<Vec<DisplayConfig>> {
        Ok(vec![])
    }
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<DisplayConfig>> {
        Ok(None)
    }
    async fn create(&self, _config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(&self, _id: Uuid, _config: &DisplayConfig) -> anyhow::Result<DisplayConfig> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: インポートジョブリポジトリ ---

/// テスト用のインポートジョブリポジトリ。全メソッドが空の結果を返す。
struct StubImportJobRepo;

#[async_trait]
impl ImportJobRepository for StubImportJobRepo {
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<ImportJob>> {
        Ok(None)
    }
    async fn create(&self, _job: &ImportJob) -> anyhow::Result<ImportJob> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(&self, _id: Uuid, _job: &ImportJob) -> anyhow::Result<ImportJob> {
        anyhow::bail!("stub: not implemented")
    }
}

// --- テストダブル: ルールエンジンサービス ---

/// テスト用のルールエンジンサービス。常に成功結果を返す。
struct StubRuleEngine;

#[async_trait]
impl RuleEngineService for StubRuleEngine {
    async fn evaluate_rule(
        &self,
        _rule: &ConsistencyRule,
        _record_data: &Value,
    ) -> anyhow::Result<RuleResult> {
        Ok(RuleResult::pass())
    }
}

/// テスト用の AppState を構築し、router を生成するヘルパー関数。
/// 認証有効モードで構築する（ダミー JWKS verifier を使用）。
/// PhysicalSchemaManager は遅延接続の PgPool を使用し、実際のDB接続は行わない。
fn make_test_app() -> axum::Router {
    let table_repo: Arc<dyn TableDefinitionRepository> = Arc::new(StubTableDefRepo);
    let column_repo: Arc<dyn ColumnDefinitionRepository> = Arc::new(StubColumnDefRepo);
    let record_repo: Arc<dyn DynamicRecordRepository> = Arc::new(StubRecordRepo);
    let rule_repo: Arc<dyn ConsistencyRuleRepository> = Arc::new(StubRuleRepo);
    let change_log_repo: Arc<dyn ChangeLogRepository> = Arc::new(StubChangeLogRepo);
    let relationship_repo: Arc<dyn TableRelationshipRepository> = Arc::new(StubRelationshipRepo);
    let display_config_repo: Arc<dyn DisplayConfigRepository> = Arc::new(StubDisplayConfigRepo);
    let import_job_repo: Arc<dyn ImportJobRepository> = Arc::new(StubImportJobRepo);
    let rule_engine: Arc<dyn RuleEngineService> = Arc::new(StubRuleEngine);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "master-maintenance-test",
    ));

    // 遅延接続でプールを作成（テスト中はDBアクセスしないため接続不要）
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/dummy_test_db")
        .expect("failed to create lazy pool");
    let schema_manager = Arc::new(PhysicalSchemaManager::new(pool));

    // 各ユースケースの構築
    let manage_tables_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::manage_table_definitions::ManageTableDefinitionsUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            schema_manager.clone(),
        ),
    );
    let manage_columns_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::manage_column_definitions::ManageColumnDefinitionsUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            schema_manager.clone(),
        ),
    );
    let crud_records_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::crud_records::CrudRecordsUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            rule_repo.clone(),
            record_repo.clone(),
            change_log_repo.clone(),
            rule_engine.clone(),
        ),
    );
    let manage_rules_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::manage_rules::ManageRulesUseCase::new(
            table_repo.clone(),
            rule_repo.clone(),
        ),
    );
    let check_consistency_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::check_consistency::CheckConsistencyUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            rule_repo.clone(),
            record_repo.clone(),
            rule_engine.clone(),
        ),
    );
    let get_audit_logs_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::get_audit_logs::GetAuditLogsUseCase::new(
            change_log_repo.clone(),
        ),
    );
    let manage_relationships_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::manage_relationships::ManageRelationshipsUseCase::new(
            table_repo.clone(),
            relationship_repo.clone(),
            record_repo.clone(),
            column_repo.clone(),
            schema_manager.clone(),
        ),
    );
    let manage_display_configs_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::manage_display_configs::ManageDisplayConfigsUseCase::new(
            table_repo.clone(),
            display_config_repo.clone(),
        ),
    );
    let import_export_uc = Arc::new(
        k1s0_master_maintenance_server::usecase::import_export::ImportExportUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            import_job_repo.clone(),
            crud_records_uc.clone(),
        ),
    );

    // ダミーの JwksVerifier を作成（テスト中は実際にトークン検証しない）
    let verifier = Arc::new(
        k1s0_auth::JwksVerifier::new(
            "https://dummy.example.com/jwks",
            "https://dummy.example.com",
            "dummy-audience",
            Duration::from_secs(600),
        )
        .expect("Failed to create JWKS verifier"),
    );
    // 共通AuthStateを使用して認証状態を構築
    let auth_state = AuthState { verifier };

    let state = AppState {
        manage_tables_uc,
        manage_columns_uc,
        crud_records_uc,
        manage_rules_uc,
        check_consistency_uc,
        get_audit_logs_uc,
        manage_relationships_uc,
        manage_display_configs_uc,
        import_export_uc,
        metrics,
        kafka_producer: None,
        auth_state: Some(auth_state),
    };
    router(state)
}

// --- テストケース ---

/// /healthz と /readyz への GET リクエストが 200 を返すことを検証する。
#[tokio::test]
async fn test_healthz_and_readyz() {
    // /healthz の検証
    let app = make_test_app();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz の検証
    let app = make_test_app();
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 保護されたエンドポイントに token なしでアクセスすると 401 が返ることを検証する。
#[tokio::test]
async fn test_unauthorized_without_token() {
    let app = make_test_app();
    // テーブル一覧エンドポイントに認証なしでアクセス
    let req = Request::builder()
        .uri("/api/v1/tables")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // 認証トークンがないため 401 が返る
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// /metrics エンドポイントが 200 を返すことを検証する。
#[tokio::test]
async fn test_metrics_returns_ok() {
    let app = make_test_app();
    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
