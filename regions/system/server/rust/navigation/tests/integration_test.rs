// ナビゲーションサーバーの統合テスト。
// router の初期化と基本的なエンドポイントの動作を検証する。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_navigation_server::adapter::handler::{router, AppState};
use k1s0_navigation_server::domain::entity::navigation::NavigationConfig;
use k1s0_navigation_server::infrastructure::navigation_loader::NavigationConfigLoader;
use k1s0_navigation_server::usecase::GetNavigationUseCase;

// --- テストダブル ---

/// テスト用のナビゲーション設定ローダー。空の設定を返す。
struct StubNavigationConfigLoader;

impl NavigationConfigLoader for StubNavigationConfigLoader {
    fn load(&self) -> anyhow::Result<NavigationConfig> {
        Ok(NavigationConfig {
            version: 1,
            guards: vec![],
            routes: vec![],
        })
    }
}

/// テスト用の AppState を構築し、router を生成するヘルパー関数。
fn make_test_app() -> axum::Router {
    let loader: Arc<dyn NavigationConfigLoader> = Arc::new(StubNavigationConfigLoader);
    let get_navigation_uc = Arc::new(GetNavigationUseCase::new(loader, None));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("navigation-test"));
    let state = AppState {
        metrics,
        get_navigation_uc,
    };
    router(state, false, "/metrics")
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

/// 保護されたエンドポイントに token なしでアクセスした場合のレスポンスを検証する。
/// ナビゲーションサーバーは認証ミドルウェアを持たないため、token なしでも
/// エンドポイント自体にはアクセスできる（空のナビゲーション結果が返る）。
#[tokio::test]
async fn test_navigation_endpoint_without_token() {
    let app = make_test_app();
    let req = Request::builder()
        .uri("/api/v1/navigation")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // 認証なしモードでは 200 が返る（空のナビゲーション設定）
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 存在しないエンドポイントへのアクセスが 404 を返すことを検証する。
#[tokio::test]
async fn test_nonexistent_endpoint_returns_404() {
    let app = make_test_app();
    let req = Request::builder()
        .uri("/api/v1/nonexistent")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
