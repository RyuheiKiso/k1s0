// k1s0-ratelimit-server の router 初期化 smoke test。
// healthz/readyz の疎通確認と、認証なしでの保護エンドポイントアクセスを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_ratelimit_server::adapter::handler::{router, AppState};
use k1s0_ratelimit_server::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};
use k1s0_ratelimit_server::domain::repository::{
    rate_limit_repository::UsageSnapshot, RateLimitRepository, RateLimitStateStore,
};
use k1s0_ratelimit_server::usecase::{
    CheckRateLimitUseCase, CreateRuleUseCase, DeleteRuleUseCase, GetRuleUseCase, GetUsageUseCase,
    ListRulesUseCase, ResetRateLimitUseCase, UpdateRuleUseCase,
};

// --- テストダブル: RateLimitRepository のスタブ実装 ---

struct StubRateLimitRepository;

#[async_trait]
impl RateLimitRepository for StubRateLimitRepository {
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule> {
        Ok(rule.clone())
    }

    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<RateLimitRule> {
        Ok(RateLimitRule::new(
            "stub".to_string(),
            "*".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        ))
    }

    async fn find_by_name(&self, _name: &str) -> anyhow::Result<Option<RateLimitRule>> {
        Ok(None)
    }

    async fn find_by_scope(&self, _scope: &str) -> anyhow::Result<Vec<RateLimitRule>> {
        Ok(vec![])
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RateLimitRule>> {
        Ok(vec![])
    }

    async fn find_page(
        &self,
        _page: u32,
        _page_size: u32,
        _scope: Option<String>,
        _enabled_only: bool,
    ) -> anyhow::Result<(Vec<RateLimitRule>, u64)> {
        Ok((vec![], 0))
    }

    async fn update(&self, _rule: &RateLimitRule) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn reset_state(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: RateLimitStateStore のスタブ実装 ---

struct StubRateLimitStateStore;

#[async_trait]
impl RateLimitStateStore for StubRateLimitStateStore {
    async fn check_token_bucket(
        &self,
        _key: &str,
        limit: i64,
        _window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        Ok(RateLimitDecision::allowed(
            limit,
            limit - 1,
            chrono::Utc::now(),
        ))
    }

    async fn check_fixed_window(
        &self,
        _key: &str,
        limit: i64,
        _window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        Ok(RateLimitDecision::allowed(
            limit,
            limit - 1,
            chrono::Utc::now(),
        ))
    }

    async fn check_sliding_window(
        &self,
        _key: &str,
        limit: i64,
        _window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        Ok(RateLimitDecision::allowed(
            limit,
            limit - 1,
            chrono::Utc::now(),
        ))
    }

    async fn check_leaky_bucket(
        &self,
        _key: &str,
        limit: i64,
        _window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        Ok(RateLimitDecision::allowed(
            limit,
            limit - 1,
            chrono::Utc::now(),
        ))
    }

    async fn reset(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_usage(
        &self,
        _key: &str,
        _limit: i64,
        _window_secs: i64,
    ) -> anyhow::Result<Option<UsageSnapshot>> {
        Ok(None)
    }
}

// --- テストアプリケーション構築 ---

/// スタブリポジトリを使って AppState と Router を構築するヘルパー。
fn make_test_app() -> axum::Router {
    let repo: Arc<dyn RateLimitRepository> = Arc::new(StubRateLimitRepository);
    let state_store: Arc<dyn RateLimitStateStore> = Arc::new(StubRateLimitStateStore);

    let state = AppState::new(
        Arc::new(CheckRateLimitUseCase::new(repo.clone(), state_store.clone())),
        Arc::new(CreateRuleUseCase::new(repo.clone())),
        Arc::new(GetRuleUseCase::new(repo.clone())),
        Arc::new(ListRulesUseCase::new(repo.clone())),
        Arc::new(UpdateRuleUseCase::new(repo.clone())),
        Arc::new(DeleteRuleUseCase::new(repo.clone())),
        Arc::new(GetUsageUseCase::new(repo)),
        Arc::new(ResetRateLimitUseCase::new(state_store)),
        None,
    );
    router(state)
}

// --- テスト: /healthz と /readyz が 200 を返す ---

#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz への GET リクエストで 200 OK を確認
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz への GET リクエストで 200 OK を確認
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- テスト: 認証なしモード（auth_state=None）では保護エンドポイントに直接アクセス可能 ---
// ratelimit サーバーは auth_state=None の場合、認証なしで全 API にアクセスできる仕様。

#[tokio::test]
async fn test_api_accessible_without_auth() {
    let app = make_test_app();

    // /api/v1/ratelimit/rules への GET が 200 を返す（認証なしモード）
    let req = Request::builder()
        .uri("/api/v1/ratelimit/rules")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
