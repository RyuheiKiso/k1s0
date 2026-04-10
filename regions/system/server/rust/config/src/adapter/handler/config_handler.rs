use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::AppState;
use crate::domain::entity::config_entry::{ConfigEntry, ConfigListResult, ServiceConfigResult};
use crate::usecase::list_configs::ListConfigsParams;
use crate::usecase::update_config::UpdateConfigInput;

/// システムテナントUUID: JWT Claims がない場合（dev モード / ヘルスチェック）のフォールバック値。
const SYSTEM_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

/// JWT クレームからテナントIDを抽出する。
/// クレームが存在しない場合、または `tenant_id` が無効な UUID の場合は 401 を返す。
// Option<&T> の方が &Option<T> よりも慣用的（Clippy: ref_option）
fn extract_tenant_id(
    claims: Option<&Extension<k1s0_auth::Claims>>,
) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    let tenant_id_str = claims
        .map(|Extension(c)| c.tenant_id.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Authentication required",
                    "code": 401
                })),
            )
        })?;

    Uuid::parse_str(tenant_id_str).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid tenant_id in JWT claims",
                "code": 401
            })),
        )
    })
}

#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "Health check OK")))]
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// # Panics
/// 定数 `SYSTEM_TENANT_ID` が有効な UUID でない場合にパニックする（設計上発生しない）。
#[utoipa::path(get, path = "/readyz", responses((status = 200, description = "Ready"), (status = 503, description = "Not ready")))]
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // システムテナントで軽量クエリを実行して DB 接続確認を行う
    let system_tenant =
        Uuid::parse_str(SYSTEM_TENANT_ID).expect("system tenant UUID must be valid");
    // MED-001 対応: .is_ok() でエラーを握り潰さず tracing::error! で詳細を記録する
    let db_ok = match state
        .config_repo
        .list_by_namespace(system_tenant, "__readyz__", 1, 1, None)
        .await
    {
        Ok(_) => true,
        Err(e) => {
            tracing::error!(error = %e, "readyz: DB health check failed");
            false
        }
    };

    let db_status = if db_ok { "ok" } else { "error" };

    // Kafka: 設定済みかどうか（プロデューサーは起動時に初期化済み）
    let kafka_status = if state.kafka_configured {
        "ok"
    } else {
        "not_configured"
    };

    let all_ok = db_ok && (!state.kafka_configured || kafka_status == "ok");
    // ADR-0068 準拠: "healthy"/"unhealthy" を使用する（MED-006 監査対応）
    let status = if all_ok { "healthy" } else { "unhealthy" };
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path((namespace, key)): Path<(String, String)>,
) -> impl IntoResponse {
    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id(claims.as_ref()) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    match state
        .get_config_uc
        .execute(tenant_id, &namespace, &key)
        .await
    {
        Ok(entry) => {
            let resp = serde_json::json!({
                "id": entry.id,
                "namespace": entry.namespace,
                "key": entry.key,
                "value": entry.value_json,
                "version": entry.version,
                "description": entry.description,
                "created_by": entry.created_by,
                "updated_by": entry.updated_by,
                "created_at": entry.created_at,
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(namespace): Path<String>,
    Query(params): Query<ListConfigsParams>,
) -> impl IntoResponse {
    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id(claims.as_ref()) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    match state
        .list_configs_uc
        .execute(tenant_id, &namespace, &params)
        .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
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

#[derive(Debug, Deserialize)]
pub struct ServiceConfigQuery {
    pub environment: Option<String>,
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
    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id(claims.as_ref()) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let updated_by = claims
        .and_then(|Extension(c)| c.preferred_username.clone())
        .unwrap_or_else(|| "api-user".to_string());

    let input = UpdateConfigInput {
        tenant_id,
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path((namespace, key)): Path<(String, String)>,
) -> impl IntoResponse {
    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id(claims.as_ref()) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let deleted_by = claims
        .and_then(|Extension(c)| c.preferred_username.clone())
        .unwrap_or_else(|| "api-user".to_string());

    match state
        .delete_config_uc
        .execute(tenant_id, &namespace, &key, &deleted_by)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/config/services/{service_name}",
    params(
        ("service_name" = String, Path, description = "Service name"),
        ("environment" = Option<String>, Query, description = "Environment filter"),
    ),
    responses(
        (status = 200, description = "Service config", body = ServiceConfigResult),
        (status = 404, description = "Service not found"),
    )
)]
pub async fn get_service_config(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(service_name): Path<String>,
    Query(query): Query<ServiceConfigQuery>,
) -> impl IntoResponse {
    // テナントIDが無効な場合は 401 を返す
    let tenant_id = match extract_tenant_id(claims.as_ref()) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    match state
        .get_service_config_uc
        .execute(tenant_id, &service_name)
        .await
    {
        Ok(mut result) => {
            if let Some(environment) = query.environment {
                result
                    .entries
                    .retain(|entry| entry.namespace.contains(&environment));
                if result.entries.is_empty() {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(super::ErrorResponse::new(
                            k1s0_server_common::error::config::service_not_found().as_str(),
                            format!(
                                "service config not found for {service_name} in environment {environment}"
                            ),
                        )),
                    )
                        .into_response();
                }
            }
            (StatusCode::OK, Json(result)).into_response()
        }
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
    use crate::domain::error::ConfigRepositoryError;
    use crate::domain::repository::config_repository::MockConfigRepository;
    use crate::domain::repository::config_schema_repository::MockConfigSchemaRepository;
    use axum::body::Body;
    use axum::http::Request;
    use chrono::Utc;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    /// テスト用の有効なテナントUUID
    const TEST_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

    /// テスト用の有効なJWT Claimsを作成するヘルパー。
    /// リクエストに認証情報を注入する際に使用する。
    fn make_test_claims() -> k1s0_auth::Claims {
        k1s0_auth::Claims {
            sub: "test-user".to_string(),
            iss: "https://auth.example.com".to_string(),
            aud: k1s0_auth::Audience(vec!["test".to_string()]),
            exp: 9999999999,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("test-user".to_string()),
            email: None,
            realm_access: None,
            resource_access: None,
            tier_access: None,
            tenant_id: TEST_TENANT_ID.to_string(),
        }
    }

    fn make_test_entry() -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: "認証サーバーの DB 最大接続数".to_string(),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_app_state(mock: MockConfigRepository) -> AppState {
        let mut schema_mock = MockConfigSchemaRepository::new();
        // CRITICAL-RUST-001 監査対応: tenant_id パラメータが追加されたため 2 引数のクロージャを使用する
        schema_mock
            .expect_find_by_namespace()
            .returning(|_, _| Ok(None));
        AppState::new(Arc::new(mock), Arc::new(schema_mock))
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_app_state(MockConfigRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .expect("request build should succeed");

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_readyz() {
        let mut mock = MockConfigRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む5引数シグネチャ
        mock.expect_list_by_namespace()
            .returning(|_, _, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![],
                    pagination: Pagination {
                        total_count: 0,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });
        let state = make_app_state(mock);
        let app = router(state);

        let req = Request::builder()
            .uri("/readyz")
            .body(Body::empty())
            .expect("request build should succeed");

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        // ADR-0068 準拠: "healthy" が正しい値（MED-006 監査対応）
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["checks"]["database"], "ok");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = make_app_state(MockConfigRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .expect("request build should succeed");

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        // STATIC-CRITICAL-001: tenant_id を含む3引数シグネチャ
        mock.expect_find_by_namespace_and_key()
            .withf(|_tid, ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(move |_, _, _| Ok(Some(entry.clone())));

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .uri("/api/v1/config/system.auth.database/max_connections")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["namespace"], "system.auth.database");
        assert_eq!(json["key"], "max_connections");
        assert_eq!(json["value"], 25);
        assert_eq!(json["version"], 3);
    }

    #[tokio::test]
    async fn test_get_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _, _| Ok(None));

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .uri("/api/v1/config/nonexistent.namespace/missing_key")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_list_configs_success() {
        let mut mock = MockConfigRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む5引数シグネチャ
        mock.expect_list_by_namespace()
            .returning(|_, _, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![ConfigEntry {
                        id: Uuid::new_v4(),
                        namespace: "system.auth.database".to_string(),
                        key: "max_connections".to_string(),
                        value_json: serde_json::json!(25),
                        version: 3,
                        description: "DB max conns".to_string(),
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

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .uri("/api/v1/config/system.auth.database?page=1&page_size=20")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(
            json["entries"]
                .as_array()
                .expect("entries should be an array")
                .len(),
            1
        );
        assert_eq!(json["pagination"]["total_count"], 1);
    }

    #[tokio::test]
    async fn test_update_config_success() {
        let mut mock = MockConfigRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む3引数シグネチャ
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _, _| Ok(Some(make_test_entry())));
        // STATIC-CRITICAL-001: tenant_id を含む7引数シグネチャ
        mock.expect_update()
            .returning(|_, ns, key, value, _, desc, updated_by| {
                Ok(ConfigEntry {
                    id: Uuid::new_v4(),
                    namespace: ns.to_string(),
                    key: key.to_string(),
                    value_json: value.clone(),
                    version: 4,
                    description: desc.unwrap_or_default(),
                    created_by: "admin@example.com".to_string(),
                    updated_by: updated_by.to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            });
        mock.expect_record_change_log().returning(|_| Ok(()));

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .method("PUT")
            .uri("/api/v1/config/system.auth.database/max_connections")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"value":50,"version":3,"description":"増設"}"#,
            ))
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["value"], 50);
        assert_eq!(json["version"], 4);
    }

    #[tokio::test]
    async fn test_update_config_version_conflict() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _, _| Ok(Some(make_test_entry())));
        // STATIC-CRITICAL-001: 7引数シグネチャ
        mock.expect_update().returning(|_, _, _, _, _, _, _| {
            Err(ConfigRepositoryError::VersionConflict {
                expected: 3,
                current: 4,
            })
        });

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .method("PUT")
            .uri("/api/v1/config/system.auth.database/max_connections")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"value":50,"version":3}"#))
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["error"]["code"], "SYS_CONFIG_VERSION_CONFLICT");
    }

    #[tokio::test]
    async fn test_delete_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(move |_, _, _| Ok(Some(entry.clone())));
        // STATIC-CRITICAL-001: tenant_id を含む3引数シグネチャ
        mock.expect_delete()
            .withf(|_tid, ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _, _| Ok(true));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/config/system.auth.database/max_connections")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _, _| Ok(None));
        mock.expect_delete().returning(|_, _, _| Ok(false));

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/config/nonexistent.namespace/missing_key")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_get_service_config_success() {
        let mut mock = MockConfigRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_service_name()
            .withf(|_tid, name| name == "auth-server")
            .returning(|_, _| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "max_connections".to_string(),
                            value: serde_json::json!(25),
                            version: 3,
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "ssl_mode".to_string(),
                            value: serde_json::json!("require"),
                            version: 1,
                        },
                    ],
                })
            });

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .uri("/api/v1/config/services/auth-server")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["service_name"], "auth-server");
        assert_eq!(
            json["entries"]
                .as_array()
                .expect("entries should be an array")
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn test_get_service_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name().returning(|_, _| {
            Err(ConfigRepositoryError::ServiceNotFound(
                "nonexistent-service".to_string(),
            ))
        });

        let state = make_app_state(mock);
        let app = router(state);

        // 有効なJWT Claimsを注入してリクエストを送信する
        let mut req = Request::builder()
            .uri("/api/v1/config/services/nonexistent-service")
            .body(Body::empty())
            .expect("request build should succeed");
        req.extensions_mut().insert(make_test_claims());

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["error"]["code"], "SYS_CONFIG_SERVICE_NOT_FOUND");
    }

    /// JWT クレームなし（認証情報未設定）の場合に 401 を返すことを確認するテスト。
    /// これにより SYSTEM テナントへのフォールバックが廃止されたことを検証する。
    #[tokio::test]
    async fn test_get_config_unauthorized_no_claims() {
        let state = make_app_state(MockConfigRepository::new());
        let app = router(state);

        // JWT Claimsを注入せずにリクエストを送信する
        let req = Request::builder()
            .uri("/api/v1/config/system.auth.database/max_connections")
            .body(Body::empty())
            .expect("request build should succeed");

        let resp = app
            .oneshot(req)
            .await
            .expect("request build should succeed");
        // クレームなしの場合は 401 が返されることを確認する
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("request build should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("request build should succeed");
        assert_eq!(json["error"], "Authentication required");
        assert_eq!(json["code"], 401);
    }
}
