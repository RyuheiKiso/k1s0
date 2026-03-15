pub mod health;
pub mod navigation_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

#[derive(Clone)]
pub struct AppState {
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub get_navigation_uc: Arc<crate::usecase::GetNavigationUseCase>,
}

pub fn router(state: AppState, metrics_enabled: bool, metrics_path: &str) -> Router {
    let mut router = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route(
            "/api/v1/navigation",
            get(navigation_handler::get_navigation),
        );

    if metrics_enabled {
        router = router.route(metrics_path, get(metrics_handler));
    }

    router.with_state(state)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
