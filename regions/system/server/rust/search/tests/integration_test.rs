// k1s0-search-server の router 初期化 smoke test。
// healthz/readyz の疎通確認と、認証なしでの保護エンドポイントアクセスを検証する。
// search サーバーは router() 関数を公開していないため、startup.rs と同様に手動構築する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_search_server::adapter::handler::search_handler::{self, AppState};
use k1s0_search_server::domain::entity::search_index::{
    SearchDocument, SearchIndex, SearchQuery, SearchResult,
};
use k1s0_search_server::domain::repository::SearchRepository;
use k1s0_search_server::infrastructure::kafka_producer::NoopSearchEventPublisher;
use k1s0_search_server::usecase::{
    CreateIndexUseCase, DeleteDocumentUseCase, IndexDocumentUseCase, ListIndicesUseCase,
    SearchUseCase,
};

// --- テストダブル: SearchRepository のスタブ実装 ---

struct StubSearchRepository;

#[async_trait]
impl SearchRepository for StubSearchRepository {
    async fn create_index(&self, _index: &SearchIndex) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_index(&self, _name: &str) -> anyhow::Result<Option<SearchIndex>> {
        Ok(None)
    }

    async fn index_document(&self, _doc: &SearchDocument) -> anyhow::Result<()> {
        Ok(())
    }

    async fn search(&self, _query: &SearchQuery) -> anyhow::Result<SearchResult> {
        Ok(SearchResult {
            total: 0,
            hits: vec![],
            facets: std::collections::HashMap::new(),
            pagination: k1s0_search_server::domain::entity::search_index::PaginationResult {
                total_count: 0,
                page: 1,
                page_size: 10,
                has_next: false,
            },
        })
    }

    async fn delete_document(&self, _index_name: &str, _doc_id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>> {
        Ok(vec![])
    }
}

// --- テストアプリケーション構築 ---

/// スタブリポジトリを使い、startup.rs と同様のルーティングで Router を構築するヘルパー。
fn make_test_app() -> axum::Router {
    let repo: Arc<dyn SearchRepository> = Arc::new(StubSearchRepository);
    let event_publisher = Arc::new(NoopSearchEventPublisher);

    let state = AppState {
        search_uc: Arc::new(SearchUseCase::new(repo.clone())),
        index_document_uc: Arc::new(IndexDocumentUseCase::new(repo.clone(), event_publisher)),
        delete_document_uc: Arc::new(DeleteDocumentUseCase::new(repo.clone())),
        create_index_uc: Arc::new(CreateIndexUseCase::new(repo.clone())),
        list_indices_uc: Arc::new(ListIndicesUseCase::new(repo)),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-search-server-test",
        )),
        auth_state: None,
    };

    // startup.rs と同じ構成: public_routes + api_routes (auth_state=None)
    let public_routes = axum::Router::new()
        .route(
            "/healthz",
            axum::routing::get(k1s0_search_server::adapter::handler::health::healthz),
        )
        .route(
            "/readyz",
            axum::routing::get(k1s0_search_server::adapter::handler::health::readyz),
        );

    // 認証なしモードの API ルート
    let api_routes = axum::Router::new()
        .route(
            "/api/v1/search",
            axum::routing::post(search_handler::search),
        )
        .route(
            "/api/v1/search/index",
            axum::routing::post(search_handler::index_document),
        )
        .route(
            "/api/v1/search/index/{index_name}/{id}",
            axum::routing::delete(search_handler::delete_document_from_index),
        )
        .route(
            "/api/v1/search/indices",
            axum::routing::post(search_handler::create_index),
        )
        .route(
            "/api/v1/search/indices",
            axum::routing::get(search_handler::list_indices),
        );

    public_routes.merge(api_routes).with_state(state)
}

// --- テスト: /healthz と /readyz が 200 を返す ---

#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz への GET リクエストで 200 OK を確認
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz への GET リクエストで 200 OK を確認
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- テスト: 認証なしモード（auth_state=None）では保護エンドポイントに直接アクセス可能 ---

#[tokio::test]
async fn test_api_accessible_without_auth() {
    let app = make_test_app();

    // /api/v1/search/indices への GET が 200 を返す（認証なしモード）
    let req = Request::builder()
        .uri("/api/v1/search/indices")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
