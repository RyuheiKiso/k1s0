// クォータサーバーの統合テスト。
// router の初期化と基本的なエンドポイントの動作を検証する。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_quota_server::adapter::handler::{router, AppState};
use k1s0_quota_server::adapter::middleware::auth::QuotaAuthState;
use k1s0_quota_server::domain::entity::quota::QuotaPolicy;
use k1s0_quota_server::domain::repository::{
    CheckAndIncrementResult, QuotaPolicyRepository, QuotaUsageRepository,
};
use k1s0_quota_server::infrastructure::kafka_producer::NoopQuotaEventPublisher;
use k1s0_quota_server::usecase::{
    CreateQuotaPolicyUseCase, DeleteQuotaPolicyUseCase, GetQuotaPolicyUseCase,
    GetQuotaUsageUseCase, IncrementQuotaUsageUseCase, ListQuotaPoliciesUseCase,
    ResetQuotaUsageUseCase, UpdateQuotaPolicyUseCase,
};

// --- テストダブル: クォータポリシーリポジトリ ---

/// テスト用のクォータポリシーリポジトリ。全メソッドが空の結果を返す。
struct StubQuotaPolicyRepo;

#[async_trait]
impl QuotaPolicyRepository for StubQuotaPolicyRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        Ok(None)
    }
    async fn find_all(
        &self,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<QuotaPolicy>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _policy: &QuotaPolicy) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _policy: &QuotaPolicy) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// --- テストダブル: クォータ使用量リポジトリ ---

/// テスト用のクォータ使用量リポジトリ。全メソッドが空の結果を返す。
struct StubQuotaUsageRepo;

#[async_trait]
impl QuotaUsageRepository for StubQuotaUsageRepo {
    async fn get_usage(&self, _quota_id: &str) -> anyhow::Result<Option<u64>> {
        Ok(None)
    }
    async fn increment(&self, _quota_id: &str, _amount: u64) -> anyhow::Result<u64> {
        Ok(0)
    }
    async fn reset(&self, _quota_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn check_and_increment(
        &self,
        _quota_id: &str,
        _amount: u64,
        _limit: u64,
    ) -> anyhow::Result<CheckAndIncrementResult> {
        Ok(CheckAndIncrementResult {
            used: 0,
            allowed: true,
        })
    }
}

/// テスト用の AppState を構築し、router を生成するヘルパー関数。
/// 認証有効モードで構築する（ダミー JWKS verifier を使用）。
fn make_test_app() -> axum::Router {
    let policy_repo: Arc<dyn QuotaPolicyRepository> = Arc::new(StubQuotaPolicyRepo);
    let usage_repo: Arc<dyn QuotaUsageRepository> = Arc::new(StubQuotaUsageRepo);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("quota-test"));

    // ダミーの JwksVerifier を作成（テスト中は実際にトークン検証しない）
    let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
        "https://dummy.example.com/jwks",
        "https://dummy.example.com",
        "dummy-audience",
        Duration::from_secs(600),
    ));
    let auth_state = QuotaAuthState { verifier };

    let state = AppState {
        create_policy_uc: Arc::new(CreateQuotaPolicyUseCase::new(policy_repo.clone())),
        get_policy_uc: Arc::new(GetQuotaPolicyUseCase::new(policy_repo.clone())),
        list_policies_uc: Arc::new(ListQuotaPoliciesUseCase::new(policy_repo.clone())),
        update_policy_uc: Arc::new(UpdateQuotaPolicyUseCase::new(policy_repo.clone())),
        delete_policy_uc: Arc::new(DeleteQuotaPolicyUseCase::new(policy_repo.clone())),
        get_usage_uc: Arc::new(GetQuotaUsageUseCase::new(
            policy_repo.clone(),
            usage_repo.clone(),
        )),
        increment_usage_uc: Arc::new(IncrementQuotaUsageUseCase::new(
            policy_repo.clone(),
            usage_repo.clone(),
            Arc::new(NoopQuotaEventPublisher),
        )),
        reset_usage_uc: Arc::new(ResetQuotaUsageUseCase::new(
            policy_repo.clone(),
            usage_repo.clone(),
        )),
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
    // クォータ一覧エンドポイントに認証なしでアクセス
    let req = Request::builder()
        .uri("/api/v1/quotas")
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
