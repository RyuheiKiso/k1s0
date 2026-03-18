// ドメインマスターサーバーの統合テスト
// router 初期化の smoke test として、ヘルスチェックと認証なしアクセスを検証する

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// ドメインマスターサーバーのクレートから必要な型をインポート
use k1s0_domain_master_server::adapter::handler::{router, AppState};
use k1s0_domain_master_server::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use k1s0_domain_master_server::domain::entity::master_item::{
    CreateMasterItem, MasterItem, UpdateMasterItem,
};
use k1s0_domain_master_server::domain::entity::master_item_version::MasterItemVersion;
use k1s0_domain_master_server::domain::entity::tenant_master_extension::{
    TenantMasterExtension, UpsertTenantMasterExtension,
};
use k1s0_domain_master_server::domain::repository::category_repository::CategoryRepository;
use k1s0_domain_master_server::domain::repository::item_repository::ItemRepository;
use k1s0_domain_master_server::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use k1s0_domain_master_server::domain::repository::version_repository::VersionRepository;
use k1s0_domain_master_server::usecase;
use k1s0_domain_master_server::usecase::event_publisher::DomainMasterEventPublisher;
use uuid::Uuid;

// --- テスト用スタブ: CategoryRepository ---

/// テスト用のカテゴリリポジトリ。全メソッドが空の結果を返す。
struct StubCategoryRepo;

#[async_trait]
impl CategoryRepository for StubCategoryRepo {
    async fn find_all(&self, _active_only: bool) -> anyhow::Result<Vec<MasterCategory>> {
        Ok(vec![])
    }
    async fn find_by_code(&self, _code: &str) -> anyhow::Result<Option<MasterCategory>> {
        Ok(None)
    }
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<MasterCategory>> {
        Ok(None)
    }
    async fn create(
        &self,
        _input: &CreateMasterCategory,
        _created_by: &str,
    ) -> anyhow::Result<MasterCategory> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(
        &self,
        _code: &str,
        _input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _code: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用スタブ: ItemRepository ---

/// テスト用のアイテムリポジトリ。全メソッドが空の結果を返す。
struct StubItemRepo;

#[async_trait]
impl ItemRepository for StubItemRepo {
    async fn find_by_category(
        &self,
        _category_id: Uuid,
        _active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        Ok(vec![])
    }
    async fn find_by_category_and_code(
        &self,
        _category_id: Uuid,
        _code: &str,
    ) -> anyhow::Result<Option<MasterItem>> {
        Ok(None)
    }
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<MasterItem>> {
        Ok(None)
    }
    async fn create(
        &self,
        _category_id: Uuid,
        _input: &CreateMasterItem,
        _created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update(&self, _id: Uuid, _input: &UpdateMasterItem) -> anyhow::Result<MasterItem> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用スタブ: TenantExtensionRepository ---

/// テスト用のテナント拡張リポジトリ。全メソッドが空の結果を返す。
struct StubTenantExtRepo;

#[async_trait]
impl TenantExtensionRepository for StubTenantExtRepo {
    async fn find_by_tenant_and_item(
        &self,
        _tenant_id: &str,
        _item_id: Uuid,
    ) -> anyhow::Result<Option<TenantMasterExtension>> {
        Ok(None)
    }
    async fn find_by_tenant_and_category(
        &self,
        _tenant_id: &str,
        _category_id: Uuid,
    ) -> anyhow::Result<Vec<TenantMasterExtension>> {
        Ok(vec![])
    }
    async fn upsert(
        &self,
        _tenant_id: &str,
        _item_id: Uuid,
        _input: &UpsertTenantMasterExtension,
    ) -> anyhow::Result<TenantMasterExtension> {
        anyhow::bail!("stub: not implemented")
    }
    async fn delete(&self, _tenant_id: &str, _item_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用スタブ: VersionRepository ---

/// テスト用のバージョンリポジトリ。全メソッドが空の結果を返す。
struct StubVersionRepo;

#[async_trait]
impl VersionRepository for StubVersionRepo {
    async fn find_by_item(&self, _item_id: Uuid) -> anyhow::Result<Vec<MasterItemVersion>> {
        Ok(vec![])
    }
    async fn get_latest_version_number(&self, _item_id: Uuid) -> anyhow::Result<i32> {
        Ok(0)
    }
    async fn create<'a>(
        &self,
        _item_id: Uuid,
        _version_number: i32,
        _before_data: Option<serde_json::Value>,
        _after_data: Option<serde_json::Value>,
        _changed_by: &'a str,
        _change_reason: Option<&'a str>,
    ) -> anyhow::Result<MasterItemVersion> {
        anyhow::bail!("stub: not implemented")
    }
}

// --- テスト用スタブ: DomainMasterEventPublisher ---

/// テスト用のイベントパブリッシャー。全イベントを破棄する。
struct StubEventPublisher;

#[async_trait]
impl DomainMasterEventPublisher for StubEventPublisher {
    async fn publish_category_changed(&self, _event: &serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }
    async fn publish_item_changed(&self, _event: &serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }
    async fn publish_tenant_extension_changed(
        &self,
        _event: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用アプリケーション構築 ---

/// テスト用の AppState を構築し、router を返すヘルパー関数。
/// 全リポジトリにスタブを使用し、認証は無効化する。
fn make_test_app() -> axum::Router {
    let cat_repo: Arc<dyn CategoryRepository> = Arc::new(StubCategoryRepo);
    let item_repo: Arc<dyn ItemRepository> = Arc::new(StubItemRepo);
    let tenant_ext_repo: Arc<dyn TenantExtensionRepository> = Arc::new(StubTenantExtRepo);
    let version_repo: Arc<dyn VersionRepository> = Arc::new(StubVersionRepo);
    let event_pub: Arc<dyn DomainMasterEventPublisher> = Arc::new(StubEventPublisher);

    // AppState の構築（認証なし）
    let state = AppState {
        manage_categories_uc: Arc::new(usecase::manage_categories::ManageCategoriesUseCase::new(
            cat_repo.clone(),
            event_pub.clone(),
        )),
        manage_items_uc: Arc::new(usecase::manage_items::ManageItemsUseCase::new(
            cat_repo.clone(),
            item_repo.clone(),
            version_repo.clone(),
            event_pub.clone(),
        )),
        get_item_versions_uc: Arc::new(usecase::get_item_versions::GetItemVersionsUseCase::new(
            cat_repo.clone(),
            item_repo.clone(),
            version_repo.clone(),
        )),
        manage_tenant_extensions_uc: Arc::new(
            usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase::new(
                cat_repo.clone(),
                item_repo.clone(),
                tenant_ext_repo,
                event_pub,
            ),
        ),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-domain-master-server-test",
        )),
        auth_state: None,
    };

    router(state)
}

// --- ヘルスチェックテスト ---

/// /healthz と /readyz エンドポイントが 200 OK を返すことを確認する
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz へのリクエスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz へのリクエスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- 認証なしアクセステスト ---

/// 認証が無効な状態で保護エンドポイントにアクセスすると正常にルーティングされることを確認する。
/// auth_state が None の場合、認証ミドルウェアはスキップされる。
#[tokio::test]
async fn test_api_routes_are_reachable() {
    let app = make_test_app();

    // 認証なしモードでは /api/v1/categories にアクセスできる
    let req = Request::builder()
        .uri("/api/v1/categories")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // ルーターが正常に応答すること（500 でないこと）を確認
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
