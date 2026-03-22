#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_app_registry::adapter::handler::{router, AppState, ValidateTokenUseCase};
use k1s0_app_registry::domain::entity::app::App;
use k1s0_app_registry::domain::entity::claims::{Claims, RealmAccess};
use k1s0_app_registry::domain::entity::platform::Platform;
use k1s0_app_registry::domain::entity::version::AppVersion;
use k1s0_app_registry::domain::repository::{
    AppRepository, DownloadStatsRepository, VersionRepository,
};
use k1s0_app_registry::infrastructure::file_storage::FileStorage;
use k1s0_app_registry::infrastructure::TokenVerifier;

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

struct TestTokenVerifier {
    should_succeed: bool,
}

#[async_trait]
impl TokenVerifier for TestTokenVerifier {
    async fn verify_token(&self, _token: &str) -> anyhow::Result<Claims> {
        if self.should_succeed {
            let now = chrono::Utc::now().timestamp();
            Ok(Claims {
                sub: "test-user-1".to_string(),
                iss: "test-issuer".to_string(),
                aud: "test-audience".to_string(),
                exp: now + 3600,
                iat: now,
                realm_access: RealmAccess {
                    roles: vec!["user".to_string(), "sys_operator".to_string()],
                },
                tier_access: vec!["system".to_string()],
                ..Default::default()
            })
        } else {
            Err(anyhow::anyhow!("token verification failed"))
        }
    }
}

struct TestAppRepository {
    apps: tokio::sync::RwLock<Vec<App>>,
}

impl TestAppRepository {
    fn new() -> Self {
        Self {
            apps: tokio::sync::RwLock::new(vec![]),
        }
    }

    fn with_apps(apps: Vec<App>) -> Self {
        Self {
            apps: tokio::sync::RwLock::new(apps),
        }
    }
}

#[async_trait]
impl AppRepository for TestAppRepository {
    async fn list(
        &self,
        _category: Option<String>,
        _search: Option<String>,
    ) -> anyhow::Result<Vec<App>> {
        Ok(self.apps.read().await.clone())
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<App>> {
        let apps = self.apps.read().await;
        Ok(apps.iter().find(|a| a.id == id).cloned())
    }

    async fn create(&self, app: &App) -> anyhow::Result<App> {
        let mut apps = self.apps.write().await;
        apps.push(app.clone());
        Ok(app.clone())
    }

    async fn update(&self, app: &App) -> anyhow::Result<App> {
        let mut apps = self.apps.write().await;
        if let Some(existing) = apps.iter_mut().find(|a| a.id == app.id) {
            *existing = app.clone();
        }
        Ok(app.clone())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut apps = self.apps.write().await;
        let len_before = apps.len();
        apps.retain(|a| a.id != id);
        Ok(apps.len() < len_before)
    }
}

struct TestVersionRepository {
    versions: tokio::sync::RwLock<Vec<AppVersion>>,
}

impl TestVersionRepository {
    fn new() -> Self {
        Self {
            versions: tokio::sync::RwLock::new(vec![]),
        }
    }

    fn with_versions(versions: Vec<AppVersion>) -> Self {
        Self {
            versions: tokio::sync::RwLock::new(versions),
        }
    }
}

#[async_trait]
impl VersionRepository for TestVersionRepository {
    async fn list_by_app(&self, app_id: &str) -> anyhow::Result<Vec<AppVersion>> {
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .filter(|v| v.app_id == app_id)
            .cloned()
            .collect())
    }

    async fn create(&self, version: &AppVersion) -> anyhow::Result<AppVersion> {
        let mut versions = self.versions.write().await;
        versions.push(version.clone());
        Ok(version.clone())
    }

    async fn delete(
        &self,
        app_id: &str,
        version: &str,
        platform: &Platform,
        arch: &str,
    ) -> anyhow::Result<()> {
        let mut versions = self.versions.write().await;
        versions.retain(|v| {
            !(v.app_id == app_id
                && v.version == version
                && v.platform == *platform
                && v.arch == arch)
        });
        Ok(())
    }
}

struct TestDownloadStatsRepository;

