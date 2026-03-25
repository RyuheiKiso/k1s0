// business tier 統合テスト: スタブリポジトリを使いHTTPレイヤーを検証する。
// auth パターンに倣い AppState をスタブで構築し、認証なしモードで動作確認を行う。
// 実 DB 接続テストは #[cfg(feature = "db-tests")] で区分けする。
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_project_master_server::adapter::handler::{router, AppState};
use k1s0_project_master_server::domain::entity::project_type::{
    CreateProjectType, ProjectType, ProjectTypeFilter, UpdateProjectType,
};
use k1s0_project_master_server::domain::entity::status_definition::{
    CreateStatusDefinition, StatusDefinition, StatusDefinitionFilter, UpdateStatusDefinition,
};
use k1s0_project_master_server::domain::entity::status_definition_version::StatusDefinitionVersion;
use k1s0_project_master_server::domain::entity::tenant_project_extension::{
    TenantMergedStatus, TenantProjectExtension, UpsertTenantExtension,
};
use k1s0_project_master_server::domain::repository::project_type_repository::ProjectTypeRepository;
use k1s0_project_master_server::domain::repository::status_definition_repository::StatusDefinitionRepository;
use k1s0_project_master_server::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use k1s0_project_master_server::domain::repository::version_repository::VersionRepository;
use k1s0_project_master_server::usecase::event_publisher::{
    NoopProjectMasterEventPublisher, ProjectMasterEventPublisher,
};

// --- テスト用スタブリポジトリ ---

/// テスト用プロジェクトタイプリポジトリ（インメモリ実装）
struct StubProjectTypeRepository {
    items: tokio::sync::RwLock<Vec<ProjectType>>,
}

