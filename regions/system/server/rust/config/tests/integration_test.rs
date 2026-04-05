#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_config_server::adapter::handler::{router, AppState};

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
use k1s0_config_server::domain::entity::config_change_log::ConfigChangeLog;
use k1s0_config_server::domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use k1s0_config_server::domain::entity::config_schema::ConfigSchema;
use k1s0_config_server::domain::error::ConfigRepositoryError;
use k1s0_config_server::domain::repository::{ConfigRepository, ConfigSchemaRepository};

/// テスト用のインメモリリポジトリ実装。
struct TestConfigRepository {
    entries: RwLock<Vec<ConfigEntry>>,
}

impl TestConfigRepository {
    fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    fn with_entries(entries: Vec<ConfigEntry>) -> Self {
        Self {
            entries: RwLock::new(entries),
        }
    }
}

#[async_trait]
impl ConfigRepository for TestConfigRepository {
    /// namespace と key で設定値を取得する（テスト用インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、テスト用のため分離しない。
    async fn find_by_namespace_and_key(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<Option<ConfigEntry>, ConfigRepositoryError> {
        let entries = self.entries.read().await;
        Ok(entries
            .iter()
            .find(|e| e.namespace == namespace && e.key == key)
            .cloned())
    }

    /// namespace 内の設定値一覧を取得する（テスト用インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、テスト用のため分離しない。
    async fn list_by_namespace(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> Result<ConfigListResult, ConfigRepositoryError> {
        let entries = self.entries.read().await;
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|e| {
                if e.namespace != namespace {
                    return false;
                }
                if let Some(ref s) = search {
                    if !e.key.contains(s) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total_count = filtered.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;

        filtered = filtered.into_iter().skip(offset).take(limit).collect();
        let has_next = (offset + limit) < total_count as usize;

        Ok(ConfigListResult {
            entries: filtered,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    /// 設定値を更新する（テスト用インメモリ実装、楽観的排他制御付き）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、テスト用のため分離しない。
    async fn update(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> Result<ConfigEntry, ConfigRepositoryError> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .iter_mut()
            .find(|e| e.namespace == namespace && e.key == key);

        match entry {
            Some(e) => {
                // バージョン不一致: 楽観的排他制御エラー
                if e.version != expected_version {
                    return Err(ConfigRepositoryError::VersionConflict {
                        expected: expected_version,
                        current: e.version,
                    });
                }
                e.value_json = value_json.clone();
                e.version += 1;
                if let Some(desc) = description {
                    e.description = desc;
                }
                e.updated_by = updated_by.to_string();
                e.updated_at = chrono::Utc::now();
                Ok(e.clone())
            }
            // キーが存在しない: NotFound エラー
            None => Err(ConfigRepositoryError::NotFound {
                namespace: namespace.to_string(),
                key: key.to_string(),
            }),
        }
    }

    /// 設定値を削除する（テスト用インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、テスト用のため分離しない。
    async fn delete(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<bool, ConfigRepositoryError> {
        let mut entries = self.entries.write().await;
        let len_before = entries.len();
        entries.retain(|e| !(e.namespace == namespace && e.key == key));
        Ok(entries.len() < len_before)
    }

    /// サービス名に紐づく設定値を一括取得する（テスト用インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、テスト用のため分離しない。
    async fn find_by_service_name(
        &self,
        _tenant_id: Uuid,
        service_name: &str,
    ) -> Result<ServiceConfigResult, ConfigRepositoryError> {
        let entries = self.entries.read().await;
        // サービス名からキーワードを抽出してマッチング（テスト用の簡易実装）
        // 例: "auth-server" -> "auth" -> namespace に "auth" セグメントを含むものにマッチ
        let primary_keyword = service_name.split('-').next().unwrap_or(service_name);
        let matched: Vec<ServiceConfigEntry> = entries
            .iter()
            .filter(|e| {
                e.namespace
                    .split('.')
                    .any(|ns_part| ns_part == primary_keyword)
            })
            .map(|e| ServiceConfigEntry {
                namespace: e.namespace.clone(),
                key: e.key.clone(),
                value: e.value_json.clone(),
                version: e.version,
            })
            .collect();

        if matched.is_empty() {
            return Err(ConfigRepositoryError::ServiceNotFound(
                service_name.to_string(),
            ));
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries: matched,
        })
    }

    /// 設定変更ログを記録する（テスト用、何もしない）。
    async fn record_change_log(&self, _log: &ConfigChangeLog) -> Result<(), ConfigRepositoryError> {
        Ok(())
    }
}

fn make_test_entry(
    namespace: &str,
    key: &str,
    value: serde_json::Value,
    version: i32,
) -> ConfigEntry {
    ConfigEntry {
        id: Uuid::new_v4(),
        namespace: namespace.to_string(),
        key: key.to_string(),
        value_json: value,
        version,
        description: format!("{}/{}", namespace, key),
        created_by: "admin@example.com".to_string(),
        updated_by: "admin@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

struct TestConfigSchemaRepository;

#[async_trait]
impl ConfigSchemaRepository for TestConfigSchemaRepository {
    async fn find_by_service_name(
        &self,
        _service_name: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        Ok(None)
    }

    async fn find_by_namespace(&self, _namespace: &str) -> anyhow::Result<Option<ConfigSchema>> {
        Ok(None)
    }

    async fn list_all(&self) -> anyhow::Result<Vec<ConfigSchema>> {
        Ok(vec![])
    }

    async fn upsert(&self, schema: &ConfigSchema) -> anyhow::Result<ConfigSchema> {
        Ok(schema.clone())
    }
}

struct StaticConfigSchemaRepository {
    schemas: Vec<ConfigSchema>,
}

impl StaticConfigSchemaRepository {
    fn new(schemas: Vec<ConfigSchema>) -> Self {
        Self { schemas }
    }
}

#[async_trait]
impl ConfigSchemaRepository for StaticConfigSchemaRepository {
    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        Ok(self
            .schemas
            .iter()
            .find(|schema| schema.service_name == service_name)
            .cloned())
    }

    async fn find_by_namespace(&self, namespace: &str) -> anyhow::Result<Option<ConfigSchema>> {
        Ok(self
            .schemas
            .iter()
            .find(|schema| namespace.starts_with(&schema.namespace_prefix))
            .cloned())
    }

    async fn list_all(&self) -> anyhow::Result<Vec<ConfigSchema>> {
        Ok(self.schemas.clone())
    }

    async fn upsert(&self, schema: &ConfigSchema) -> anyhow::Result<ConfigSchema> {
        Ok(schema.clone())
    }
}

fn make_app_with_entries(entries: Vec<ConfigEntry>) -> axum::Router {
    let repo = Arc::new(TestConfigRepository::with_entries(entries));
    let schema_repo = Arc::new(TestConfigSchemaRepository);
    let state = AppState::new(repo, schema_repo);
    router(state)
}

fn make_empty_app() -> axum::Router {
    let repo = Arc::new(TestConfigRepository::new());
    let schema_repo = Arc::new(TestConfigSchemaRepository);
    let state = AppState::new(repo, schema_repo);
    router(state)
}

fn make_app_with_schemas(schemas: Vec<ConfigSchema>) -> axum::Router {
    let repo = Arc::new(TestConfigRepository::new());
    let schema_repo = Arc::new(StaticConfigSchemaRepository::new(schemas));
    let state = AppState::new(repo, schema_repo);
    router(state)
}

fn make_test_schema(service_name: &str, namespace_prefix: &str) -> ConfigSchema {
    ConfigSchema {
        id: Uuid::new_v4(),
        service_name: service_name.to_string(),
        namespace_prefix: namespace_prefix.to_string(),
        schema_json: serde_json::json!({
            "categories": [{
                "id": "database",
                "label": "Database",
                "icon": "storage",
                "namespaces": [format!("{}.database", namespace_prefix)],
                "fields": [{
                    "key": "max_connections",
                    "label": "Max Connections",
                    "type": 2,
                    "default_value": 25
                }]
            }]
        }),
        updated_by: "admin@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_full_config_get() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "max_connections",
        serde_json::json!(25),
        3,
    )];
    let app = make_app_with_entries(entries);

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .uri("/api/v1/config/system.auth.database/max_connections")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

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
async fn test_get_config_not_found_returns_404() {
    let app = make_empty_app();

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .uri("/api/v1/config/nonexistent.namespace/missing_key")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
}

#[tokio::test]
async fn test_list_configs_with_pagination() {
    let entries = vec![
        make_test_entry(
            "system.auth.database",
            "max_connections",
            serde_json::json!(25),
            1,
        ),
        make_test_entry(
            "system.auth.database",
            "ssl_mode",
            serde_json::json!("require"),
            1,
        ),
        make_test_entry(
            "system.auth.database",
            "pool_timeout",
            serde_json::json!(30),
            1,
        ),
        make_test_entry(
            "system.auth.jwt",
            "issuer",
            serde_json::json!("https://auth.example.com"),
            1,
        ),
    ];
    let app = make_app_with_entries(entries);

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .uri("/api/v1/config/system.auth.database?page=1&page_size=2")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["entries"].as_array().unwrap().len(), 2);
    assert_eq!(json["pagination"]["total_count"], 3);
    assert_eq!(json["pagination"]["has_next"], true);
}

#[tokio::test]
async fn test_get_config_schema_returns_public_dto_contract() {
    let app = make_app_with_schemas(vec![make_test_schema("auth-server", "system.auth")]);

    let req = Request::builder()
        .uri("/api/v1/config-schema/auth-server")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["service"], "auth-server");
    assert_eq!(json["namespace_prefix"], "system.auth");
    assert_eq!(json["categories"][0]["fields"][0]["type"], "integer");
    assert_eq!(json["categories"][0]["fields"][0]["default"], 25);
    assert!(json.get("service_name").is_none());
    assert!(json.get("schema_json").is_none());
    assert!(json.get("updated_by").is_none());
}

#[tokio::test]
async fn test_list_config_schemas_returns_public_dto_contract() {
    let app = make_app_with_schemas(vec![
        make_test_schema("auth-server", "system.auth"),
        make_test_schema("notification-server", "system.notification"),
    ]);

    let req = Request::builder()
        .uri("/api/v1/config-schema")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json.as_array().unwrap().len(), 2);
    assert_eq!(json[0]["service"], "auth-server");
    assert_eq!(json[1]["service"], "notification-server");
    assert!(json[0].get("service_name").is_none());
    assert!(json[0].get("schema_json").is_none());
}

#[tokio::test]
async fn test_update_config_with_correct_version() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "max_connections",
        serde_json::json!(25),
        3,
    )];
    let app = make_app_with_entries(entries);

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .method("PUT")
        .uri("/api/v1/config/system.auth.database/max_connections")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"value":50,"version":3,"description":"増設"}"#,
        ))
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

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
async fn test_update_config_version_conflict_returns_409() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "max_connections",
        serde_json::json!(25),
        5, // current version is 5
    )];
    let app = make_app_with_entries(entries);

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .method("PUT")
        .uri("/api/v1/config/system.auth.database/max_connections")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"value":50,"version":3}"#)) // expects version 3
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_CONFIG_VERSION_CONFLICT");
}

