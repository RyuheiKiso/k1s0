#![allow(clippy::unwrap_used)]
// router 初期化と基本エンドポイントの smoke test
// featureflag サーバーの REST API ルーターが正しく構築され、
// ヘルスチェックおよび認証ミドルウェアが期待どおり動作することを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_featureflag_server::adapter::handler::{router, AppState};
// 認証状態の型をインポート（共通AuthStateを使用）
use k1s0_featureflag_server::adapter::middleware::auth::AuthState;
use k1s0_featureflag_server::domain::entity::feature_flag::FeatureFlag;
use k1s0_featureflag_server::domain::entity::flag_audit_log::FlagAuditLog;
use k1s0_featureflag_server::domain::repository::{FeatureFlagRepository, FlagAuditLogRepository};
use k1s0_featureflag_server::infrastructure::kafka_producer::FlagEventPublisher;
use k1s0_featureflag_server::usecase::*;

// ---------------------------------------------------------------------------
// テスト用スタブ: FeatureFlagRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubFlagRepo;

#[async_trait]
impl FeatureFlagRepository for StubFlagRepo {
    async fn find_by_key(&self, _flag_key: &str) -> anyhow::Result<FeatureFlag> {
        Err(anyhow::anyhow!("not found"))
    }
    async fn find_all(&self) -> anyhow::Result<Vec<FeatureFlag>> {
        Ok(vec![])
    }
    async fn create(&self, _flag: &FeatureFlag) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _flag: &FeatureFlag) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn exists_by_key(&self, _flag_key: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: FlagAuditLogRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubAuditRepo;

#[async_trait]
impl FlagAuditLogRepository for StubAuditRepo {
    async fn create(&self, _log: &FlagAuditLog) -> anyhow::Result<()> {
        Ok(())
    }
    async fn list_by_flag_id(
        &self,
        _flag_id: &Uuid,
        _limit: i64,
        _offset: i64,
    ) -> anyhow::Result<Vec<FlagAuditLog>> {
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: FlagEventPublisher（何もしないダミー実装）
// ---------------------------------------------------------------------------
struct StubFlagPublisher;

#[async_trait]
impl FlagEventPublisher for StubFlagPublisher {
    async fn publish_flag_changed(
        &self,
        _flag_key: &str,
        _enabled: bool,
        _actor_user_id: Option<String>,
        _before: Option<serde_json::Value>,
        _after: serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// テスト用アプリケーション構築ヘルパー（認証なしモード）
// ---------------------------------------------------------------------------
fn make_test_app() -> axum::Router {
    let flag_repo: Arc<dyn FeatureFlagRepository> = Arc::new(StubFlagRepo);
    let audit_repo: Arc<dyn FlagAuditLogRepository> = Arc::new(StubAuditRepo);
    let event_publisher: Arc<dyn FlagEventPublisher> = Arc::new(StubFlagPublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 各ユースケースをスタブリポジトリで構築
    let state = AppState {
        flag_repo: flag_repo.clone(),
        event_publisher: event_publisher.clone(),
        list_flags_uc: Arc::new(ListFlagsUseCase::new(flag_repo.clone())),
        evaluate_flag_uc: Arc::new(EvaluateFlagUseCase::new(flag_repo.clone())),
        get_flag_uc: Arc::new(GetFlagUseCase::new(flag_repo.clone())),
        create_flag_uc: Arc::new(CreateFlagUseCase::new(
            flag_repo.clone(),
            event_publisher.clone(),
            audit_repo.clone(),
        )),
        update_flag_uc: Arc::new(UpdateFlagUseCase::new(
            flag_repo.clone(),
            event_publisher.clone(),
            audit_repo.clone(),
        )),
        delete_flag_uc: Arc::new(DeleteFlagUseCase::new(
            flag_repo.clone(),
            event_publisher.clone(),
            audit_repo.clone(),
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
    let flag_repo: Arc<dyn FeatureFlagRepository> = Arc::new(StubFlagRepo);
    let audit_repo: Arc<dyn FlagAuditLogRepository> = Arc::new(StubAuditRepo);
    let event_publisher: Arc<dyn FlagEventPublisher> = Arc::new(StubFlagPublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 認証ありの AppState を構築（不正な JWKS URL でダミー verifier を生成）
    let verifier = Arc::new(
        k1s0_auth::JwksVerifier::new(
            "https://invalid.example.com/.well-known/jwks.json",
            "https://invalid.example.com",
            "test-audience",
            std::time::Duration::from_secs(60),
        )
        .expect("Failed to create JWKS verifier"),
    );
    // 共通AuthStateを使用して認証状態を構築
    let auth_state = AuthState { verifier };

    let state = AppState {
        flag_repo: flag_repo.clone(),
        event_publisher: event_publisher.clone(),
        list_flags_uc: Arc::new(ListFlagsUseCase::new(flag_repo.clone())),
        evaluate_flag_uc: Arc::new(EvaluateFlagUseCase::new(flag_repo.clone())),
        get_flag_uc: Arc::new(GetFlagUseCase::new(flag_repo.clone())),
        create_flag_uc: Arc::new(CreateFlagUseCase::new(
            flag_repo.clone(),
            event_publisher.clone(),
            audit_repo.clone(),
        )),
        update_flag_uc: Arc::new(UpdateFlagUseCase::new(
            flag_repo.clone(),
            event_publisher.clone(),
            audit_repo.clone(),
        )),
        delete_flag_uc: Arc::new(DeleteFlagUseCase::new(
            flag_repo.clone(),
            event_publisher.clone(),
            audit_repo.clone(),
        )),
        metrics,
        auth_state: Some(auth_state),
    };

    let app = router(state);

    // 保護されたエンドポイントに Authorization ヘッダーなしでアクセス
    let req = Request::builder()
        .uri("/api/v1/flags")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "token なしで保護エンドポイントは 401 を返すべき"
    );
}
