// ハンドラモジュール
// REST APIルーターとAppState定義を管理する

pub mod agent_handler;
pub mod health;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::auth::AgentAuthState;
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CreateAgentUseCase, ExecuteAgentUseCase, ListExecutionsUseCase, ReviewStepUseCase,
};

/// AppState はREST APIハンドラの共有状態を保持する
#[derive(Clone)]
pub struct AppState {
    /// エージェント作成ユースケース
    pub create_agent_uc: Arc<CreateAgentUseCase>,
    /// エージェント実行ユースケース
    pub execute_agent_uc: Arc<ExecuteAgentUseCase>,
    /// 実行履歴一覧ユースケース
    pub list_executions_uc: Arc<ListExecutionsUseCase>,
    /// ステップレビューユースケース
    pub review_step_uc: Arc<ReviewStepUseCase>,
    /// メトリクス
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    /// 認証状態
    pub auth_state: Option<AgentAuthState>,
}

impl AppState {
    /// 認証状態を設定する
    pub fn with_auth(mut self, auth_state: AgentAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

/// REST APIルーターを構築する
pub fn router(state: AppState, metrics_enabled: bool, metrics_path: &str) -> Router {
    // 認証不要のエンドポイント
    let mut public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz));
    if metrics_enabled {
        public_routes = public_routes.route(metrics_path, get(metrics_handler));
    }

    // 認証が設定されている場合はRBAC付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> agents/read
        let read_routes = Router::new()
            .route("/api/v1/executions", get(agent_handler::list_executions))
            .route_layer(axum::middleware::from_fn(require_permission(
                "agents", "read",
            )));

        // POST -> agents/write
        let write_routes = Router::new()
            .route("/api/v1/agents", post(agent_handler::create_agent))
            .route(
                "/api/v1/agents/{id}/execute",
                post(agent_handler::execute_agent),
            )
            .route(
                "/api/v1/executions/{id}/review",
                post(agent_handler::review_step),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "agents", "write",
            )));

        Router::new().merge(read_routes).merge(write_routes).layer(
            axum::middleware::from_fn_with_state(auth_state.clone(), auth_middleware),
        )
    } else {
        // 認証なし（dev モード / テスト）
        Router::new()
            .route("/api/v1/agents", post(agent_handler::create_agent))
            .route(
                "/api/v1/agents/{id}/execute",
                post(agent_handler::execute_agent),
            )
            .route("/api/v1/executions", get(agent_handler::list_executions))
            .route(
                "/api/v1/executions/{id}/review",
                post(agent_handler::review_step),
            )
    };

    public_routes.merge(api_routes).with_state(state)
}

/// メトリクスエンドポイントハンドラ
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
