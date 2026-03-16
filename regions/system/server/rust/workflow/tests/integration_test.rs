// ワークフローサーバーの統合テスト
// router 初期化の smoke test として、ヘルスチェックと認証なしアクセスを検証する

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// ワークフローサーバーのクレートから必要な型をインポート
use k1s0_workflow_server::adapter::handler::{router, AppState};
use k1s0_workflow_server::domain::entity::workflow_definition::WorkflowDefinition;
use k1s0_workflow_server::domain::entity::workflow_instance::WorkflowInstance;
use k1s0_workflow_server::domain::entity::workflow_task::WorkflowTask;
use k1s0_workflow_server::domain::repository::{
    WorkflowDefinitionRepository, WorkflowInstanceRepository, WorkflowTaskRepository,
};
use k1s0_workflow_server::infrastructure::kafka_producer::{
    NoopWorkflowEventPublisher, WorkflowEventPublisher,
};
use k1s0_workflow_server::infrastructure::notification_request_producer::NotificationRequestPublisher;
use k1s0_workflow_server::usecase;

// --- テスト用スタブ: WorkflowDefinitionRepository ---

/// テスト用のワークフロー定義リポジトリ。全メソッドが空の結果を返す。
struct StubDefinitionRepo;

#[async_trait]
impl WorkflowDefinitionRepository for StubDefinitionRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        Ok(None)
    }
    async fn find_by_name(&self, _name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        Ok(None)
    }
    async fn find_all(
        &self,
        _enabled_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _definition: &WorkflowDefinition) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _definition: &WorkflowDefinition) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// --- テスト用スタブ: WorkflowInstanceRepository ---

/// テスト用のワークフローインスタンスリポジトリ。全メソッドが空の結果を返す。
struct StubInstanceRepo;

#[async_trait]
impl WorkflowInstanceRepository for StubInstanceRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<WorkflowInstance>> {
        Ok(None)
    }
    async fn find_all(
        &self,
        _status: Option<String>,
        _workflow_id: Option<String>,
        _initiator_id: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)> {
        Ok((vec![], 0))
    }
    async fn create(&self, _instance: &WorkflowInstance) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _instance: &WorkflowInstance) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用スタブ: WorkflowTaskRepository ---

/// テスト用のワークフロータスクリポジトリ。全メソッドが空の結果を返す。
struct StubTaskRepo;

#[async_trait]
impl WorkflowTaskRepository for StubTaskRepo {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<WorkflowTask>> {
        Ok(None)
    }
    async fn find_all(
        &self,
        _assignee_id: Option<String>,
        _status: Option<String>,
        _instance_id: Option<String>,
        _overdue_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)> {
        Ok((vec![], 0))
    }
    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>> {
        Ok(vec![])
    }
    async fn create(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用スタブ: NotificationRequestPublisher ---

/// テスト用の通知パブリッシャー。全通知を破棄する。
struct StubNotificationPublisher;

#[async_trait]
impl NotificationRequestPublisher for StubNotificationPublisher {
    async fn publish_task_overdue(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テスト用アプリケーション構築 ---

/// テスト用の AppState を構築し、router を返すヘルパー関数。
/// 全リポジトリにスタブを使用し、認証は無効化する。
fn make_test_app() -> axum::Router {
    let def_repo: Arc<dyn WorkflowDefinitionRepository> = Arc::new(StubDefinitionRepo);
    let inst_repo: Arc<dyn WorkflowInstanceRepository> = Arc::new(StubInstanceRepo);
    let task_repo: Arc<dyn WorkflowTaskRepository> = Arc::new(StubTaskRepo);
    let notif_pub: Arc<dyn NotificationRequestPublisher> = Arc::new(StubNotificationPublisher);
    // テスト用のイベントパブリッシャー（Kafka 不要の Noop 実装）
    let event_pub: Arc<dyn WorkflowEventPublisher> = Arc::new(NoopWorkflowEventPublisher);

    // AppState の構築（認証なし）
    let state = AppState {
        create_workflow_uc: Arc::new(usecase::CreateWorkflowUseCase::new(def_repo.clone())),
        update_workflow_uc: Arc::new(usecase::UpdateWorkflowUseCase::new(def_repo.clone())),
        delete_workflow_uc: Arc::new(usecase::DeleteWorkflowUseCase::new(def_repo.clone())),
        get_workflow_uc: Arc::new(usecase::GetWorkflowUseCase::new(def_repo.clone())),
        list_workflows_uc: Arc::new(usecase::ListWorkflowsUseCase::new(def_repo.clone())),
        start_instance_uc: Arc::new(usecase::StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            event_pub.clone(),
        )),
        get_instance_uc: Arc::new(usecase::GetInstanceUseCase::new(inst_repo.clone())),
        list_instances_uc: Arc::new(usecase::ListInstancesUseCase::new(inst_repo.clone())),
        cancel_instance_uc: Arc::new(usecase::CancelInstanceUseCase::new(inst_repo.clone())),
        list_tasks_uc: Arc::new(usecase::ListTasksUseCase::new(task_repo.clone())),
        approve_task_uc: Arc::new(usecase::ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_pub.clone(),
        )),
        reject_task_uc: Arc::new(usecase::RejectTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_pub.clone(),
        )),
        reassign_task_uc: Arc::new(usecase::ReassignTaskUseCase::new(task_repo.clone())),
        check_overdue_tasks_uc: Arc::new(usecase::CheckOverdueTasksUseCase::new(
            task_repo.clone(),
            notif_pub,
        )),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-workflow-server-test",
        )),
        auth_state: None,
    };

    // metrics_enabled = false, metrics_path は未使用
    router(state, false, "/metrics")
}

// --- ヘルスチェックテスト ---

/// /healthz と /readyz エンドポイントが 200 OK を返すことを確認する
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz へのリクエスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz へのリクエスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- 認証なしアクセステスト ---

/// 認証が無効な状態で保護エンドポイントにアクセスすると正常にルーティングされることを確認する。
/// auth_state が None の場合、認証ミドルウェアはスキップされる。
#[tokio::test]
async fn test_api_routes_are_reachable() {
    let app = make_test_app();

    // 認証なしモードでは /api/v1/workflows にアクセスできる
    let req = Request::builder()
        .uri("/api/v1/workflows?page=1&page_size=10")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // ルーターが正常に応答すること（500 でないこと）を確認
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
