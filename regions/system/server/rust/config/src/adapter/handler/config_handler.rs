use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;

use super::AppState;
use crate::domain::entity::config_entry::{ConfigEntry, ConfigListResult, ServiceConfigResult};
use crate::usecase::list_configs::ListConfigsParams;
use crate::usecase::update_config::UpdateConfigInput;

#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "Health check OK")))]
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(get, path = "/readyz", responses((status = 200, description = "Ready"), (status = 503, description = "Not ready")))]
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // DB 接続確認: 軽量クエリで疎通チェック
    let db_ok = state
        .config_repo
        .list_by_namespace("__readyz__", 1, 1, None)
        .await
        .is_ok();

    let db_status = if db_ok { "ok" } else { "error" };

    // Kafka: 設定済みかどうか（プロデューサーは起動時に初期化済み）
    let kafka_status = if state.kafka_configured {
        "ok"
    } else {
        "not_configured"
    };

    let all_ok = db_ok;
    let status = if all_ok { "ready" } else { "not_ready" };
    let code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "checks": {
                "database": db_status,
                "kafka": kafka_status
            }
        })),
    )
}

#[utoipa::path(get, path = "/metrics", responses((status = 200, description = "Prometheus metrics")))]
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/config/{namespace}/{key}",
    params(
        ("namespace" = String, Path, description = "Config namespace"),
        ("key" = String, Path, description = "Config key"),
    ),
    responses(
        (status = 200, description = "Config entry found", body = ConfigEntry),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.get_config_uc.execute(&namespace, &key).await {
        Ok(entry) => {
            let resp = serde_json::json!({
                "namespace": entry.namespace,
                "key": entry.key,
                "value": entry.value_json,
                "version": entry.version,
                "description": entry.description,
                "updated_by": entry.updated_by,
                "updated_at": entry.updated_at,
            });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/config/{namespace}",
    params(
        ("namespace" = String, Path, description = "Config namespace"),
        ("page" = Option<i32>, Query, description = "Page number"),
        ("page_size" = Option<i32>, Query, description = "Page size"),
    ),
    responses((status = 200, description = "Config list", body = ConfigListResult))
)]
pub async fn list_configs(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
    Query(params): Query<ListConfigsParams>,
) -> impl IntoResponse {
    match state.list_configs_uc.execute(&namespace, &params).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response(),
        Err(e) => e.into_response(),
    }
}

/// PUT /api/v1/config/:namespace/:key のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateConfigRequest {
    pub value: serde_json::Value,
    pub version: i32,
    #[serde(default)]
    pub description: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/config/{namespace}/{key}",
    params(
        ("namespace" = String, Path, description = "Config namespace"),
        ("key" = String, Path, description = "Config key"),
    ),
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Config updated", body = ConfigEntry),
        (status = 404, description = "Not found"),
        (status = 409, description = "Version conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_config(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path((namespace, key)): Path<(String, String)>,
    Json(req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    let updated_by = claims
        .and_then(|Extension(c)| c.preferred_username.clone())
        .unwrap_or_else(|| "api-user".to_string());

    let input = UpdateConfigInput {
        namespace,
        key,
        value: req.value,
        version: req.version,
        description: req.description,
        updated_by,
    };

    match state.update_config_uc.execute(&input).await {
        Ok(entry) => {
            let resp = serde_json::json!({
                "namespace": entry.namespace,
                "key": entry.key,
                "value": entry.value_json,
                "version": entry.version,
                "description": entry.description,
                "updated_by": entry.updated_by,
                "updated_at": entry.updated_at,
            });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/config/{namespace}/{key}",
    params(
        ("namespace" = String, Path, description = "Config namespace"),
        ("key" = String, Path, description = "Config key"),
    ),
    responses(
        (status = 204, description = "Config deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.delete_config_uc.execute(&namespace, &key).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/config/services/{service_name}",
    params(("service_name" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "Service config", body = ServiceConfigResult),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn get_service_config(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> impl IntoResponse {
    match state.get_service_config_uc.execute(&service_name).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response(),
        Err(e) => e.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler::router;
    use crate::domain::entity::config_entry::{
        ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
    };
    use crate::domain::repository::config_repository::MockConfigRepository;
    use axum::body::Body;
    use axum::http::Request;
    use chrono::Utc;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn make_test_entry() -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: Some("認証サーバーの DB 最大接続数".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_app_state(mock: MockConfigRepository) -> AppState {
        AppState::new(Arc::new(mock))
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_app_state(MockConfigRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_readyz() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .returning(|_, _, _, _| {
                Ok(ConfigListResult {
                    entries: vec![],
                    pagination: Pagination {
                        total_count: 0,
                        page: 1,
                        page_size: 1,
                        has_next: false,
                    },
                })
            });
        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/readyz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ready");
        assert_eq!(json["checks"]["database"], "ok");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = make_app_state(MockConfigRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        mock.expect_find_by_namespace_and_key()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(move |_, _| Ok(Some(entry.clone())));

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/config/system.auth.database/max_connections")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["namespace"], "system.auth.database");
        assert_eq!(json["key"], "max_connections");
        assert_eq!(json["value"], 25);
        assert_eq!(json["version"], 3);
    }

    #[tokio::test]
    async fn test_get_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/config/nonexistent.namespace/missing_key")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_list_configs_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![ConfigEntry {
                        id: Uuid::new_v4(),
                        namespace: "system.auth.database".to_string(),
                        key: "max_connections".to_string(),
                        value_json: serde_json::json!(25),
                        version: 3,
                        description: Some("DB max conns".to_string()),
                        created_by: "admin@example.com".to_string(),
                        updated_by: "admin@example.com".to_string(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }],
                    pagination: Pagination {
                        total_count: 1,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/config/system.auth.database?page=1&page_size=20")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["entries"].as_array().unwrap().len(), 1);
        assert_eq!(json["pagination"]["total_count"], 1);
    }

    #[tokio::test]
    async fn test_update_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_update()
            .returning(|ns, key, value, _, desc, updated_by| {
                Ok(ConfigEntry {
                    id: Uuid::new_v4(),
                    namespace: ns.to_string(),
                    key: key.to_string(),
                    value_json: value.clone(),
                    version: 4,
                    description: desc,
                    created_by: "admin@example.com".to_string(),
                    updated_by: updated_by.to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            });

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .method("PUT")
            .uri("/api/v1/config/system.auth.database/max_connections")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"value":50,"version":3,"description":"増設"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["value"], 50);
        assert_eq!(json["version"], 4);
    }

    #[tokio::test]
    async fn test_update_config_version_conflict() {
        let mut mock = MockConfigRepository::new();
        mock.expect_update()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("version conflict: current=4")));

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .method("PUT")
            .uri("/api/v1/config/system.auth.database/max_connections")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"value":50,"version":3}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_CONFIG_VERSION_CONFLICT");
    }

    #[tokio::test]
    async fn test_delete_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _| Ok(true));

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/config/system.auth.database/max_connections")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_delete().returning(|_, _| Ok(false));

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/config/nonexistent.namespace/missing_key")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_get_service_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .withf(|name| name == "auth-server")
            .returning(|_| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "max_connections".to_string(),
                            value: serde_json::json!(25),
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "ssl_mode".to_string(),
                            value: serde_json::json!("require"),
                        },
                    ],
                })
            });

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/config/services/auth-server")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["service_name"], "auth-server");
        assert_eq!(json["entries"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_service_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .returning(|_| Err(anyhow::anyhow!("service not found")));

        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/config/services/nonexistent-service")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_CONFIG_SERVICE_NOT_FOUND");
    }
}
