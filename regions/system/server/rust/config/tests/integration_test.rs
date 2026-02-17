use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_config_server::adapter::handler::{router, AppState};
use k1s0_config_server::domain::entity::config_change_log::ConfigChangeLog;
use k1s0_config_server::domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use k1s0_config_server::domain::repository::ConfigRepository;

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
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries
            .iter()
            .find(|e| e.namespace == namespace && e.key == key)
            .cloned())
    }

    async fn list_by_namespace(
        &self,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> anyhow::Result<ConfigListResult> {
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

    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<ConfigEntry> {
        let mut entries = self.entries.write().await;
        entries.push(entry.clone());
        Ok(entry.clone())
    }

    async fn update(
        &self,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> anyhow::Result<ConfigEntry> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .iter_mut()
            .find(|e| e.namespace == namespace && e.key == key);

        match entry {
            Some(e) => {
                if e.version != expected_version {
                    return Err(anyhow::anyhow!("version conflict: current={}", e.version));
                }
                e.value_json = value_json.clone();
                e.version += 1;
                if let Some(desc) = description {
                    e.description = Some(desc);
                }
                e.updated_by = updated_by.to_string();
                e.updated_at = chrono::Utc::now();
                Ok(e.clone())
            }
            None => Err(anyhow::anyhow!("config not found: {}/{}", namespace, key)),
        }
    }

    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<bool> {
        let mut entries = self.entries.write().await;
        let len_before = entries.len();
        entries.retain(|e| !(e.namespace == namespace && e.key == key));
        Ok(entries.len() < len_before)
    }

    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<ServiceConfigResult> {
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
            })
            .collect();

        if matched.is_empty() {
            return Err(anyhow::anyhow!("service not found: {}", service_name));
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries: matched,
        })
    }

    async fn record_change_log(&self, _log: &ConfigChangeLog) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<ConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.iter().find(|e| e.id == *id).cloned())
    }
}

fn make_test_entry(namespace: &str, key: &str, value: serde_json::Value, version: i32) -> ConfigEntry {
    ConfigEntry {
        id: Uuid::new_v4(),
        namespace: namespace.to_string(),
        key: key.to_string(),
        value_json: value,
        version,
        description: Some(format!("{}/{}", namespace, key)),
        created_by: "admin@example.com".to_string(),
        updated_by: "admin@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_app_with_entries(entries: Vec<ConfigEntry>) -> axum::Router {
    let repo = Arc::new(TestConfigRepository::with_entries(entries));
    let state = AppState::new(repo);
    router(state)
}

fn make_empty_app() -> axum::Router {
    let repo = Arc::new(TestConfigRepository::new());
    let state = AppState::new(repo);
    router(state)
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
async fn test_get_config_not_found_returns_404() {
    let app = make_empty_app();

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
async fn test_list_configs_with_pagination() {
    let entries = vec![
        make_test_entry("system.auth.database", "max_connections", serde_json::json!(25), 1),
        make_test_entry("system.auth.database", "ssl_mode", serde_json::json!("require"), 1),
        make_test_entry("system.auth.database", "pool_timeout", serde_json::json!(30), 1),
        make_test_entry("system.auth.jwt", "issuer", serde_json::json!("https://auth.example.com"), 1),
    ];
    let app = make_app_with_entries(entries);

    let req = Request::builder()
        .uri("/api/v1/config/system.auth.database?page=1&page_size=2")
        .body(Body::empty())
        .unwrap();

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
async fn test_update_config_with_correct_version() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "max_connections",
        serde_json::json!(25),
        3,
    )];
    let app = make_app_with_entries(entries);

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
async fn test_update_config_version_conflict_returns_409() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "max_connections",
        serde_json::json!(25),
        5, // current version is 5
    )];
    let app = make_app_with_entries(entries);

    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/config/system.auth.database/max_connections")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"value":50,"version":3}"#)) // expects version 3
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
async fn test_update_config_not_found_returns_404() {
    let app = make_empty_app();

    let req = Request::builder()
        .method("PUT")
        .uri("/api/v1/config/nonexistent.namespace/missing_key")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"value":"test","version":1}"#))
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
async fn test_delete_config_returns_204() {
    let entries = vec![make_test_entry(
        "system.auth.database",
        "deprecated_setting",
        serde_json::json!("old_value"),
        1,
    )];
    let app = make_app_with_entries(entries);

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/config/system.auth.database/deprecated_setting")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_config_not_found_returns_404() {
    let app = make_empty_app();

    let req = Request::builder()
        .method("DELETE")
        .uri("/api/v1/config/nonexistent/missing")
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
async fn test_get_service_config_returns_all_entries() {
    let entries = vec![
        make_test_entry("system.auth.database", "max_connections", serde_json::json!(25), 1),
        make_test_entry("system.auth.database", "ssl_mode", serde_json::json!("require"), 1),
        make_test_entry("system.auth.jwt", "issuer", serde_json::json!("https://auth.example.com"), 1),
        make_test_entry("system.config.internal", "cache_ttl", serde_json::json!(300), 1),
    ];
    let app = make_app_with_entries(entries);

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
    // "auth" matches both "system.auth.database" and "system.auth.jwt"
    assert!(json["entries"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_service_config_not_found() {
    let app = make_empty_app();

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
    assert_eq!(json["status"], "ready");
    assert_eq!(json["checks"]["database"], "ok");
    assert_eq!(json["checks"]["kafka"], "ok");
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
