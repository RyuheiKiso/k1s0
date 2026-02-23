pub mod health;

use axum::{routing::get, Router};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub version: String,
}

impl AppState {
    pub fn new(app_name: String, version: String) -> Self {
        Self { app_name, version }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .with_state(Arc::new(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let state = AppState::new("tenant-server".to_string(), "0.1.0".to_string());
        let app = router(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