#[async_trait]
impl DownloadStatsRepository for TestDownloadStatsRepository {
    async fn record(
        &self,
        _stat: &k1s0_app_registry::domain::entity::download_stat::DownloadStat,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn count_by_app(&self, _app_id: &str) -> anyhow::Result<i64> {
        Ok(0)
    }

    async fn count_by_version(&self, _app_id: &str, _version: &str) -> anyhow::Result<i64> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

async fn make_test_app_with_repos(
    token_success: bool,
    app_repo: Arc<dyn AppRepository>,
    version_repo: Arc<dyn VersionRepository>,
    download_stats_repo: Arc<dyn DownloadStatsRepository>,
) -> axum::Router {
    // テスト用: 一時ディレクトリをストレージルートとして使用する
    let file_storage = Arc::new(FileStorage::new(std::env::temp_dir().join("k1s0-test")));
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    let state = AppState {
        list_apps_uc: Arc::new(k1s0_app_registry::usecase::ListAppsUseCase::new(
            app_repo.clone(),
        )),
        get_app_uc: Arc::new(k1s0_app_registry::usecase::GetAppUseCase::new(
            app_repo.clone(),
        )),
        create_app_uc: Arc::new(k1s0_app_registry::usecase::CreateAppUseCase::new(
            app_repo.clone(),
        )),
        update_app_uc: Arc::new(k1s0_app_registry::usecase::UpdateAppUseCase::new(
            app_repo.clone(),
        )),
        delete_app_uc: Arc::new(k1s0_app_registry::usecase::DeleteAppUseCase::new(
            app_repo.clone(),
        )),
        list_versions_uc: Arc::new(k1s0_app_registry::usecase::ListVersionsUseCase::new(
            app_repo.clone(),
            version_repo.clone(),
        )),
        create_version_uc: Arc::new(k1s0_app_registry::usecase::CreateVersionUseCase::new(
            version_repo.clone(),
        )),
        delete_version_uc: Arc::new(k1s0_app_registry::usecase::DeleteVersionUseCase::new(
            app_repo.clone(),
            version_repo.clone(),
        )),
        get_latest_uc: Arc::new(k1s0_app_registry::usecase::GetLatestUseCase::new(
            app_repo.clone(),
            version_repo.clone(),
        )),
        get_download_stats_uc: Arc::new(k1s0_app_registry::usecase::GetDownloadStatsUseCase::new(
            app_repo.clone(),
            version_repo.clone(),
            download_stats_repo.clone(),
        )),
        generate_download_url_uc: Arc::new(
            k1s0_app_registry::usecase::GenerateDownloadUrlUseCase::new(
                app_repo.clone(),
                version_repo.clone(),
                download_stats_repo.clone(),
                file_storage,
            ),
        ),
        validate_token_uc: Arc::new(ValidateTokenUseCase::new(
            Arc::new(TestTokenVerifier {
                should_succeed: token_success,
            }),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        )),
        metrics,
        db_pool: None,
    };
    router(state)
}

async fn make_test_app(token_success: bool) -> axum::Router {
    make_test_app_with_repos(
        token_success,
        Arc::new(TestAppRepository::new()),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_app(id: &str, name: &str) -> App {
    App {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!("{} description", name)),
        category: "tools".to_string(),
        icon_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn sample_version(app_id: &str, version: &str) -> AppVersion {
    AppVersion {
        id: uuid::Uuid::new_v4(),
        app_id: app_id.to_string(),
        version: version.to_string(),
        platform: Platform::Linux,
        arch: "amd64".to_string(),
        size_bytes: Some(10_000_000),
        checksum_sha256: "abc123def456".to_string(),
        storage_key: format!("{}/{}/linux/amd64/binary", app_id, version),
        release_notes: Some("Initial release".to_string()),
        mandatory: false,
        published_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    }
}

fn auth_header() -> (&'static str, &'static str) {
    ("Authorization", "Bearer test-token")
}

async fn response_body_string(resp: axum::http::Response<Body>) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app(true).await;

    // GET /healthz → 200
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // GET /readyz → 200 (no DB configured, so db_pool=None → skipped → overall ok)
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_apps() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let apps_arr = json["apps"].as_array().unwrap();
    assert_eq!(apps_arr.len(), 1);
    assert_eq!(apps_arr[0]["id"], "cli");
    assert_eq!(apps_arr[0]["name"], "k1s0 CLI");
}

#[tokio::test]
async fn test_get_app_found() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/cli")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["id"], "cli");
    assert_eq!(json["name"], "k1s0 CLI");
}

#[tokio::test]
async fn test_get_app_not_found() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/nonexistent")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_versions() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let versions = vec![sample_version("cli", "1.0.0")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::with_versions(versions)),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/cli/versions")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let versions_arr = json["versions"].as_array().unwrap();
    assert_eq!(versions_arr.len(), 1);
    assert_eq!(versions_arr[0]["version"], "1.0.0");
}

#[tokio::test]
async fn test_get_latest() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let versions = vec![sample_version("cli", "1.0.0")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::with_versions(versions)),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/cli/latest")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["version"], "1.0.0");
    assert_eq!(json["app_id"], "cli");
}

