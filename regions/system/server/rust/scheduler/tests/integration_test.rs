#![allow(clippy::unwrap_used)]
// k1s0-scheduler-server の router 初期化 smoke test。
// healthz/readyz の疎通確認と、認証なしでの保護エンドポイントアクセスを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_scheduler_server::adapter::handler::{router, AppState};
use k1s0_scheduler_server::domain::entity::scheduler_execution::SchedulerExecution;
use k1s0_scheduler_server::domain::entity::scheduler_job::SchedulerJob;
use k1s0_scheduler_server::domain::repository::{
    SchedulerExecutionRepository, SchedulerJobRepository,
};
use k1s0_scheduler_server::infrastructure::kafka_producer::NoopSchedulerEventPublisher;
use k1s0_scheduler_server::usecase::{
    CreateJobUseCase, DeleteJobUseCase, GetJobUseCase, ListExecutionsUseCase, ListJobsUseCase,
    PauseJobUseCase, ResumeJobUseCase, TriggerJobUseCase, UpdateJobUseCase,
};

// --- テストダブル: SchedulerJobRepository のスタブ実装 ---

struct StubJobRepository;

#[async_trait]
impl SchedulerJobRepository for StubJobRepository {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<SchedulerJob>> {
        Ok(None)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        Ok(vec![])
    }

    async fn create(&self, _job: &SchedulerJob) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update(&self, _job: &SchedulerJob) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _id: &str) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        Ok(vec![])
    }
}

// --- テストダブル: SchedulerExecutionRepository のスタブ実装 ---

struct StubExecutionRepository;

#[async_trait]
impl SchedulerExecutionRepository for StubExecutionRepository {
    async fn create(&self, _execution: &SchedulerExecution) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_by_job_id(&self, _job_id: &str) -> anyhow::Result<Vec<SchedulerExecution>> {
        Ok(vec![])
    }

    async fn update_status(
        &self,
        _id: &str,
        _status: String,
        _error_message: Option<String>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<SchedulerExecution>> {
        Ok(None)
    }
}

// --- テストアプリケーション構築 ---

/// スタブリポジトリを使って AppState と Router を構築するヘルパー。
fn make_test_app() -> axum::Router {
    let job_repo: Arc<dyn SchedulerJobRepository> = Arc::new(StubJobRepository);
    let exec_repo: Arc<dyn SchedulerExecutionRepository> = Arc::new(StubExecutionRepository);
    let event_publisher = Arc::new(NoopSchedulerEventPublisher);

    let state = AppState {
        list_jobs_uc: Arc::new(ListJobsUseCase::new(job_repo.clone())),
        create_job_uc: Arc::new(CreateJobUseCase::new(job_repo.clone(), event_publisher)),
        get_job_uc: Arc::new(GetJobUseCase::new(job_repo.clone())),
        delete_job_uc: Arc::new(DeleteJobUseCase::new(job_repo.clone(), exec_repo.clone())),
        pause_job_uc: Arc::new(PauseJobUseCase::new(job_repo.clone())),
        resume_job_uc: Arc::new(ResumeJobUseCase::new(job_repo.clone())),
        update_job_uc: Arc::new(UpdateJobUseCase::new(job_repo.clone())),
        trigger_job_uc: Arc::new(TriggerJobUseCase::new(job_repo, exec_repo)),
        list_executions_uc: Arc::new(ListExecutionsUseCase::new(
            Arc::new(StubJobRepository),
            Arc::new(StubExecutionRepository),
        )),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-scheduler-server-test",
        )),
        auth_state: None,
    };
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

#[tokio::test]
async fn test_api_accessible_without_auth() {
    let app = make_test_app();

    // /api/v1/jobs への GET が 200 を返す（認証なしモード）
    let req = Request::builder()
        .uri("/api/v1/jobs")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
