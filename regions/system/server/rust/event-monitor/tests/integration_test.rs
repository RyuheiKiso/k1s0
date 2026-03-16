// router 初期化と基本エンドポイントの smoke test
// event-monitor サーバーの REST API ルーターが正しく構築され、
// ヘルスチェックおよび認証ミドルウェアが期待どおり動作することを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, Utc};
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_event_monitor_server::adapter::handler::{router, AppState};
use k1s0_event_monitor_server::adapter::middleware::auth::EventMonitorAuthState;
use k1s0_event_monitor_server::domain::entity::event_record::EventRecord;
use k1s0_event_monitor_server::domain::entity::flow_definition::FlowDefinition;
use k1s0_event_monitor_server::domain::entity::flow_instance::FlowInstance;
use k1s0_event_monitor_server::domain::repository::{
    EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository,
};
use k1s0_event_monitor_server::infrastructure::dlq_client::{DlqManagerClient, NoopDlqClient};
use k1s0_event_monitor_server::usecase::*;

// ---------------------------------------------------------------------------
// テスト用スタブ: EventRecordRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubEventRecordRepo;

#[async_trait]
impl EventRecordRepository for StubEventRecordRepo {
    async fn create(&self, _record: &EventRecord) -> anyhow::Result<()> {
        Ok(())
    }
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<EventRecord>> {
        Ok(None)
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _domain: Option<String>,
        _event_type: Option<String>,
        _source: Option<String>,
        _from: Option<DateTime<Utc>>,
        _to: Option<DateTime<Utc>>,
        _status: Option<String>,
    ) -> anyhow::Result<(Vec<EventRecord>, u64)> {
        Ok((vec![], 0))
    }
    async fn find_by_correlation_id(
        &self,
        _correlation_id: String,
    ) -> anyhow::Result<Vec<EventRecord>> {
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: FlowDefinitionRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubFlowDefRepo;

#[async_trait]
impl FlowDefinitionRepository for StubFlowDefRepo {
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<FlowDefinition>> {
        Ok(None)
    }
    async fn find_all(&self) -> anyhow::Result<Vec<FlowDefinition>> {
        Ok(vec![])
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<FlowDefinition>, u64)> {
        Ok((vec![], 0))
    }
    async fn find_by_domain_and_event_type(
        &self,
        _domain: String,
        _event_type: String,
    ) -> anyhow::Result<Vec<FlowDefinition>> {
        Ok(vec![])
    }
    async fn create(&self, _flow: &FlowDefinition) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _flow: &FlowDefinition) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn exists_by_name(&self, _name: String) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// テスト用スタブ: FlowInstanceRepository（全メソッドが空の結果を返す）
// ---------------------------------------------------------------------------
struct StubFlowInstRepo;

#[async_trait]
impl FlowInstanceRepository for StubFlowInstRepo {
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<FlowInstance>> {
        Ok(None)
    }
    async fn find_by_correlation_id(
        &self,
        _correlation_id: String,
    ) -> anyhow::Result<Option<FlowInstance>> {
        Ok(None)
    }
    async fn find_by_flow_id_paginated(
        &self,
        _flow_id: &Uuid,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<FlowInstance>, u64)> {
        Ok((vec![], 0))
    }
    async fn find_in_progress(&self) -> anyhow::Result<Vec<FlowInstance>> {
        Ok(vec![])
    }
    async fn create(&self, _instance: &FlowInstance) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _instance: &FlowInstance) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// テスト用アプリケーション構築ヘルパー（認証なしモード）
// ---------------------------------------------------------------------------
fn make_test_app() -> axum::Router {
    let event_repo: Arc<dyn EventRecordRepository> = Arc::new(StubEventRecordRepo);
    let flow_def_repo: Arc<dyn FlowDefinitionRepository> = Arc::new(StubFlowDefRepo);
    let flow_inst_repo: Arc<dyn FlowInstanceRepository> = Arc::new(StubFlowInstRepo);
    let dlq_client: Arc<dyn DlqManagerClient> = Arc::new(NoopDlqClient);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 各ユースケースをスタブリポジトリで構築
    let state = AppState {
        list_events_uc: Arc::new(ListEventsUseCase::new(event_repo.clone())),
        trace_by_correlation_uc: Arc::new(TraceByCorrelationUseCase::new(
            event_repo.clone(),
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        create_flow_uc: Arc::new(CreateFlowUseCase::new(flow_def_repo.clone())),
        get_flow_uc: Arc::new(GetFlowUseCase::new(flow_def_repo.clone())),
        update_flow_uc: Arc::new(UpdateFlowUseCase::new(flow_def_repo.clone())),
        delete_flow_uc: Arc::new(DeleteFlowUseCase::new(flow_def_repo.clone())),
        list_flows_uc: Arc::new(ListFlowsUseCase::new(flow_def_repo.clone())),
        get_flow_instances_uc: Arc::new(GetFlowInstancesUseCase::new(flow_inst_repo.clone())),
        get_flow_instance_uc: Arc::new(GetFlowInstanceUseCase::new(flow_inst_repo.clone())),
        get_flow_kpi_uc: Arc::new(GetFlowKpiUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        get_kpi_summary_uc: Arc::new(GetKpiSummaryUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        get_slo_status_uc: Arc::new(GetSloStatusUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        get_slo_burn_rate_uc: Arc::new(GetSloBurnRateUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        preview_replay_uc: Arc::new(PreviewReplayUseCase::new(
            event_repo.clone(),
            flow_def_repo.clone(),
            dlq_client.clone(),
        )),
        execute_replay_uc: Arc::new(ExecuteReplayUseCase::new(dlq_client.clone())),
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
    let event_repo: Arc<dyn EventRecordRepository> = Arc::new(StubEventRecordRepo);
    let flow_def_repo: Arc<dyn FlowDefinitionRepository> = Arc::new(StubFlowDefRepo);
    let flow_inst_repo: Arc<dyn FlowInstanceRepository> = Arc::new(StubFlowInstRepo);
    let dlq_client: Arc<dyn DlqManagerClient> = Arc::new(NoopDlqClient);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    // 認証ありの AppState を構築（不正な JWKS URL でダミー verifier を生成）
    let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
        "https://invalid.example.com/.well-known/jwks.json",
        "https://invalid.example.com",
        "test-audience",
        std::time::Duration::from_secs(60),
    ));
    let auth_state = EventMonitorAuthState { verifier };

    let state = AppState {
        list_events_uc: Arc::new(ListEventsUseCase::new(event_repo.clone())),
        trace_by_correlation_uc: Arc::new(TraceByCorrelationUseCase::new(
            event_repo.clone(),
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        create_flow_uc: Arc::new(CreateFlowUseCase::new(flow_def_repo.clone())),
        get_flow_uc: Arc::new(GetFlowUseCase::new(flow_def_repo.clone())),
        update_flow_uc: Arc::new(UpdateFlowUseCase::new(flow_def_repo.clone())),
        delete_flow_uc: Arc::new(DeleteFlowUseCase::new(flow_def_repo.clone())),
        list_flows_uc: Arc::new(ListFlowsUseCase::new(flow_def_repo.clone())),
        get_flow_instances_uc: Arc::new(GetFlowInstancesUseCase::new(flow_inst_repo.clone())),
        get_flow_instance_uc: Arc::new(GetFlowInstanceUseCase::new(flow_inst_repo.clone())),
        get_flow_kpi_uc: Arc::new(GetFlowKpiUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        get_kpi_summary_uc: Arc::new(GetKpiSummaryUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        get_slo_status_uc: Arc::new(GetSloStatusUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        get_slo_burn_rate_uc: Arc::new(GetSloBurnRateUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )),
        preview_replay_uc: Arc::new(PreviewReplayUseCase::new(
            event_repo.clone(),
            flow_def_repo.clone(),
            dlq_client.clone(),
        )),
        execute_replay_uc: Arc::new(ExecuteReplayUseCase::new(dlq_client.clone())),
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
