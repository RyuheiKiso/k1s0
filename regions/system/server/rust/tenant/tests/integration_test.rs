use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_tenant_server::adapter::handler::{self, AppState};

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let state = AppState::new("tenant-server".to_string(), "0.1.0".to_string());
    let app = handler::router(state);

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_returns_service_info() {
    let state = AppState::new("tenant-server".to_string(), "0.1.0".to_string());
    let app = handler::router(state);

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "tenant-server");
    assert_eq!(json["version"], "0.1.0");
}
