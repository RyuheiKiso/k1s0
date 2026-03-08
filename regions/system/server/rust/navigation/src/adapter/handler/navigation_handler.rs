use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;

use crate::adapter::presentation::NavigationResponseBody;

use super::AppState;

pub async fn get_navigation(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let bearer_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(extract_bearer_token)
        .unwrap_or_default();

    match state.get_navigation_uc.execute(bearer_token).await {
        Ok(result) => (StatusCode::OK, Json(NavigationResponseBody::from(result))).into_response(),
        Err(crate::usecase::get_navigation::NavigationError::TokenVerification(err)) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_NAVIGATION_UNAUTHENTICATED",
                    "message": err,
                }
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_NAVIGATION_INTERNAL_ERROR",
                    "message": err.to_string(),
                }
            })),
        )
            .into_response(),
    }
}

fn extract_bearer_token(value: &str) -> Option<&str> {
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    use crate::domain::entity::navigation::{Guard, GuardType, NavigationConfig, Route};
    use crate::infrastructure::navigation_loader::MockNavigationConfigLoader;
    use crate::usecase::get_navigation::MockNavigationTokenVerifier;
    use crate::usecase::GetNavigationUseCase;

    use super::super::{router, AppState};

    fn state_with_config(config: NavigationConfig) -> AppState {
        state_with_config_and_verifier(config, None)
    }

    fn state_with_config_and_verifier(
        config: NavigationConfig,
        verifier: Option<Arc<dyn crate::usecase::get_navigation::NavigationTokenVerifier>>,
    ) -> AppState {
        let mut mock = MockNavigationConfigLoader::new();
        mock.expect_load().return_once(move || Ok(config));

        AppState {
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("test-navigation")),
            get_navigation_uc: Arc::new(GetNavigationUseCase::new(Arc::new(mock), verifier)),
        }
    }

    #[tokio::test]
    async fn rest_endpoint_returns_public_navigation_without_token() {
        let app = router(state_with_config(NavigationConfig {
            version: 1,
            guards: vec![Guard {
                id: "auth_required".to_string(),
                guard_type: GuardType::AuthRequired,
                redirect_to: "/login".to_string(),
                roles: vec![],
            }],
            routes: vec![
                Route {
                    id: "public".to_string(),
                    path: "/".to_string(),
                    component_id: Some("PublicPage".to_string()),
                    guards: vec![],
                    transition: None,
                    transition_duration_ms: 300,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
                Route {
                    id: "private".to_string(),
                    path: "/dashboard".to_string(),
                    component_id: Some("DashboardPage".to_string()),
                    guards: vec!["auth_required".to_string()],
                    transition: None,
                    transition_duration_ms: 300,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
            ],
        }), true, "/metrics");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/navigation")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["routes"].as_array().unwrap().len(), 1);
        assert_eq!(json["routes"][0]["id"], "public");
    }

    #[tokio::test]
    async fn rest_endpoint_accepts_bearer_header() {
        let app = router(state_with_config(NavigationConfig {
            version: 1,
            guards: vec![],
            routes: vec![Route {
                id: "public".to_string(),
                path: "/".to_string(),
                component_id: Some("PublicPage".to_string()),
                guards: vec![],
                transition: None,
                transition_duration_ms: 300,
                redirect_to: None,
                children: vec![],
                params: vec![],
            }],
        }), true, "/metrics");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/navigation")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rest_endpoint_returns_unauthorized_for_invalid_token() {
        let mut mock_verifier = MockNavigationTokenVerifier::new();
        mock_verifier
            .expect_verify_roles()
            .withf(|token| token == "bad-token")
            .returning(|_| Err(anyhow::anyhow!("signature mismatch")));

        let app = router(
            state_with_config_and_verifier(
                NavigationConfig {
                    version: 1,
                    guards: vec![],
                    routes: vec![Route {
                        id: "public".to_string(),
                        path: "/".to_string(),
                        component_id: Some("PublicPage".to_string()),
                        guards: vec![],
                        transition: None,
                        transition_duration_ms: 300,
                        redirect_to: None,
                        children: vec![],
                        params: vec![],
                    }],
                },
                Some(Arc::new(mock_verifier)),
            ),
            true,
            "/metrics",
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/navigation")
                    .header(header::AUTHORIZATION, "Bearer bad-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_NAVIGATION_UNAUTHENTICATED");
    }
}
