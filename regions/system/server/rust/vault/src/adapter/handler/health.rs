use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::adapter::handler::AppState;

/// ヘルスチェックエンドポイント: サーバーが起動しているかを返す
pub async fn healthz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
        })),
    )
}

/// レディネスチェックエンドポイント: バックエンド種別と疎通確認結果を返す。
/// DB/Vault 未構成（in-memory フォールバック）時は degraded ステータスを返して
/// 永続化バックエンド未構成であることを運用チームに明示する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref pool) = state.db_pool {
        // PostgreSQL または Vault KV v2 バックエンドが構成されている場合
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "ready",
                    "service": "vault",
                    "backend": "postgres",
                    "checks": {
                        "database": "ok",
                    }
                })),
            ),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "service": "vault",
                    "backend": "postgres",
                    "checks": {
                        "database": e.to_string(),
                    }
                })),
            ),
        }
    } else {
        // DB 未構成時は in-memory で動作中のため degraded を返す
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "degraded",
                "service": "vault",
                "backend": "in-memory",
                "reason": "永続化バックエンド未構成のため in-memory で動作中"
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
