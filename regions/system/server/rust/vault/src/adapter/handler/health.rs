use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::adapter::handler::AppState;

pub async fn healthz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
        })),
    )
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "ready",
                    "checks": {
                        "database": "ok",
                    }
                })),
            ),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "checks": {
                        "database": e.to_string(),
                    }
                })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "checks": {
                    "database": "not_configured",
                }
            })),
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_healthz_returns_ok() {
        let app = Router::new().route("/healthz", get(healthz));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