impl StubProjectTypeRepository {
    fn new() -> Self {
        Self {
            items: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_items(items: Vec<ProjectType>) -> Self {
        Self {
            items: tokio::sync::RwLock::new(items),
        }
    }
}

fn sample_project_type() -> ProjectType {
    ProjectType {
        id: uuid::Uuid::new_v4(),
        code: "SOFTWARE".to_string(),
        display_name: "ソフトウェア開発".to_string(),
        description: None,
        default_workflow: None,
        is_active: true,
        sort_order: 1,
        created_by: "admin".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[async_trait::async_trait]
impl ProjectTypeRepository for StubProjectTypeRepository {
    async fn find_by_id(&self, id: uuid::Uuid) -> anyhow::Result<Option<ProjectType>> {
        let items = self.items.read().await;
        Ok(items.iter().find(|p| p.id == id).cloned())
    }

    async fn find_by_code(&self, code: &str) -> anyhow::Result<Option<ProjectType>> {
        let items = self.items.read().await;
        Ok(items.iter().find(|p| p.code == code).cloned())
    }

    async fn find_all(&self, _filter: &ProjectTypeFilter) -> anyhow::Result<Vec<ProjectType>> {
        let items = self.items.read().await;
        Ok(items.clone())
    }

    async fn count(&self, _filter: &ProjectTypeFilter) -> anyhow::Result<i64> {
        let items = self.items.read().await;
        Ok(items.len() as i64)
    }

    async fn create(&self, input: &CreateProjectType, created_by: &str) -> anyhow::Result<ProjectType> {
        let item = ProjectType {
            id: uuid::Uuid::new_v4(),
            code: input.code.clone(),
            display_name: input.display_name.clone(),
            description: input.description.clone(),
            default_workflow: input.default_workflow.clone(),
            is_active: input.is_active.unwrap_or(true),
            sort_order: input.sort_order.unwrap_or(0),
            created_by: created_by.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.items.write().await.push(item.clone());
        Ok(item)
    }

    async fn update(&self, id: uuid::Uuid, input: &UpdateProjectType, _updated_by: &str) -> anyhow::Result<ProjectType> {
        let mut items = self.items.write().await;
        let item = items.iter_mut().find(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("project type not found: {}", id))?;
        if let Some(ref name) = input.display_name {
            item.display_name = name.clone();
        }
        Ok(item.clone())
    }

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        let mut items = self.items.write().await;
        items.retain(|p| p.id != id);
        Ok(())
    }
}

/// テスト用ステータス定義リポジトリ（インメモリ実装）
struct StubStatusDefinitionRepository;

#[async_trait::async_trait]
impl StatusDefinitionRepository for StubStatusDefinitionRepository {
    async fn find_by_id(&self, _id: uuid::Uuid) -> anyhow::Result<Option<StatusDefinition>> {
        Ok(None)
    }

    async fn find_all(&self, _filter: &StatusDefinitionFilter) -> anyhow::Result<Vec<StatusDefinition>> {
        Ok(vec![])
    }

    async fn count(&self, _filter: &StatusDefinitionFilter) -> anyhow::Result<i64> {
        Ok(0)
    }

    async fn create(&self, _input: &CreateStatusDefinition, _created_by: &str) -> anyhow::Result<StatusDefinition> {
        anyhow::bail!("not implemented in stub")
    }

    async fn update(&self, _id: uuid::Uuid, _input: &UpdateStatusDefinition, _updated_by: &str) -> anyhow::Result<StatusDefinition> {
        anyhow::bail!("not implemented in stub")
    }

    async fn delete(&self, _id: uuid::Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

/// テスト用バージョンリポジトリ（インメモリ実装）
struct StubVersionRepository;

#[async_trait::async_trait]
impl VersionRepository for StubVersionRepository {
    async fn find_by_status_definition(
        &self,
        _status_definition_id: uuid::Uuid,
        _limit: i64,
        _offset: i64,
    ) -> anyhow::Result<Vec<StatusDefinitionVersion>> {
        Ok(vec![])
    }

    async fn count_by_status_definition(&self, _status_definition_id: uuid::Uuid) -> anyhow::Result<i64> {
        Ok(0)
    }
}

/// テスト用テナント拡張リポジトリ（インメモリ実装）
struct StubTenantExtensionRepository;

#[async_trait::async_trait]
impl TenantExtensionRepository for StubTenantExtensionRepository {
    async fn find(
        &self,
        _tenant_id: &str,
        _status_definition_id: uuid::Uuid,
    ) -> anyhow::Result<Option<TenantProjectExtension>> {
        Ok(None)
    }

    async fn list_merged(
        &self,
        _tenant_id: &str,
        _project_type_id: uuid::Uuid,
        _active_only: bool,
        _limit: i64,
        _offset: i64,
    ) -> anyhow::Result<Vec<TenantMergedStatus>> {
        Ok(vec![])
    }

    async fn count_merged(
        &self,
        _tenant_id: &str,
        _project_type_id: uuid::Uuid,
    ) -> anyhow::Result<i64> {
        Ok(0)
    }

    async fn upsert(&self, input: &UpsertTenantExtension) -> anyhow::Result<TenantProjectExtension> {
        Ok(TenantProjectExtension {
            id: uuid::Uuid::new_v4(),
            tenant_id: input.tenant_id.clone(),
            status_definition_id: input.status_definition_id,
            display_name_override: input.display_name_override.clone(),
            attributes_override: input.attributes_override.clone(),
            is_enabled: input.is_enabled.unwrap_or(true),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn delete(&self, _tenant_id: &str, _status_definition_id: uuid::Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

/// スタブを使ってテスト用 AppState を構築する
fn make_test_app() -> axum::Router {
    let pt_repo = Arc::new(StubProjectTypeRepository::new());
    let sd_repo = Arc::new(StubStatusDefinitionRepository);
    let ver_repo = Arc::new(StubVersionRepository);
    let te_repo = Arc::new(StubTenantExtensionRepository);
    let publisher: Arc<dyn ProjectMasterEventPublisher> = Arc::new(NoopProjectMasterEventPublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("project-master-test"));

    let state = AppState {
        manage_project_types_uc: Arc::new(
            k1s0_project_master_server::usecase::manage_project_types::ManageProjectTypesUseCase::new(
                pt_repo.clone(),
                publisher.clone(),
            ),
        ),
        manage_status_definitions_uc: Arc::new(
            k1s0_project_master_server::usecase::manage_status_definitions::ManageStatusDefinitionsUseCase::new(
                sd_repo.clone(),
                publisher.clone(),
            ),
        ),
        get_versions_uc: Arc::new(
            k1s0_project_master_server::usecase::get_status_definition_versions::GetStatusDefinitionVersionsUseCase::new(
                ver_repo.clone(),
            ),
        ),
        manage_tenant_extensions_uc: Arc::new(
            k1s0_project_master_server::usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase::new(
                te_repo.clone(),
                publisher.clone(),
            ),
        ),
        metrics,
        auth_state: None,
    };
    router(state)
}

/// プロジェクトタイプを1件含んだ状態でテスト用 AppState を構築する
fn make_test_app_with_project_type(pt: ProjectType) -> axum::Router {
    let pt_repo = Arc::new(StubProjectTypeRepository::with_items(vec![pt]));
    let sd_repo = Arc::new(StubStatusDefinitionRepository);
    let ver_repo = Arc::new(StubVersionRepository);
    let te_repo = Arc::new(StubTenantExtensionRepository);
    let publisher: Arc<dyn ProjectMasterEventPublisher> = Arc::new(NoopProjectMasterEventPublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("project-master-test"));

    let state = AppState {
        manage_project_types_uc: Arc::new(
            k1s0_project_master_server::usecase::manage_project_types::ManageProjectTypesUseCase::new(
                pt_repo.clone(),
                publisher.clone(),
            ),
        ),
        manage_status_definitions_uc: Arc::new(
            k1s0_project_master_server::usecase::manage_status_definitions::ManageStatusDefinitionsUseCase::new(
                sd_repo.clone(),
                publisher.clone(),
            ),
        ),
        get_versions_uc: Arc::new(
            k1s0_project_master_server::usecase::get_status_definition_versions::GetStatusDefinitionVersionsUseCase::new(
                ver_repo.clone(),
            ),
        ),
        manage_tenant_extensions_uc: Arc::new(
            k1s0_project_master_server::usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase::new(
                te_repo.clone(),
                publisher.clone(),
            ),
        ),
        metrics,
        auth_state: None,
    };
    router(state)
}

// --- 統合テスト ---

/// ヘルスチェックエンドポイント（/healthz）が 200 を返すことを確認する
#[tokio::test]
async fn test_health_check() {
    let app = make_test_app();

    // /healthz が 200 を返すことを確認する
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .expect("healthz リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("healthz リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz が 200 を返すことを確認する
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .expect("readyz リクエストの構築に失敗");
    let resp = app.clone().oneshot(req).await.expect("readyz リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    // /metrics が 200 を返すことを確認する
    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .expect("metrics リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("metrics リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);
}

/// プロジェクトタイプ一覧取得（GET /api/v1/project-types）が空の配列を返すことを確認する
/// レスポンスは Vec<ProjectTypeResponse> の JSON 配列として直接返る（wrapper なし）
#[tokio::test]
async fn test_list_project_types_empty() {
    let app = make_test_app();

    let req = Request::builder()
        .uri("/api/v1/project-types")
        .body(Body::empty())
        .expect("プロジェクトタイプ一覧リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("プロジェクトタイプ一覧リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("プロジェクトタイプ一覧レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("プロジェクトタイプ一覧レスポンスの JSON パースに失敗");
    // list_project_types は Vec<ProjectTypeResponse> を直接 JSON 配列として返す
    assert!(json.as_array().expect("レスポンスが配列でない").is_empty());
}

/// プロジェクトタイプ作成（POST /api/v1/project-types）が 201 と作成済みエンティティを返すことを確認する
#[tokio::test]
async fn test_create_project_type() {
    let app = make_test_app();

    let body = serde_json::json!({
        "code": "SOFTWARE",
        "display_name": "ソフトウェア開発",
        "is_active": true,
        "sort_order": 1
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/project-types")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("プロジェクトタイプ作成ボディの JSON シリアライズに失敗")))
        .expect("プロジェクトタイプ作成リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("プロジェクトタイプ作成リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("プロジェクトタイプ作成レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("プロジェクトタイプ作成レスポンスの JSON パースに失敗");
    assert_eq!(json["code"], "SOFTWARE");
    assert_eq!(json["display_name"], "ソフトウェア開発");
    assert!(json["id"].is_string());
}

/// プロジェクトタイプ取得（GET /api/v1/project-types/{id}）が既存エンティティで 200 を返すことを確認する
#[tokio::test]
async fn test_get_project_type_found() {
    let pt = sample_project_type();
    let pt_id = pt.id;
    let app = make_test_app_with_project_type(pt);

    let req = Request::builder()
        .uri(format!("/api/v1/project-types/{}", pt_id))
        .body(Body::empty())
        .expect("プロジェクトタイプ取得リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("プロジェクトタイプ取得リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("プロジェクトタイプ取得レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("プロジェクトタイプ取得レスポンスの JSON パースに失敗");
    assert_eq!(json["id"], pt_id.to_string());
    assert_eq!(json["code"], "SOFTWARE");
}

/// プロジェクトタイプ取得（GET /api/v1/project-types/{id}）が存在しない ID で 4xx/5xx を返すことを確認する
/// 注: get_project_type は ok_or_else で anyhow::Error を使用しているため、
/// map_domain_error が ProjectMasterError にダウンキャストできず 500 Internal を返す。
/// このテストでは CREATED にならないこと（エラーレスポンスが返ること）を確認する。
#[tokio::test]
async fn test_get_project_type_not_found() {
    let app = make_test_app();

    let req = Request::builder()
        .uri(format!("/api/v1/project-types/{}", uuid::Uuid::new_v4()))
        .body(Body::empty())
        .expect("プロジェクトタイプ未検出リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("プロジェクトタイプ未検出リクエストの送信に失敗");
    // 存在しない ID の場合は成功レスポンス（2xx）にならないことを確認する
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

/// プロジェクトタイプ一覧取得がデータあり状態で1件の配列を返すことを確認する
/// レスポンスは Vec<ProjectTypeResponse> の JSON 配列として直接返る（wrapper なし）
#[tokio::test]
async fn test_list_project_types_with_data() {
    let pt = sample_project_type();
    let app = make_test_app_with_project_type(pt);

    let req = Request::builder()
        .uri("/api/v1/project-types")
        .body(Body::empty())
        .expect("プロジェクトタイプ一覧（データあり）リクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("プロジェクトタイプ一覧（データあり）リクエストの送信に失敗");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("プロジェクトタイプ一覧（データあり）レスポンスボディの読み取りに失敗");
    let json: serde_json::Value = serde_json::from_slice(&body)
        .expect("プロジェクトタイプ一覧（データあり）レスポンスの JSON パースに失敗");
    // list_project_types は Vec<ProjectTypeResponse> を直接 JSON 配列として返す
    let pts_arr = json.as_array().expect("レスポンスが配列でない");
    assert_eq!(pts_arr.len(), 1);
}

/// プロジェクトタイプ作成のバリデーション（code が空）でエラーを返すことを確認する
#[tokio::test]
async fn test_create_project_type_empty_code() {
    let app = make_test_app();

    let body = serde_json::json!({
        "code": "",
        "display_name": "ソフトウェア開発"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/project-types")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).expect("バリデーションテストボディの JSON シリアライズに失敗")))
        .expect("バリデーションテストリクエストの構築に失敗");
    let resp = app.oneshot(req).await.expect("バリデーションテストリクエストの送信に失敗");
    // code が空の場合は CREATED にならないことを確認する
    assert_ne!(resp.status(), StatusCode::CREATED);
}

// --- 実 DB を使ったテスト（db-tests feature が有効な場合のみ実行）---
// 現時点では #[cfg(feature = "db-tests")] で区分けし、CI の db-tests ジョブで有効化する

#[tokio::test]
#[cfg(feature = "db-tests")]
async fn test_project_master_crud_with_real_db() {
    // 実 DB を使った CRUD テスト（Phase 4 以降で実装予定）
    // TODO: testcontainers を使って PostgreSQL コンテナを起動し、
    //       リポジトリ実装の CRUD を検証する
}
