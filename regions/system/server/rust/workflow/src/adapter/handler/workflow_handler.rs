// ワークフローCRUDハンドラ
// ワークフロー定義の作成・取得・一覧・更新・削除操作を提供する

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::usecase::create_workflow::{CreateWorkflowError, CreateWorkflowInput};
use crate::usecase::delete_workflow::{DeleteWorkflowError, DeleteWorkflowInput};
use crate::usecase::get_workflow::{GetWorkflowError, GetWorkflowInput};
use crate::usecase::list_workflows::{ListWorkflowsError, ListWorkflowsInput};
use crate::usecase::update_workflow::{UpdateWorkflowError, UpdateWorkflowInput};

use super::dto::{
    error_json, to_step_response, AppState, CreateWorkflowRequest, ListWorkflowsQuery,
    ListWorkflowsResponse, PaginationResponse, UpdateWorkflowRequest, WorkflowResponse,
};

/// POST /api/v1/workflows
/// 新しいワークフロー定義を作成する
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    use crate::domain::entity::workflow_step::WorkflowStep;

    // リクエストのステップ情報をドメインエンティティに変換
    let steps: Vec<WorkflowStep> = req
        .steps
        .into_iter()
        .map(|s| {
            WorkflowStep::new(
                s.step_id,
                s.name,
                s.step_type,
                s.assignee_role,
                s.timeout_hours,
                s.on_approve,
                s.on_reject,
            )
        })
        .collect();

    // ユースケース入力を組み立てる
    let input = CreateWorkflowInput {
        name: req.name,
        description: req.description,
        enabled: req.enabled,
        steps,
    };

    match state.create_workflow_uc.execute(&input).await {
        Ok(def) => {
            // 成功時はワークフロー定義をレスポンスに変換
            let step_count = def.step_count();
            let resp = WorkflowResponse {
                id: def.id,
                name: def.name,
                description: def.description,
                version: def.version,
                enabled: def.enabled,
                step_count,
                steps: def.steps.iter().map(to_step_response).collect(),
                created_at: def.created_at.to_rfc3339(),
                updated_at: def.updated_at.to_rfc3339(),
            };
            // レスポンスDTOを直接 Json として返す（.expect() 排除）
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        // 同名のワークフローが既に存在する場合
        Err(CreateWorkflowError::AlreadyExists(name)) => (
            StatusCode::CONFLICT,
            Json(error_json(
                "SYS_WORKFLOW_ALREADY_EXISTS",
                &format!("workflow already exists: {}", name),
            )),
        )
            .into_response(),
        // バリデーションエラー
        Err(CreateWorkflowError::Validation(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(error_json("SYS_WORKFLOW_VALIDATION_ERROR", &msg)),
        )
            .into_response(),
        // 内部エラー
        Err(CreateWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// GET /api/v1/workflows/:id
/// 指定されたワークフロー定義を取得する
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetWorkflowInput { id: id.clone() };

    match state.get_workflow_uc.execute(&input).await {
        Ok(def) => {
            // ワークフロー定義をレスポンスDTOに変換
            let step_count = def.step_count();
            let resp = WorkflowResponse {
                id: def.id,
                name: def.name,
                description: def.description,
                version: def.version,
                enabled: def.enabled,
                step_count,
                steps: def.steps.iter().map(to_step_response).collect(),
                created_at: def.created_at.to_rfc3339(),
                updated_at: def.updated_at.to_rfc3339(),
            };
            // レスポンスDTOを直接 Json として返す（.expect() 排除）
            (StatusCode::OK, Json(resp)).into_response()
        }
        // ワークフローが見つからない場合
        Err(GetWorkflowError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_NOT_FOUND",
                &format!("workflow not found: {}", id),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(GetWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// GET /api/v1/workflows
/// フィルタ条件に基づいてワークフロー一覧を取得する
pub async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<ListWorkflowsQuery>,
) -> impl IntoResponse {
    // クエリパラメータからユースケース入力を組み立てる
    let input = ListWorkflowsInput {
        enabled_only: query.enabled_only,
        page: query.page,
        page_size: query.page_size,
    };

    match state.list_workflows_uc.execute(&input).await {
        Ok(output) => {
            // 各ワークフロー定義をレスポンスDTOに変換
            let resp = ListWorkflowsResponse {
                workflows: output
                    .workflows
                    .into_iter()
                    .map(|def| {
                        let step_count = def.step_count();
                        WorkflowResponse {
                            id: def.id,
                            name: def.name,
                            description: def.description,
                            version: def.version,
                            enabled: def.enabled,
                            step_count,
                            steps: def.steps.iter().map(to_step_response).collect(),
                            created_at: def.created_at.to_rfc3339(),
                            updated_at: def.updated_at.to_rfc3339(),
                        }
                    })
                    .collect(),
                pagination: PaginationResponse {
                    total_count: output.total_count,
                    page: output.page,
                    page_size: output.page_size,
                    has_next: output.has_next,
                },
            };
            // レスポンスDTOを直接 Json として返す（.expect() 排除）
            (StatusCode::OK, Json(resp)).into_response()
        }
        // 内部エラー
        Err(ListWorkflowsError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// PUT /api/v1/workflows/:id
/// 既存のワークフロー定義を更新する
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> impl IntoResponse {
    use crate::domain::entity::workflow_step::WorkflowStep;

    // ステップ情報がある場合のみドメインエンティティに変換
    let steps = req.steps.map(|steps| {
        steps
            .into_iter()
            .map(|s| {
                WorkflowStep::new(
                    s.step_id,
                    s.name,
                    s.step_type,
                    s.assignee_role,
                    s.timeout_hours,
                    s.on_approve,
                    s.on_reject,
                )
            })
            .collect()
    });

    // ユースケース入力を組み立てる
    let input = UpdateWorkflowInput {
        id: id.clone(),
        name: req.name,
        description: req.description,
        enabled: req.enabled,
        steps,
    };

    match state.update_workflow_uc.execute(&input).await {
        Ok(def) => {
            // 更新後のワークフロー定義をレスポンスに変換
            let step_count = def.step_count();
            let resp = WorkflowResponse {
                id: def.id,
                name: def.name,
                description: def.description,
                version: def.version,
                enabled: def.enabled,
                step_count,
                steps: def.steps.iter().map(to_step_response).collect(),
                created_at: def.created_at.to_rfc3339(),
                updated_at: def.updated_at.to_rfc3339(),
            };
            // レスポンスDTOを直接 Json として返す（.expect() 排除）
            (StatusCode::OK, Json(resp)).into_response()
        }
        // ワークフローが見つからない場合
        Err(UpdateWorkflowError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_NOT_FOUND",
                &format!("workflow not found: {}", id),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(UpdateWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/workflows/:id
/// 指定されたワークフロー定義を削除する
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = DeleteWorkflowInput { id: id.clone() };

    match state.delete_workflow_uc.execute(&input).await {
        // 成功時はコンテンツなしで204を返す
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        // ワークフローが見つからない場合
        Err(DeleteWorkflowError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_NOT_FOUND",
                &format!("workflow not found: {}", id),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(DeleteWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}
