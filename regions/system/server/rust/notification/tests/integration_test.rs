// 通知サーバーの統合テスト。
// router の初期化と基本的なエンドポイントの動作を検証する。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_notification_server::adapter::handler::{router, AppState};
// 認証状態の型をインポート（共通AuthStateを使用）
use k1s0_notification_server::adapter::middleware::auth::AuthState;
use k1s0_notification_server::domain::entity::notification_channel::NotificationChannel;
use k1s0_notification_server::domain::entity::notification_log::NotificationLog;
use k1s0_notification_server::domain::entity::notification_template::NotificationTemplate;
use k1s0_notification_server::domain::repository::{
    NotificationChannelRepository, NotificationLogRepository, NotificationTemplateRepository,
};
use k1s0_notification_server::usecase::{
    CreateChannelUseCase, CreateTemplateUseCase, DeleteChannelUseCase, DeleteTemplateUseCase,
    GetChannelUseCase, GetTemplateUseCase, ListChannelsUseCase, ListTemplatesUseCase,
    RetryNotificationUseCase, SendNotificationUseCase, UpdateChannelUseCase, UpdateTemplateUseCase,
};

// --- テストダブル: チャネルリポジトリ ---

/// テスト用のチャネルリポジトリ。全メソッドが空の結果を返す。
struct StubChannelRepo;

#[async_trait]
impl NotificationChannelRepository for StubChannelRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<NotificationChannel>> {
        Ok(None)
    }
    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        Ok(vec![])
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _channel_type: Option<String>,
        _enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _channel: &NotificationChannel) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _channel: &NotificationChannel) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// --- テストダブル: ログリポジトリ ---

/// テスト用のログリポジトリ。全メソッドが空の結果を返す。
struct StubLogRepo;

#[async_trait]
impl NotificationLogRepository for StubLogRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<NotificationLog>> {
        Ok(None)
    }
    async fn find_by_channel_id(&self, _channel_id: &str) -> anyhow::Result<Vec<NotificationLog>> {
        Ok(vec![])
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _channel_id: Option<String>,
        _status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _log: &NotificationLog) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _log: &NotificationLog) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: テンプレートリポジトリ ---

/// テスト用のテンプレートリポジトリ。全メソッドが空の結果を返す。
struct StubTemplateRepo;

#[async_trait]
impl NotificationTemplateRepository for StubTemplateRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        Ok(None)
    }
    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>> {
        Ok(vec![])
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _template: &NotificationTemplate) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _template: &NotificationTemplate) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

/// テスト用の AppState を構築し、router を生成するヘルパー関数。
/// 認証有効モードで構築する（ダミー JWKS verifier を使用）。
fn make_test_app() -> axum::Router {
    let channel_repo: Arc<dyn NotificationChannelRepository> = Arc::new(StubChannelRepo);
    let log_repo: Arc<dyn NotificationLogRepository> = Arc::new(StubLogRepo);
    let template_repo: Arc<dyn NotificationTemplateRepository> = Arc::new(StubTemplateRepo);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("notification-test"));

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
        send_notification_uc: Arc::new(SendNotificationUseCase::new(
            channel_repo.clone(),
            log_repo.clone(),
        )),
        retry_notification_uc: Arc::new(RetryNotificationUseCase::new(
            log_repo.clone(),
            channel_repo.clone(),
        )),
        log_repo: log_repo.clone(),
        create_channel_uc: Arc::new(CreateChannelUseCase::new(channel_repo.clone())),
        list_channels_uc: Arc::new(ListChannelsUseCase::new(channel_repo.clone())),
        get_channel_uc: Arc::new(GetChannelUseCase::new(channel_repo.clone())),
        update_channel_uc: Arc::new(UpdateChannelUseCase::new(channel_repo.clone())),
        delete_channel_uc: Arc::new(DeleteChannelUseCase::new(channel_repo.clone())),
        create_template_uc: Arc::new(CreateTemplateUseCase::new(template_repo.clone())),
        list_templates_uc: Arc::new(ListTemplatesUseCase::new(template_repo.clone())),
        get_template_uc: Arc::new(GetTemplateUseCase::new(template_repo.clone())),
        update_template_uc: Arc::new(UpdateTemplateUseCase::new(template_repo.clone())),
        delete_template_uc: Arc::new(DeleteTemplateUseCase::new(template_repo.clone())),
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
    // 通知一覧エンドポイントに認証なしでアクセス
    let req = Request::builder()
        .uri("/api/v1/notifications")
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
