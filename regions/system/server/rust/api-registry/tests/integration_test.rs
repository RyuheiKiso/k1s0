// router 初期化と基本エンドポイントの smoke test
// api-registry サーバーの REST API ルーターが正しく構築され、
// ヘルスチェックおよび認証ミドルウェアが期待どおり動作することを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_api_registry_server::adapter::handler::{router, AppState};
use k1s0_api_registry_server::adapter::middleware::auth::ApiRegistryAuthState;
use k1s0_api_registry_server::domain::entity::api_registration::{
    ApiSchema, ApiSchemaVersion,
};
use k1s0_api_registry_server::domain::repository::{
    ApiSchemaRepository, ApiSchemaVersionRepository,
};
use k1s0_api_registry_server::usecase::*;

// ---------------------------------------------------------------------------
// テスト用スタブ: ApiSchemaRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubSchemaRepo;

#[async_trait]
impl ApiSchemaRepository for StubSchemaRepo {
    async fn find_by_name(&self, _name: &str) -> anyhow::Result<Option<ApiSchema>> {
        Ok(None)
    }
    async fn find_all(
        &self,
        _schema_type: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _schema: &ApiSchema) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _schema: &ApiSchema) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: ApiSchemaVersionRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubVersionRepo;

#[async_trait]
impl ApiSchemaVersionRepository for StubVersionRepo {
    async fn find_by_name_and_version(
        &self,
        _name: &str,
        _version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        Ok(None)
    }
    async fn find_latest_by_name(
        &self,
        _name: &str,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        Ok(None)
    }
    async fn find_all_by_name(
        &self,
        _name: &str,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _version: &ApiSchemaVersion) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _name: &str, _version: u32) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn count_by_name(&self, _name: &str) -> anyhow::Result<u64> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// テスト用アプリケーション構築ヘルパー（認証なしモード）
// ---------------------------------------------------------------------------
fn make_test_app() -> axum::Router {
    let schema_repo: Arc<dyn ApiSchemaRepository> = Arc::new(StubSchemaRepo);
    let version_repo: Arc<dyn ApiSchemaVersionRepository> = Arc::new(StubVersionRepo);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 各ユースケースをスタブリポジトリで構築
    let state = AppState {
        list_schemas_uc: Arc::new(ListSchemasUseCase::new(schema_repo.clone())),
        register_schema_uc: Arc::new(RegisterSchemaUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        get_schema_uc: Arc::new(GetSchemaUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        list_versions_uc: Arc::new(ListVersionsUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        register_version_uc: Arc::new(RegisterVersionUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        get_schema_version_uc: Arc::new(GetSchemaVersionUseCase::new(version_repo.clone())),
        delete_version_uc: Arc::new(DeleteVersionUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        check_compatibility_uc: Arc::new(CheckCompatibilityUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        get_diff_uc: Arc::new(GetDiffUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        metrics,
        auth_state: None,
    };

    router(state)
}

// ---------------------------------------------------------------------------
// テスト: /healthz と /readyz が 200 を返すことを確認
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz への GET リクエスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/healthz は 200 を返すべき");

    // /readyz への GET リクエスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/readyz は 200 を返すべき");
}

// ---------------------------------------------------------------------------
// テスト: 認証有効時に token なしで保護エンドポイントにアクセスすると 401 を返す
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_unauthorized_without_token() {
    let schema_repo: Arc<dyn ApiSchemaRepository> = Arc::new(StubSchemaRepo);
    let version_repo: Arc<dyn ApiSchemaVersionRepository> = Arc::new(StubVersionRepo);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 認証ありの AppState を構築（不正な JWKS URL でダミー verifier を生成）
    let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
        "https://invalid.example.com/.well-known/jwks.json",
        "https://invalid.example.com",
        "test-audience",
        std::time::Duration::from_secs(60),
    ));
    let auth_state = ApiRegistryAuthState { verifier };

    let state = AppState {
        list_schemas_uc: Arc::new(ListSchemasUseCase::new(schema_repo.clone())),
        register_schema_uc: Arc::new(RegisterSchemaUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        get_schema_uc: Arc::new(GetSchemaUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        list_versions_uc: Arc::new(ListVersionsUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        register_version_uc: Arc::new(RegisterVersionUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        get_schema_version_uc: Arc::new(GetSchemaVersionUseCase::new(version_repo.clone())),
        delete_version_uc: Arc::new(DeleteVersionUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        check_compatibility_uc: Arc::new(CheckCompatibilityUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        get_diff_uc: Arc::new(GetDiffUseCase::new(
            schema_repo.clone(),
            version_repo.clone(),
        )),
        metrics,
        auth_state: Some(auth_state),
    };

    let app = router(state);

    // 保護されたエンドポイントに Authorization ヘッダーなしでアクセス
    let req = Request::builder()
        .uri("/api/v1/schemas")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "token なしで保護エンドポイントは 401 を返すべき"
    );
}
