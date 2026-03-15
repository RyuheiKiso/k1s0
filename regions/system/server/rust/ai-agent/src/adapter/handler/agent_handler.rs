// エージェントAPIハンドラ
// REST APIエンドポイントの実装。エージェントの作成、実行、履歴取得、レビューを処理する

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::AppState;

/// エージェント作成リクエストボディ
#[derive(Debug, Deserialize)]
pub struct CreateAgentBody {
    pub name: String,
    pub description: String,
    pub model_id: String,
    pub system_prompt: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_max_steps")]
    pub max_steps: i32,
}

fn default_max_steps() -> i32 {
    10
}

/// エージェント実行リクエストボディ
#[derive(Debug, Deserialize)]
pub struct ExecuteAgentBody {
    pub input: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub tenant_id: String,
}

/// 実行履歴一覧クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ListExecutionsQuery {
    pub agent_id: String,
}

/// ステップレビューリクエストボディ
#[derive(Debug, Deserialize)]
pub struct ReviewStepBody {
    pub step_index: i32,
    pub approved: bool,
    #[serde(default)]
    pub feedback: String,
}

/// エージェントレスポンス
#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model_id: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
    pub max_steps: i32,
    pub enabled: bool,
}

/// POST /api/v1/agents - エージェントを作成する
pub async fn create_agent(
    State(state): State<AppState>,
    Json(body): Json<CreateAgentBody>,
) -> impl IntoResponse {
    let req = crate::usecase::create_agent::CreateAgentRequest {
        name: body.name,
        description: body.description,
        model_id: body.model_id,
        system_prompt: body.system_prompt,
        tools: body.tools,
        max_steps: body.max_steps,
    };

    match state.create_agent_uc.execute(req).await {
        Ok(resp) => {
            let agent = resp.agent;
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "agent": {
                        "id": agent.id,
                        "name": agent.name,
                        "description": agent.description,
                        "model_id": agent.model_id,
                        "system_prompt": agent.system_prompt,
                        "tools": agent.tools,
                        "max_steps": agent.max_steps,
                        "enabled": agent.enabled,
                    }
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/v1/agents/:id/execute - エージェントを実行する
pub async fn execute_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(body): Json<ExecuteAgentBody>,
) -> impl IntoResponse {
    let req = crate::usecase::execute_agent::ExecuteAgentRequest {
        agent_id,
        input: body.input,
        session_id: if body.session_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            body.session_id
        },
        tenant_id: body.tenant_id,
    };

    match state.execute_agent_uc.execute(req).await {
        Ok(resp) => {
            let exec = resp.execution;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "execution_id": exec.id,
                    "status": exec.status.to_string(),
                    "output": exec.output,
                    "steps": exec.steps,
                })),
            )
                .into_response()
        }
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// GET /api/v1/executions - 実行履歴を取得する
pub async fn list_executions(
    State(state): State<AppState>,
    Query(query): Query<ListExecutionsQuery>,
) -> impl IntoResponse {
    let req = crate::usecase::list_executions::ListExecutionsRequest {
        agent_id: query.agent_id,
    };

    match state.list_executions_uc.execute(req).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(serde_json::json!({"executions": resp.executions})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/v1/executions/:id/review - 実行ステップをレビューする
pub async fn review_step(
    State(state): State<AppState>,
    Path(execution_id): Path<String>,
    Json(body): Json<ReviewStepBody>,
) -> impl IntoResponse {
    let req = crate::usecase::review_step::ReviewStepRequest {
        execution_id,
        step_index: body.step_index,
        approved: body.approved,
        feedback: body.feedback,
    };

    match state.review_step_uc.execute(req).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "execution_id": resp.execution_id,
                "resumed": resp.resumed,
            })),
        )
            .into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}
