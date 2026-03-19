// ポリシーサーバーの統合テスト。
// router の初期化と基本的なエンドポイントの動作を検証する。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_policy_server::adapter::handler::{router, AppState};
// 認証状態の型をインポート（共通AuthStateを使用）
use k1s0_policy_server::adapter::middleware::auth::AuthState;
use k1s0_policy_server::domain::entity::policy::Policy;
use k1s0_policy_server::domain::entity::policy_bundle::PolicyBundle;
use k1s0_policy_server::domain::repository::{PolicyBundleRepository, PolicyRepository};
use k1s0_policy_server::usecase::{
    CreateBundleUseCase, CreatePolicyUseCase, DeletePolicyUseCase, EvaluatePolicyUseCase,
    GetBundleUseCase, GetPolicyUseCase, ListBundlesUseCase, ListPoliciesUseCase,
    UpdatePolicyUseCase,
};

// --- テストダブル: ポリシーリポジトリ ---

/// テスト用のポリシーリポジトリ。全メソッドが空の結果を返す。
struct StubPolicyRepo;

#[async_trait]
impl PolicyRepository for StubPolicyRepo {
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<Policy>> {
        Ok(None)
    }
    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        Ok(vec![])
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _bundle_id: Option<Uuid>,
        _enabled_only: bool,
    ) -> anyhow::Result<(Vec<Policy>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _policy: &Policy) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _policy: &Policy) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn exists_by_name(&self, _name: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// --- テストダブル: バンドルリポジトリ ---

/// テスト用のバンドルリポジトリ。全メソッドが空の結果を返す。
struct StubBundleRepo;

#[async_trait]
impl PolicyBundleRepository for StubBundleRepo {
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<PolicyBundle>> {
        Ok(None)
    }
    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>> {
        Ok(vec![])
    }
    async fn create(&self, _bundle: &PolicyBundle) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }
}

/// テスト用の AppState を構築し、router を生成するヘルパー関数。
/// 認証有効モードで構築する（ダミー JWKS verifier を使用）。
fn make_test_app() -> axum::Router {
    let policy_repo: Arc<dyn PolicyRepository> = Arc::new(StubPolicyRepo);
    let bundle_repo: Arc<dyn PolicyBundleRepository> = Arc::new(StubBundleRepo);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("policy-test"));

    // ダミーの JwksVerifier を作成（テスト中は実際にトークン検証しない）
    let verifier = Arc::new(
        k1s0_auth::JwksVerifier::new(
            "https://dummy.example.com/jwks",
            "https://dummy.example.com",
            "dummy-audience",
            Duration::from_secs(600),
        )
        .expect("Failed to create JWKS verifier"),
    );
    // 共通AuthStateを使用して認証状態を構築
    let auth_state = AuthState { verifier };

    let state = AppState {
        create_policy_uc: Arc::new(CreatePolicyUseCase::new(policy_repo.clone())),
        get_policy_uc: Arc::new(GetPolicyUseCase::new(policy_repo.clone())),
        list_policies_uc: Arc::new(ListPoliciesUseCase::new(policy_repo.clone())),
        update_policy_uc: Arc::new(UpdatePolicyUseCase::new(policy_repo.clone())),
        delete_policy_uc: Arc::new(DeletePolicyUseCase::new(policy_repo.clone())),
        evaluate_policy_uc: Arc::new(EvaluatePolicyUseCase::new(policy_repo.clone(), None)),
        create_bundle_uc: Arc::new(CreateBundleUseCase::new(bundle_repo.clone())),
        get_bundle_uc: Arc::new(GetBundleUseCase::new(bundle_repo.clone())),
        list_bundles_uc: Arc::new(ListBundlesUseCase::new(bundle_repo.clone())),
        metrics,
        auth_state: Some(auth_state),
    };
    router(state)
}

// --- テストケース ---

/// /healthz と /readyz への GET リクエストが 200 を返すことを検証する。
#[tokio::test]
async fn test_healthz_and_readyz() {
    // /healthz の検証
    let app = make_test_app();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz の検証
    let app = make_test_app();
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 保護されたエンドポイントに token なしでアクセスすると 401 が返ることを検証する。
#[tokio::test]
async fn test_unauthorized_without_token() {
    let app = make_test_app();
    // ポリシー一覧エンドポイントに認証なしでアクセス
    let req = Request::builder()
        .uri("/api/v1/policies")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // 認証トークンがないため 401 が返る
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// /metrics エンドポイントが 200 を返すことを検証する。
#[tokio::test]
async fn test_metrics_returns_ok() {
    let app = make_test_app();
    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
