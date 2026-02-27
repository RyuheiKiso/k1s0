use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::path::PathBuf;

use super::AppState;
use crate::domain::entity::navigation::NavigationConfig;

/// GET /api/v1/navigation
///
/// navigation.yaml を読み込みナビゲーション設定を返す。
/// 公開エンドポイント（認証不要）。
#[utoipa::path(
    get,
    path = "/api/v1/navigation",
    responses(
        (status = 200, description = "Navigation config"),
        (status = 500, description = "Failed to load navigation config"),
    )
)]
pub async fn get_navigation(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config_path = state
        .navigation_config_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("config/navigation.yaml"));

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to read navigation config: {}", e);
            let err = super::ErrorResponse::new(
                "SYS_NAV_CONFIG_READ_ERROR",
                "Failed to read navigation config",
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(err).unwrap())).into_response();
        }
    };

    let config: NavigationConfig = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to parse navigation config: {}", e);
            let err = super::ErrorResponse::new(
                "SYS_NAV_CONFIG_PARSE_ERROR",
                "Failed to parse navigation config",
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(err).unwrap())).into_response();
        }
    };

    (StatusCode::OK, Json(serde_json::to_value(config).unwrap())).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler::router;
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use crate::domain::repository::user_repository::MockUserRepository;
    use crate::infrastructure::MockTokenVerifier;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_navigation_success() {
        let state = {
            use crate::domain::repository::api_key_repository::MockApiKeyRepository;
            AppState::new(
                Arc::new(MockTokenVerifier::new()),
                Arc::new(MockUserRepository::new()),
                Arc::new(MockAuditLogRepository::new()),
                Arc::new(MockApiKeyRepository::new()),
                "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                "k1s0-api".to_string(),
                None,
                None,
            )
        };

        // テスト用のnavigation.yamlパスを設定
        let mut state = state;
        state.navigation_config_path = Some(PathBuf::from(
            concat!(env!("CARGO_MANIFEST_DIR"), "/config/navigation.yaml"),
        ));

        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/navigation")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["version"], 1);
        assert!(json["guards"].as_array().unwrap().len() >= 2);
        assert!(json["routes"].as_array().unwrap().len() >= 4);
    }

    #[tokio::test]
    async fn test_get_navigation_file_not_found() {
        let state = {
            use crate::domain::repository::api_key_repository::MockApiKeyRepository;
            AppState::new(
                Arc::new(MockTokenVerifier::new()),
                Arc::new(MockUserRepository::new()),
                Arc::new(MockAuditLogRepository::new()),
                Arc::new(MockApiKeyRepository::new()),
                "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                "k1s0-api".to_string(),
                None,
                None,
            )
        };

        let mut state = state;
        state.navigation_config_path = Some(PathBuf::from("/nonexistent/navigation.yaml"));

        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/navigation")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_NAV_CONFIG_READ_ERROR");
    }
}
