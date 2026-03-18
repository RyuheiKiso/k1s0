// インスタンス管理ハンドラ
// ワークフローインスタンスの実行・取得・一覧・キャンセル操作を提供する

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::usecase::cancel_instance::{CancelInstanceError, CancelInstanceInput};
use crate::usecase::get_instance::{GetInstanceError, GetInstanceInput};
use crate::usecase::list_instances::{ListInstancesError, ListInstancesInput};
use crate::usecase::start_instance::{StartInstanceError, StartInstanceInput};

use super::dto::{
    error_json, AppState, CancelInstanceRequest, ExecuteWorkflowRequest, ExecuteWorkflowResponse,
    InstanceStatusResponse, ListInstancesQuery,
};

/// POST /api/v1/workflows/:id/execute
/// 指定されたワークフローの新しいインスタンスを開始する
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteWorkflowRequest>,
) -> impl IntoResponse {
    // リクエストからユースケース入力を組み立てる
    let input = StartInstanceInput {
        workflow_id: id.clone(),
        title: req.title,
        initiator_id: req.initiator_id,
        context: req.context,
    };

    match state.start_instance_uc.execute(&input).await {
        Ok(output) => {
            // 成功時はインスタンス情報をレスポンスに変換
            let resp = ExecuteWorkflowResponse {
                id: output.instance.id,
                workflow_id: output.instance.workflow_id,
                workflow_name: output.instance.workflow_name,
                title: output.instance.title,
                initiator_id: output.instance.initiator_id,
                context: output.instance.context,
                status: output.instance.status,
                current_step_id: output.instance.current_step_id,
                started_at: output.instance.started_at.to_rfc3339(),
                created_at: output.instance.created_at.to_rfc3339(),
                completed_at: output.instance.completed_at.map(|t| t.to_rfc3339()),
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).expect("インスタンス実行レスポンスのJSON変換に失敗")),
            )
                .into_response()
        }
        // ワークフローが見つからない場合
        Err(StartInstanceError::WorkflowNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_NOT_FOUND",
                &format!("workflow not found: {}", id),
            )),
        )
            .into_response(),
        // ワークフローが無効の場合
        Err(StartInstanceError::WorkflowDisabled(_)) => (
            StatusCode::CONFLICT,
            Json(error_json(
                "SYS_WORKFLOW_DISABLED",
                &format!("workflow is disabled: {}", id),
            )),
        )
            .into_response(),
        // ワークフローにステップが定義されていない場合
        Err(StartInstanceError::NoSteps(_)) => (
            StatusCode::BAD_REQUEST,
            Json(error_json(
                "SYS_WORKFLOW_NO_STEPS",
                &format!("workflow has no steps: {}", id),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(StartInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// GET /api/v1/instances/:id/status
/// インスタンスのステータス情報を取得する
pub async fn get_instance_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetInstanceInput { id: id.clone() };

    match state.get_instance_uc.execute(&input).await {
        Ok(inst) => {
            // インスタンス情報をレスポンスDTOに変換
            let resp = InstanceStatusResponse {
                id: inst.id,
                workflow_id: inst.workflow_id,
                workflow_name: inst.workflow_name,
                title: inst.title,
                initiator_id: inst.initiator_id,
                context: inst.context,
                status: inst.status,
                current_step_id: inst.current_step_id,
                started_at: inst.started_at.to_rfc3339(),
                created_at: inst.created_at.to_rfc3339(),
                completed_at: inst.completed_at.map(|t| t.to_rfc3339()),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("インスタンスステータスレスポンスのJSON変換に失敗"))).into_response()
        }
        // インスタンスが見つからない場合
        Err(GetInstanceError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_INSTANCE_NOT_FOUND",
                &format!("instance not found: {}", id),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(GetInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// GET /api/v1/instances
/// フィルタ条件に基づいてインスタンス一覧を取得する
pub async fn list_instances(
    State(state): State<AppState>,
    Query(query): Query<ListInstancesQuery>,
) -> impl IntoResponse {
    // クエリパラメータからユースケース入力を組み立てる
    let input = ListInstancesInput {
        status: query.status,
        workflow_id: query.workflow_id,
        initiator_id: query.initiator_id,
        page: query.page,
        page_size: query.page_size,
    };

    match state.list_instances_uc.execute(&input).await {
        Ok(output) => {
            // 各インスタンスをJSON値に変換
            let instances: Vec<serde_json::Value> = output
                .instances
                .into_iter()
                .map(|inst| {
                    serde_json::json!({
                        "id": inst.id,
                        "workflow_id": inst.workflow_id,
                        "workflow_name": inst.workflow_name,
                        "title": inst.title,
                        "initiator_id": inst.initiator_id,
                        "status": inst.status,
                        "current_step_id": inst.current_step_id,
                        "started_at": inst.started_at.to_rfc3339(),
                        "created_at": inst.created_at.to_rfc3339(),
                        "completed_at": inst.completed_at.map(|t| t.to_rfc3339()),
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "instances": instances,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        // 内部エラー
        Err(ListInstancesError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// GET /api/v1/instances/:id
/// 指定されたインスタンスの詳細情報を取得する
pub async fn get_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetInstanceInput { id: id.clone() };

    match state.get_instance_uc.execute(&input).await {
        Ok(inst) => {
            // インスタンス情報をレスポンスDTOに変換
            let resp = InstanceStatusResponse {
                id: inst.id,
                workflow_id: inst.workflow_id,
                workflow_name: inst.workflow_name,
                title: inst.title,
                initiator_id: inst.initiator_id,
                context: inst.context,
                status: inst.status,
                current_step_id: inst.current_step_id,
                started_at: inst.started_at.to_rfc3339(),
                created_at: inst.created_at.to_rfc3339(),
                completed_at: inst.completed_at.map(|t| t.to_rfc3339()),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("インスタンス詳細レスポンスのJSON変換に失敗"))).into_response()
        }
        // インスタンスが見つからない場合
        Err(GetInstanceError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_INSTANCE_NOT_FOUND",
                &format!("instance not found: {}", id),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(GetInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// POST /api/v1/instances/:id/cancel
/// 実行中のインスタンスをキャンセルする
pub async fn cancel_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CancelInstanceRequest>,
) -> impl IntoResponse {
    let input = CancelInstanceInput {
        id: id.clone(),
        reason: req.reason,
    };

    match state.cancel_instance_uc.execute(&input).await {
        Ok(inst) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": inst.id,
                "status": inst.status,
                "cancelled_at": inst.completed_at.map(|t| t.to_rfc3339()),
                "message": "instance cancelled"
            })),
        )
            .into_response(),
        // インスタンスが見つからない場合
        Err(CancelInstanceError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_INSTANCE_NOT_FOUND",
                &format!("instance not found: {}", id),
            )),
        )
            .into_response(),
        // キャンセルできないステータスの場合
        Err(CancelInstanceError::InvalidStatus(_, status)) => (
            StatusCode::CONFLICT,
            Json(error_json(
                "SYS_WORKFLOW_INSTANCE_INVALID_STATUS",
                &format!("cannot cancel instance with status: {}", status),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(CancelInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}
