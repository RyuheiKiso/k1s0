#![allow(clippy::unwrap_used)]
// k1s0-session-server の router 初期化 smoke test。
// healthz/readyz の疎通確認と、認証なしでの保護エンドポイントアクセスを検証する。
// session サーバーは router() 関数を公開していないため、startup.rs と同様に手動構築する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_session_server::adapter::handler::session_handler::{self, AppState};
use k1s0_session_server::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
use k1s0_session_server::domain::entity::session::Session;
use k1s0_session_server::domain::repository::SessionRepository;
use k1s0_session_server::error::SessionError;
use k1s0_session_server::infrastructure::kafka_producer::NoopSessionEventPublisher;
use k1s0_session_server::usecase::{
    CreateSessionUseCase, GetSessionUseCase, ListUserSessionsUseCase, RefreshSessionUseCase,
    RevokeAllSessionsUseCase, RevokeSessionUseCase,
};

// --- テストダブル: SessionRepository のスタブ実装 ---

struct StubSessionRepository;

#[async_trait]
impl SessionRepository for StubSessionRepository {
    async fn save(&self, _session: &Session) -> Result<(), SessionError> {
        Ok(())
    }

    async fn find_by_id(&self, _id: &str) -> Result<Option<Session>, SessionError> {
        Ok(None)
    }

    async fn find_by_token(&self, _token: &str) -> Result<Option<Session>, SessionError> {
        Ok(None)
    }

    async fn find_by_user_id(&self, _user_id: &str) -> Result<Vec<Session>, SessionError> {
        Ok(vec![])
    }

    async fn delete(&self, _id: &str) -> Result<(), SessionError> {
        Ok(())
    }
}

// --- テストアプリケーション構築 ---

/// スタブリポジトリを使い、startup.rs と同様のルーティングで Router を構築するヘルパー。
fn make_test_app() -> axum::Router {
    let repo: Arc<dyn SessionRepository> = Arc::new(StubSessionRepository);
    let metadata_repo = Arc::new(NoopSessionMetadataRepository);
    let event_publisher = Arc::new(NoopSessionEventPublisher);

    let state = AppState {
        create_uc: Arc::new(CreateSessionUseCase::new(
            repo.clone(),
            metadata_repo.clone(),
            event_publisher.clone(),
            3600,
            86400,
        )),
        get_uc: Arc::new(GetSessionUseCase::new(repo.clone())),
        refresh_uc: Arc::new(RefreshSessionUseCase::new(repo.clone(), 86400)),
        revoke_uc: Arc::new(RevokeSessionUseCase::new(
            repo.clone(),
            metadata_repo.clone(),
            event_publisher.clone(),
        )),
        list_uc: Arc::new(ListUserSessionsUseCase::new(repo.clone())),
        revoke_all_uc: Arc::new(RevokeAllSessionsUseCase::new(repo, metadata_repo.clone())),
        metadata_repo,
        event_publisher,
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-session-server-test",
        )),
        auth_state: None,
        // テスト用: Redis未設定（インメモリリポジトリを使用）
        redis_configured: false,
    };

    // startup.rs と同じ構成: public_routes + api_routes (auth_state=None)
    let public_routes = axum::Router::new()
        .route(
            "/healthz",
            axum::routing::get(k1s0_session_server::adapter::handler::health::healthz),
        )
        .route(
            "/readyz",
            axum::routing::get(k1s0_session_server::adapter::handler::health::readyz),
        );

    // 認証なしモードの API ルート
    let api_routes = axum::Router::new()
        .route(
            "/api/v1/sessions",
            axum::routing::post(session_handler::create_session),
        )
        .route(
            "/api/v1/sessions/{session_id}",
            axum::routing::get(session_handler::get_session)
                .delete(session_handler::revoke_session),
        )
        .route(
            "/api/v1/sessions/{session_id}/refresh",
            axum::routing::post(session_handler::refresh_session),
        )
        .route(
            "/api/v1/users/{user_id}/sessions",
            axum::routing::get(session_handler::list_user_sessions)
                .delete(session_handler::revoke_all_sessions),
        );

    public_routes.merge(api_routes).with_state(state)
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

    // /readyz への GET リクエストで 200 OK を確認（StubRepo なので全チェック OK）
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- テスト: 認証なしモード（auth_state=None）では保護エンドポイントに直接アクセス可能 ---

#[tokio::test]
async fn test_api_accessible_without_auth() {
    let app = make_test_app();

    // /api/v1/users/test-user/sessions への GET が 200 を返す（認証なしモード）
    let req = Request::builder()
        .uri("/api/v1/users/test-user/sessions")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
