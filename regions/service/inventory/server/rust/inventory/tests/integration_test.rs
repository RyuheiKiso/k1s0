// 在庫サーバーの統合テスト
// router 初期化の smoke test として、ヘルスチェックと認証なしアクセスを検証する

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// 在庫サーバーのクレートから必要な型をインポート
use k1s0_inventory_server::adapter::handler::{router, AppState};
use k1s0_inventory_server::domain::entity::inventory_item::{InventoryFilter, InventoryItem};
use k1s0_inventory_server::domain::entity::outbox::OutboxEvent;
use k1s0_inventory_server::domain::repository::inventory_repository::InventoryRepository;
use k1s0_inventory_server::usecase;
use uuid::Uuid;

// --- テスト用スタブ: InventoryRepository ---

/// テスト用の在庫リポジトリ。全メソッドが空の結果を返す。
struct StubInventoryRepo;

#[async_trait]
impl InventoryRepository for StubInventoryRepo {
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<InventoryItem>> {
        Ok(None)
    }
    async fn find_by_product_and_warehouse(
        &self,
        _product_id: &str,
        _warehouse_id: &str,
    ) -> anyhow::Result<Option<InventoryItem>> {
        Ok(None)
    }
    async fn find_all(&self, _filter: &InventoryFilter) -> anyhow::Result<Vec<InventoryItem>> {
        Ok(vec![])
    }
    async fn count(&self, _filter: &InventoryFilter) -> anyhow::Result<i64> {
        Ok(0)
    }
    async fn reserve_stock(
        &self,
        _id: Uuid,
        _quantity: i32,
        _expected_version: i32,
        _order_id: &str,
    ) -> anyhow::Result<InventoryItem> {
        anyhow::bail!("stub: not implemented")
    }
    async fn release_stock(
        &self,
        _id: Uuid,
        _quantity: i32,
        _expected_version: i32,
        _order_id: &str,
        _reason: &str,
    ) -> anyhow::Result<InventoryItem> {
        anyhow::bail!("stub: not implemented")
    }
    async fn update_stock(
        &self,
        _id: Uuid,
        _qty_available: i32,
        _expected_version: i32,
    ) -> anyhow::Result<InventoryItem> {
        anyhow::bail!("stub: not implemented")
    }
    async fn create(
        &self,
        _product_id: &str,
        _warehouse_id: &str,
        _qty_available: i32,
    ) -> anyhow::Result<InventoryItem> {
        anyhow::bail!("stub: not implemented")
    }
    async fn insert_outbox_event(
        &self,
        _aggregate_type: &str,
        _aggregate_id: &str,
        _event_type: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        Ok(vec![])
    }
    async fn mark_event_published(&self, _event_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用アプリケーション構築 ---

/// テスト用の AppState を構築し、router を返すヘルパー関数。
/// 全リポジトリにスタブを使用し、認証は無効化する。
fn make_test_app() -> axum::Router {
    let repo: Arc<dyn InventoryRepository> = Arc::new(StubInventoryRepo);

    // AppState の構築（認証なし、DB なし）
    let state = AppState {
        reserve_stock_uc: Arc::new(usecase::reserve_stock::ReserveStockUseCase::new(
            repo.clone(),
        )),
        release_stock_uc: Arc::new(usecase::release_stock::ReleaseStockUseCase::new(
            repo.clone(),
        )),
        get_inventory_uc: Arc::new(usecase::get_inventory::GetInventoryUseCase::new(
            repo.clone(),
        )),
        list_inventory_uc: Arc::new(usecase::list_inventory::ListInventoryUseCase::new(
            repo.clone(),
        )),
        update_stock_uc: Arc::new(usecase::update_stock::UpdateStockUseCase::new(repo.clone())),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-inventory-server-test",
        )),
        auth_state: None,
        db_pool: None,
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

    // 認証なしモードでは /api/v1/inventory にアクセスできる
    let req = Request::builder()
        .uri("/api/v1/inventory")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // ルーターが正常に応答すること（500 でないこと）を確認
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
