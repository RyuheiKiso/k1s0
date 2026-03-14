pub mod ai_handler;
pub mod health;

pub use ai_handler::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

use crate::adapter::middleware::auth::auth_middleware;

/// REST APIルーターを構築する。
/// ヘルスチェック、メトリクス、AI APIエンドポイントを統合する。
pub fn router(state: AppState, metrics_enabled: bool, metrics_path: &str) -> Router {
    // 認証不要のエンドポイント
    let mut public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz));
    if metrics_enabled {
        public_routes = public_routes.route(metrics_path, get(metrics_handler));
    }

    // 認証が設定されている場合はRBAC付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        Router::new()
            .route("/api/v1/complete", post(ai_handler::complete))
            .route("/api/v1/embed", post(ai_handler::embed))
            .route("/api/v1/models", get(ai_handler::list_models))
            .route("/api/v1/usage", get(ai_handler::get_usage))
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）
        Router::new()
            .route("/api/v1/complete", post(ai_handler::complete))
            .route("/api/v1/embed", post(ai_handler::embed))
            .route("/api/v1/models", get(ai_handler::list_models))
            .route("/api/v1/usage", get(ai_handler::get_usage))
    };

    public_routes.merge(api_routes).with_state(state)
}

/// メトリクスハンドラー。Prometheus形式でメトリクスを返す。
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
