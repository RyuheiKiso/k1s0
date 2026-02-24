use axum::response::IntoResponse;
use axum::Json;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "session"}))
}

pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready", "service": "session"}))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_check() {
        let app = Router::new().route("/healthz", get(super::healthz));
        let response = app
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "session");
    }

    #[tokio::test]
    async fn readyz_check() {
        let app = Router::new().route("/readyz", get(super::readyz));
        let response = app
            .oneshot(Request::builder().uri("/readyz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ready");
        assert_eq!(json["service"], "session");
    }
}
