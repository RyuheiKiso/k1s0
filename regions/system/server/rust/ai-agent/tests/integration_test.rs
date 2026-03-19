#![allow(clippy::unwrap_used)]
// AI Agent サーバーの統合テスト
// router 初期化と healthz エンドポイントの smoke test を行う

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_ai_agent_server::adapter::handler::{router, AppState};
use k1s0_ai_agent_server::domain::entity::{AgentDefinition, Execution};
use k1s0_ai_agent_server::domain::repository::{AgentRepository, ExecutionRepository};
use k1s0_ai_agent_server::domain::service::{ReActEngine, ToolRegistry};
use k1s0_ai_agent_server::usecase::{
    CreateAgentUseCase, ExecuteAgentUseCase, ListExecutionsUseCase, ReviewStepUseCase,
};
use k1s0_bb_ai_client::traits::AiClient;
use k1s0_bb_ai_client::types::{
    AiClientError, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse, ModelInfo,
};

// --- テストダブル ---

/// スタブ: エージェントリポジトリ（空の結果を返す）
struct StubAgentRepository;

#[async_trait::async_trait]
impl AgentRepository for StubAgentRepository {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<AgentDefinition>> {
        Ok(None)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<AgentDefinition>> {
        Ok(vec![])
    }

    async fn save(&self, _agent: &AgentDefinition) -> anyhow::Result<()> {
        Ok(())
    }
}

/// スタブ: 実行リポジトリ（空の結果を返す）
struct StubExecutionRepository;

#[async_trait::async_trait]
impl ExecutionRepository for StubExecutionRepository {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<Execution>> {
        Ok(None)
    }

    async fn save(&self, _execution: &Execution) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_by_agent(&self, _agent_id: &str) -> anyhow::Result<Vec<Execution>> {
        Ok(vec![])
    }
}

/// スタブ: AIクライアント（テストでは実際のLLM呼び出しを行わない）
struct StubAiClient;

#[async_trait::async_trait]
impl AiClient for StubAiClient {
    async fn complete(&self, _req: &CompleteRequest) -> Result<CompleteResponse, AiClientError> {
        Err(AiClientError::HttpError("stub".to_string()))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
        Err(AiClientError::HttpError("stub".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError> {
        Ok(vec![])
    }
}

/// テスト用の AppState とルーターを構築する
fn make_test_app() -> axum::Router {
    // 共有リポジトリのインスタンスを作成
    let agent_repo: Arc<dyn AgentRepository> = Arc::new(StubAgentRepository);
    let execution_repo: Arc<dyn ExecutionRepository> = Arc::new(StubExecutionRepository);
    let ai_client: Arc<dyn AiClient> = Arc::new(StubAiClient);

    // ツールレジストリとReActエンジンを生成
    let tool_registry = ToolRegistry::new();
    let react_engine = Arc::new(ReActEngine::new(tool_registry));

    // 各ユースケースを生成
    let create_agent_uc = Arc::new(CreateAgentUseCase::new(agent_repo.clone()));
    let execute_agent_uc = Arc::new(ExecuteAgentUseCase::new(
        agent_repo.clone(),
        execution_repo.clone(),
        react_engine,
        ai_client,
    ));
    let list_executions_uc = Arc::new(ListExecutionsUseCase::new(execution_repo.clone()));
    let review_step_uc = Arc::new(ReviewStepUseCase::new(execution_repo));

    // メトリクスを生成
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("ai-agent-test"));

    // AppState を構築（認証なし = テストモード）
    let state = AppState {
        create_agent_uc,
        execute_agent_uc,
        list_executions_uc,
        review_step_uc,
        metrics,
        auth_state: None,
    };

    // ルーターを構築（メトリクス無効）
    router(state, false, "/metrics")
}

// --- 統合テスト ---

/// /healthz と /readyz エンドポイントが 200 OK を返すことを確認する
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz のテスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/healthz は 200 を返すべき");

    // /readyz のテスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/readyz は 200 を返すべき");
}
