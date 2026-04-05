#![allow(clippy::unwrap_used)]
// router 初期化と基本エンドポイントの smoke test
// event-store サーバーの REST API ルーターが正しく構築され、
// ヘルスチェックおよび認証ミドルウェアが期待どおり動作することを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_event_store_server::adapter::handler::{router, AppState};
// 認証状態の型をインポート（共通AuthStateを使用）
use k1s0_event_store_server::adapter::middleware::auth::AuthState;
use k1s0_event_store_server::domain::entity::event::{EventStream, Snapshot, StoredEvent};
use k1s0_event_store_server::domain::repository::{
    EventRepository, EventStreamRepository, SnapshotRepository,
};
use k1s0_event_store_server::infrastructure::kafka::EventPublisher;
use k1s0_event_store_server::usecase::*;

// ---------------------------------------------------------------------------
// テスト用スタブ: EventStreamRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubStreamRepo;

#[async_trait]
impl EventStreamRepository for StubStreamRepo {
    async fn find_by_id(&self, _tenant_id: &str, _id: &str) -> anyhow::Result<Option<EventStream>> {
        Ok(None)
    }
    async fn list_all(
        &self,
        _tenant_id: &str,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<EventStream>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _stream: &EventStream) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update_version(
        &self,
        _tenant_id: &str,
        _id: &str,
        _new_version: i64,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _tenant_id: &str, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: EventRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubEventRepo;

#[async_trait]
impl EventRepository for StubEventRepo {
    async fn append(
        &self,
        _tenant_id: &str,
        _stream_id: &str,
        _events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        Ok(vec![])
    }
    async fn find_by_stream(
        &self,
        _tenant_id: &str,
        _stream_id: &str,
        _from_version: i64,
        _to_version: Option<i64>,
        _event_type: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        Ok((vec![], 0))
    }
    async fn find_all(
        &self,
        _tenant_id: &str,
        _event_type: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        Ok((vec![], 0))
    }
    async fn find_by_sequence(
        &self,
        _tenant_id: &str,
        _stream_id: &str,
        _sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>> {
        Ok(None)
    }
    async fn delete_by_stream(&self, _tenant_id: &str, _stream_id: &str) -> anyhow::Result<u64> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: SnapshotRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubSnapshotRepo;

#[async_trait]
impl SnapshotRepository for StubSnapshotRepo {
    async fn create(&self, _snapshot: &Snapshot) -> anyhow::Result<()> {
        Ok(())
    }
    async fn find_latest(
        &self,
        _tenant_id: &str,
        _stream_id: &str,
    ) -> anyhow::Result<Option<Snapshot>> {
        Ok(None)
    }
    async fn delete_by_stream(
        &self,
        _tenant_id: &str,
        _stream_id: &str,
    ) -> anyhow::Result<u64> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: EventPublisher（何もしないダミー実装）
// ---------------------------------------------------------------------------
struct StubEventPublisher;

#[async_trait]
impl EventPublisher for StubEventPublisher {
    async fn publish_events(
        &self,
        _stream_id: &str,
        _events: &[StoredEvent],
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// テスト用アプリケーション構築ヘルパー（認証なしモード）
// ---------------------------------------------------------------------------
fn make_test_app() -> axum::Router {
    let stream_repo: Arc<dyn EventStreamRepository> = Arc::new(StubStreamRepo);
    let event_repo: Arc<dyn EventRepository> = Arc::new(StubEventRepo);
    let snapshot_repo: Arc<dyn SnapshotRepository> = Arc::new(StubSnapshotRepo);
    let event_publisher: Arc<dyn EventPublisher> = Arc::new(StubEventPublisher);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 各ユースケースをスタブリポジトリで構築
    let state = AppState {
        append_events_uc: Arc::new(AppendEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        )),
        read_events_uc: Arc::new(ReadEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        )),
        read_event_by_sequence_uc: Arc::new(ReadEventBySequenceUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        )),
        list_events_uc: Arc::new(ListEventsUseCase::new(event_repo.clone())),
        list_streams_uc: Arc::new(ListStreamsUseCase::new(stream_repo.clone())),
        create_snapshot_uc: Arc::new(CreateSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        )),
        get_latest_snapshot_uc: Arc::new(GetLatestSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        )),
        delete_stream_uc: Arc::new(DeleteStreamUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
            snapshot_repo.clone(),
        )),
        stream_repo: stream_repo.clone(),
        event_publisher,
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
    let stream_repo: Arc<dyn EventStreamRepository> = Arc::new(StubStreamRepo);
    let event_repo: Arc<dyn EventRepository> = Arc::new(StubEventRepo);
    let snapshot_repo: Arc<dyn SnapshotRepository> = Arc::new(StubSnapshotRepo);
    let event_publisher: Arc<dyn EventPublisher> = Arc::new(StubEventPublisher);
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
        append_events_uc: Arc::new(AppendEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        )),
        read_events_uc: Arc::new(ReadEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        )),
        read_event_by_sequence_uc: Arc::new(ReadEventBySequenceUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        )),
        list_events_uc: Arc::new(ListEventsUseCase::new(event_repo.clone())),
        list_streams_uc: Arc::new(ListStreamsUseCase::new(stream_repo.clone())),
        create_snapshot_uc: Arc::new(CreateSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        )),
        get_latest_snapshot_uc: Arc::new(GetLatestSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        )),
        delete_stream_uc: Arc::new(DeleteStreamUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
            snapshot_repo.clone(),
        )),
        stream_repo: stream_repo.clone(),
        event_publisher,
        metrics,
        auth_state: Some(auth_state),
    };

    let app = router(state);

    // 保護されたエンドポイントに Authorization ヘッダーなしでアクセス
    let req = Request::builder()
        .uri("/api/v1/events")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "token なしで保護エンドポイントは 401 を返すべき"
    );
}