#[tokio::test]
async fn test_unauthorized_without_token() {
    let app = make_test_app(true).await;

    // No Authorization header → 401
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_with_invalid_token() {
    let app = make_test_app(false).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// アプリ作成テスト
// ---------------------------------------------------------------------------

/// アプリを POST で新規作成できることを検証する。
#[tokio::test]
async fn test_create_app_success() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let input = serde_json::json!({
        "name": "New CLI Tool",
        "description": "A new command line tool",
        "category": "tools"
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("content-type", "application/json")
                .header(key, val)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["name"], "New CLI Tool");
    assert_eq!(json["category"], "tools");
}

/// 名前が空のアプリ作成はバリデーションエラーになる。
#[tokio::test]
async fn test_create_app_empty_name_returns_error() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let input = serde_json::json!({
        "name": "",
        "category": "tools"
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("content-type", "application/json")
                .header(key, val)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    // バリデーションエラーは 400 系のレスポンスになる
    assert!(resp.status().is_client_error());
}

/// カテゴリが空のアプリ作成はバリデーションエラーになる。
#[tokio::test]
async fn test_create_app_empty_category_returns_error() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let input = serde_json::json!({
        "name": "Valid Name",
        "category": " "
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/apps")
                .header("content-type", "application/json")
                .header(key, val)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
}

// ---------------------------------------------------------------------------
// アプリ更新テスト
// ---------------------------------------------------------------------------

/// アプリを PUT で更新できることを検証する。
#[tokio::test]
async fn test_update_app() {
    let apps = vec![sample_app("update-me", "Old Name")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let input = serde_json::json!({
        "name": "Updated Name",
        "description": "Updated description",
        "category": "utilities"
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/apps/update-me")
                .header("content-type", "application/json")
                .header(key, val)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["name"], "Updated Name");
}

/// 存在しないアプリの更新は 404 になる。
#[tokio::test]
async fn test_update_app_not_found() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let input = serde_json::json!({
        "name": "Updated",
        "category": "tools"
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/apps/nonexistent-id")
                .header("content-type", "application/json")
                .header(key, val)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// アプリ削除テスト
// ---------------------------------------------------------------------------

/// アプリを DELETE で削除できることを検証する。
#[tokio::test]
async fn test_delete_app_success() {
    let apps = vec![sample_app("delete-me", "Delete Target")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/apps/delete-me")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // delete_app ハンドラーは NO_CONTENT (204) を返す
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

/// 存在しないアプリの削除は 404 になる。
#[tokio::test]
async fn test_delete_app_not_found() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/apps/no-such-app")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// バージョン関連テスト
// ---------------------------------------------------------------------------

/// 複数バージョンのリスト取得を検証する。
#[tokio::test]
async fn test_list_multiple_versions() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let versions = vec![
        sample_version("cli", "1.0.0"),
        sample_version("cli", "1.1.0"),
        sample_version("cli", "2.0.0"),
    ];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::with_versions(versions)),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/cli/versions")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let versions_arr = json["versions"].as_array().unwrap();
    assert_eq!(versions_arr.len(), 3);
}

/// 存在しないアプリのバージョンリスト取得は 404 になる。
#[tokio::test]
async fn test_list_versions_app_not_found() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/nonexistent/versions")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// バージョンが無いアプリの latest 取得は 404 になる。
#[tokio::test]
async fn test_get_latest_no_versions_returns_404() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/cli/latest")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// 存在しないアプリの latest は 404 になる。
#[tokio::test]
async fn test_get_latest_app_not_found() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/nonexistent/latest")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// アプリ一覧のフィルタリングテスト
// ---------------------------------------------------------------------------

/// 空のアプリリストは空配列を返す。
#[tokio::test]
async fn test_list_apps_empty() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let apps_arr = json["apps"].as_array().unwrap();
    assert!(apps_arr.is_empty());
}

/// 複数アプリが正しくリストされる。
#[tokio::test]
async fn test_list_multiple_apps() {
    let apps = vec![
        sample_app("cli", "k1s0 CLI"),
        sample_app("web", "k1s0 Web"),
        sample_app("mobile", "k1s0 Mobile"),
    ];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::new()),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = response_body_string(resp).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let apps_arr = json["apps"].as_array().unwrap();
    assert_eq!(apps_arr.len(), 3);
}

// ---------------------------------------------------------------------------
// ダウンロード統計テスト
// ---------------------------------------------------------------------------

/// ダウンロード統計エンドポイントが正常にレスポンスを返す。
#[tokio::test]
async fn test_get_download_stats() {
    let apps = vec![sample_app("cli", "k1s0 CLI")];
    let versions = vec![sample_version("cli", "1.0.0")];
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestAppRepository::with_apps(apps)),
        Arc::new(TestVersionRepository::with_versions(versions)),
        Arc::new(TestDownloadStatsRepository),
    )
    .await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/cli/stats")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 存在しないアプリのダウンロード統計は 404 になる。
#[tokio::test]
async fn test_get_download_stats_app_not_found() {
    let app = make_test_app(true).await;

    let (key, val) = auth_header();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps/nonexistent/stats")
                .header(key, val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// 認証バリエーションテスト
// ---------------------------------------------------------------------------

/// 不正な Authorization ヘッダー形式は 401 になる。
#[tokio::test]
async fn test_malformed_auth_header() {
    let app = make_test_app(true).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/apps")
                .header("Authorization", "InvalidScheme token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// healthz/readyz は認証不要であることを確認する。
#[tokio::test]
async fn test_health_endpoints_no_auth_required() {
    let app = make_test_app(false).await;

    // healthz は認証不要
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // readyz は認証不要
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
