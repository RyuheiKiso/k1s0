// Notification サーバーの handler テスト
// axum-test を使って REST API エンドポイントの動作を確認する
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use async_trait::async_trait;
use axum_test::TestServer;
use tokio::sync::RwLock;

use k1s0_notification_server::adapter::handler::{router, AppState};
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

// ---------------------------------------------------------------------------
// Stub: In-memory NotificationChannelRepository
// ---------------------------------------------------------------------------

struct StubChannelRepo {
    channels: RwLock<Vec<NotificationChannel>>,
}

impl StubChannelRepo {
    fn new() -> Self {
        Self {
            channels: RwLock::new(Vec::new()),
        }
    }
}

/// MEDIUM-RUST-001 監査対応: StubChannelRepo もトレイト変更に追従する。
#[async_trait]
impl NotificationChannelRepository for StubChannelRepo {
    async fn find_by_id(
        &self,
        id: &str,
        _tenant_id: &str,
    ) -> anyhow::Result<Option<NotificationChannel>> {
        Ok(self
            .channels
            .read()
            .await
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }
    async fn find_all(&self, _tenant_id: &str) -> anyhow::Result<Vec<NotificationChannel>> {
        Ok(self.channels.read().await.clone())
    }
    async fn find_all_paginated(
        &self,
        _tenant_id: &str,
        _page: u32,
        _page_size: u32,
        _channel_type: Option<String>,
        _enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)> {
        let ch = self.channels.read().await.clone();
        let n = ch.len() as u64;
        Ok((ch, n))
    }
    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        self.channels.write().await.push(channel.clone());
        Ok(())
    }
    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        let mut channels = self.channels.write().await;
        if let Some(e) = channels.iter_mut().find(|c| c.id == channel.id) {
            *e = channel.clone();
        }
        Ok(())
    }
    async fn delete(&self, id: &str, _tenant_id: &str) -> anyhow::Result<bool> {
        let mut channels = self.channels.write().await;
        let before = channels.len();
        channels.retain(|c| c.id != id);
        Ok(channels.len() < before)
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory NotificationTemplateRepository
// ---------------------------------------------------------------------------

struct StubTemplateRepo {
    templates: RwLock<Vec<NotificationTemplate>>,
}

impl StubTemplateRepo {
    fn new() -> Self {
        Self {
            templates: RwLock::new(Vec::new()),
        }
    }
}

/// テナント分離対応: トレイト変更に追従する。tenant_id 引数はスタブのため無視する。
#[async_trait]
impl NotificationTemplateRepository for StubTemplateRepo {
    async fn find_by_id(&self, id: &str, _tenant_id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        Ok(self
            .templates
            .read()
            .await
            .iter()
            .find(|t| t.id == id)
            .cloned())
    }
    async fn find_all(&self, _tenant_id: &str) -> anyhow::Result<Vec<NotificationTemplate>> {
        Ok(self.templates.read().await.clone())
    }
    async fn find_all_paginated(
        &self,
        _tenant_id: &str,
        _page: u32,
        _page_size: u32,
        _channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)> {
        let t = self.templates.read().await.clone();
        let n = t.len() as u64;
        Ok((t, n))
    }
    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        self.templates.write().await.push(template.clone());
        Ok(())
    }
    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut templates = self.templates.write().await;
        if let Some(e) = templates.iter_mut().find(|t| t.id == template.id) {
            *e = template.clone();
        }
        Ok(())
    }
    async fn delete(&self, id: &str, _tenant_id: &str) -> anyhow::Result<bool> {
        let mut templates = self.templates.write().await;
        let before = templates.len();
        templates.retain(|t| t.id != id);
        Ok(templates.len() < before)
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory NotificationLogRepository
// ---------------------------------------------------------------------------

struct StubLogRepo {
    logs: RwLock<Vec<NotificationLog>>,
}

impl StubLogRepo {
    fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
        }
    }
}

/// テナント分離対応: トレイト変更に追従する。tenant_id 引数はスタブのため無視する。
#[async_trait]
impl NotificationLogRepository for StubLogRepo {
    async fn find_by_id(&self, id: &str, _tenant_id: &str) -> anyhow::Result<Option<NotificationLog>> {
        Ok(self.logs.read().await.iter().find(|l| l.id == id).cloned())
    }
    async fn find_by_channel_id(&self, channel_id: &str, _tenant_id: &str) -> anyhow::Result<Vec<NotificationLog>> {
        Ok(self
            .logs
            .read()
            .await
            .iter()
            .filter(|l| l.channel_id == channel_id)
            .cloned()
            .collect())
    }
    async fn find_all_paginated(
        &self,
        _tenant_id: &str,
        _page: u32,
        _page_size: u32,
        _channel_id: Option<String>,
        _status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)> {
        let l = self.logs.read().await.clone();
        let n = l.len() as u64;
        Ok((l, n))
    }
    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        self.logs.write().await.push(log.clone());
        Ok(())
    }
    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut logs = self.logs.write().await;
        if let Some(e) = logs.iter_mut().find(|l| l.id == log.id) {
            *e = log.clone();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper: AppState を構築する
// ---------------------------------------------------------------------------

fn build_state() -> AppState {
    let channel_repo = Arc::new(StubChannelRepo::new());
    let template_repo = Arc::new(StubTemplateRepo::new());
    let log_repo = Arc::new(StubLogRepo::new());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("notification-test"));
    AppState {
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
        auth_state: None,
        db_pool: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// GET /healthz は 200 {"status":"ok"} を返す
#[tokio::test]
async fn healthz_returns_ok() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/healthz").await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["status"], "ok");
}

/// GET /readyz は 200 {"status":"healthy"} を返す（ADR-0068 対応: "ready" → "healthy" に統一済み）
#[tokio::test]
async fn readyz_returns_ready() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/readyz").await;
    resp.assert_status_ok();
    // ADR-0068 対応: status は "ready" ではなく "healthy" に統一されている
    assert_eq!(resp.json::<serde_json::Value>()["status"], "healthy");
}

/// GET /api/v1/channels は空リストを返す（認証なしモード）
#[tokio::test]
async fn list_channels_returns_empty() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/channels").await;
    resp.assert_status_ok();
    let body = resp.json::<serde_json::Value>();
    assert!(body["channels"].as_array().unwrap().is_empty());
}

/// GET /api/v1/channels/{id} は存在しない ID に 404 を返す
#[tokio::test]
async fn get_channel_not_found() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/channels/ch_nonexistent").await;
    resp.assert_status_not_found();
}

/// POST /api/v1/channels でチャンネルを作成できる
#[tokio::test]
async fn create_channel_success() {
    let server = TestServer::new(router(build_state())).unwrap();
    let body = serde_json::json!({
        "name": "email-alert",
        "channel_type": "email",
        "config": {"smtp_host": "localhost"},
        "enabled": true
    });
    let resp = server.post("/api/v1/channels").json(&body).await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let created = resp.json::<serde_json::Value>();
    assert_eq!(created["name"], "email-alert");
}

/// GET /api/v1/templates は空リストを返す
#[tokio::test]
async fn list_templates_returns_empty() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/templates").await;
    resp.assert_status_ok();
    let body = resp.json::<serde_json::Value>();
    assert!(body["templates"].as_array().unwrap().is_empty());
}

/// GET /api/v1/notifications は通知ログ一覧を返す
#[tokio::test]
async fn list_notifications_returns_empty() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/notifications").await;
    resp.assert_status_ok();
    let body = resp.json::<serde_json::Value>();
    assert!(body["notifications"].as_array().unwrap().is_empty());
}