#[tokio::test]
async fn test_update_config_not_found_returns_404() {
    let app = make_empty_app();

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .method("PUT")
        .uri("/api/v1/config/system.auth.database/missing_key")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"value":"test","version":1}"#))
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
}

#[tokio::test]
async fn test_delete_config_returns_204() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "deprecated_setting",
        serde_json::json!("old_value"),
        1,
    )];
    let app = make_app_with_entries(entries);

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/config/system.auth.database/deprecated_setting")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_config_not_found_returns_404() {
    let app = make_empty_app();

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/config/nonexistent/missing")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
}

#[tokio::test]
async fn test_get_service_config_returns_all_entries() {
    let entries = vec![
        make_test_entry(
            "system.auth.database",
            "max_connections",
            serde_json::json!(25),
            1,
        ),
        make_test_entry(
            "system.auth.database",
            "ssl_mode",
            serde_json::json!("require"),
            1,
        ),
        make_test_entry(
            "system.auth.jwt",
            "issuer",
            serde_json::json!("https://auth.example.com"),
            1,
        ),
        make_test_entry(
            "system.config.internal",
            "cache_ttl",
            serde_json::json!(300),
            1,
        ),
    ];
    let app = make_app_with_entries(entries);

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .uri("/api/v1/config/services/auth-server")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["service_name"], "auth-server");
    // "auth" matches both "system.auth.database" and "system.auth.jwt"
    assert!(json["entries"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_service_config_not_found() {
    let app = make_empty_app();

    // 有効なJWT Claimsを注入してリクエストを送信する
    let mut req = Request::builder()
        .uri("/api/v1/config/services/nonexistent-service")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(make_test_claims());

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_CONFIG_SERVICE_NOT_FOUND");
}

#[tokio::test]
async fn test_healthz_returns_ok() {
    let app = make_empty_app();

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

/// ADR-0068 準拠: readyz エンドポイントは "healthy"/"unhealthy" を返す（MED-006 監査対応）。
#[tokio::test]
async fn test_readyz_returns_ready() {
    let app = make_empty_app();

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
    // ADR-0068 準拠: "healthy" が正しい値（"ready" ではない）
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["checks"]["database"], "ok");
    assert_eq!(json["checks"]["kafka"], "not_configured");
}

#[tokio::test]
async fn test_nonexistent_endpoint_returns_404() {
    let app = make_empty_app();

    let req = Request::builder()
        .uri("/api/v1/nonexistent")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
