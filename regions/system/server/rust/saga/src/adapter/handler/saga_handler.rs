use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use k1s0_auth::Claims;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::SagaError;
use super::AppState;
use crate::domain::entity::saga_state::SagaStatus;
use crate::domain::repository::saga_repository::SagaListParams;
use crate::usecase::{CancelSagaError, CompensateSagaError};

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StartSagaRequest {
    pub workflow_name: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StartSagaResponse {
    pub saga_id: String,
    pub status: String,
}

/// Saga 一覧取得のクエリパラメータ。
/// cursor が指定された場合は keyset ページネーションを優先する。
/// 後方互換のため page / `page_size` も維持する。
#[derive(Debug, Deserialize)]
pub struct ListSagasQuery {
    pub workflow_name: Option<String>,
    pub status: Option<String>,
    pub correlation_id: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    /// keyset ページネーション用カーソル。形式: "{`created_at_unix_ms`}_{id}"
    /// 指定された場合は OFFSET より優先される。
    pub cursor: Option<String>,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SagaResponse {
    pub saga_id: String,
    pub workflow_name: String,
    pub current_step: i32,
    pub status: String,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SagaDetailResponse {
    pub saga: SagaResponse,
    pub step_logs: Vec<StepLogResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StepLogResponse {
    pub id: String,
    pub saga_id: String,
    pub step_index: i32,
    pub step_name: String,
    pub action: String,
    pub status: String,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListSagasResponse {
    pub sagas: Vec<SagaResponse>,
    pub pagination: PaginationResponse,
}

/// ページネーション情報レスポンス。
/// keyset ページネーション使用時は `next_cursor` に次ページのカーソル値が入る。
/// cursor が None の場合はリストの末尾に達したことを示す。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationResponse {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
    /// 次ページ取得用カーソル。形式: "{`created_at_unix_ms`}_{id}"
    /// keyset ページネーション使用時のみ設定される。
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterWorkflowRequest {
    pub workflow_yaml: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegisterWorkflowResponse {
    pub name: String,
    pub step_count: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkflowSummaryResponse {
    pub name: String,
    pub step_count: usize,
    pub step_names: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowSummaryResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CancelSagaResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CompensateSagaResponse {
    pub success: bool,
    pub saga_id: String,
    pub status: String,
    pub message: String,
}

// --- Handlers ---

/// liveness probe: プロセスが起動していれば常に ok を返す。
/// CRITICAL-003 監査対応: DB 確認は readyz に移動し、healthz は liveness のみとする。
#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "Health check OK")))]
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// readiness probe: DB 接続確認を行い、サービスがリクエスト受付可能かを返す。
/// CRITICAL-003 監査対応: Docker Compose の healthcheck および Kubernetes readinessProbe として使用する。
/// DB が設定されている場合は SELECT 1 で疎通確認し、失敗時は 503 を返す。
#[utoipa::path(get, path = "/readyz", responses((status = 200, description = "Ready"), (status = 503, description = "Not ready")))]
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            // ADR-0068 準拠: "healthy"/"unhealthy" + checks + timestamp
            Ok(_) => Json(serde_json::json!({
                "status": "healthy",
                "checks": { "database": "ok" },
                "timestamp": Utc::now().to_rfc3339()
            }))
            .into_response(),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "unhealthy",
                    "checks": { "database": "error" },
                    "timestamp": Utc::now().to_rfc3339()
                })),
            )
                .into_response(),
        }
    } else {
        // DB 未構成でも起動完了とみなす（ADR-0068 準拠）
        Json(serde_json::json!({
            "status": "healthy",
            "checks": { "database": "not_configured" },
            "timestamp": Utc::now().to_rfc3339()
        }))
        .into_response()
    }
}

#[utoipa::path(get, path = "/metrics", responses((status = 200, description = "Prometheus metrics")))]
pub async fn metrics(State(state): State<AppState>) -> String {
    state.metrics.gather_metrics()
}

#[utoipa::path(
    post,
    path = "/api/v1/sagas",
    request_body = StartSagaRequest,
    responses(
        (status = 201, description = "Saga started", body = StartSagaResponse),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn start_saga(
    State(state): State<AppState>,
    // CRIT-005 対応: Claims から tenant_id を取得する（auth middleware が挿入）
    claims: Option<Extension<Claims>>,
    Json(req): Json<StartSagaRequest>,
) -> Result<(StatusCode, Json<StartSagaResponse>), SagaError> {
    if req.workflow_name.is_empty() {
        return Err(SagaError::Validation(
            "workflow_name is required".to_string(),
        ));
    }

    // テナント ID を Claims から取得する。Claims がない場合は "system" を使用する
    let tenant_id = claims
        .as_ref()
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string());

    // ドメインエラー（SagaError）をアダプタ層のハンドラーエラー型に型安全に変換する
    let saga_id = state
        .start_saga_uc
        .execute(
            req.workflow_name,
            req.payload,
            req.correlation_id,
            req.initiated_by,
            tenant_id,
        )
        .await
        .map_err(|e| {
            use crate::domain::error::SagaError as DomainSagaError;
            match e {
                DomainSagaError::NotFound(msg) => SagaError::NotFound(msg),
                DomainSagaError::ValidationFailed(msg) => SagaError::Validation(msg),
                other => SagaError::Internal(other.to_string()),
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(StartSagaResponse {
            saga_id: saga_id.to_string(),
            status: "STARTED".to_string(),
        }),
    ))
}

/// Saga 一覧を取得する。
/// cursor が指定された場合は keyset ページネーションを使用し、
/// 指定されない場合は OFFSET ベースのページネーション（後方互換）を使用する。
#[utoipa::path(
    get,
    path = "/api/v1/sagas",
    params(
        ("workflow_name" = Option<String>, Query, description = "ワークフロー名でフィルタ"),
        ("status" = Option<String>, Query, description = "ステータスでフィルタ"),
        ("page" = Option<i32>, Query, description = "ページ番号（cursor 未指定時に有効）"),
        ("page_size" = Option<i32>, Query, description = "1ページあたりの件数"),
        ("cursor" = Option<String>, Query, description = "keysetページネーション用カーソル。形式: {created_at_unix_ms}_{id}"),
    ),
    responses(
        (status = 200, description = "Saga 一覧", body = ListSagasResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_sagas(
    State(state): State<AppState>,
    // CRIT-005 対応: Claims から tenant_id を取得する
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListSagasQuery>,
) -> Result<Json<ListSagasResponse>, SagaError> {
    // ステータス文字列をドメイン型に変換する
    let status = if let Some(ref s) = query.status {
        Some(SagaStatus::from_str_value(s).map_err(|e| SagaError::Validation(e.to_string()))?)
    } else {
        None
    };

    // テナント ID を Claims から取得する
    let tenant_id = claims
        .as_ref()
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string());

    // cursor が指定された場合は keyset ページネーション、未指定の場合は OFFSET を使用する
    let params = SagaListParams {
        workflow_name: query.workflow_name.clone(),
        status,
        correlation_id: query.correlation_id.clone(),
        page: query.page,
        page_size: query.page_size,
        cursor: query.cursor.clone(),
        tenant_id,
    };

    let (sagas, total) = state
        .list_sagas_uc
        .execute(params)
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?;

    // 最後のレコードから次ページのカーソル値を生成する
    // 形式: "{created_at_unix_ms}_{id}"
    let next_cursor = sagas.last().map(|s| {
        let ts_ms = s.created_at.timestamp_millis();
        format!("{}_{}", ts_ms, s.saga_id)
    });

    // cursor 使用時は next_cursor の有無で has_next を判定する
    // OFFSET 使用時は従来通り total_count で判定する
    let total_i64 = i64::from(total);
    let page_size_i32 = query.page_size;
    let has_next = if query.cursor.is_some() {
        // keyset ページネーション: 取得件数が page_size と同じなら次ページあり
        // LOW-008: 安全な型変換（オーバーフロー防止）
        i32::try_from(sagas.len()).unwrap_or(i32::MAX) == page_size_i32 && next_cursor.is_some()
    } else {
        // OFFSET ページネーション: 従来通りの計算
        (i64::from(query.page) * i64::from(query.page_size)) < total_i64
    };

    let saga_responses: Vec<SagaResponse> = sagas
        .into_iter()
        .map(|s| SagaResponse {
            saga_id: s.saga_id.to_string(),
            workflow_name: s.workflow_name,
            current_step: s.current_step,
            status: s.status.to_string(),
            payload: s.payload,
            correlation_id: s.correlation_id,
            initiated_by: s.initiated_by,
            error_message: s.error_message,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListSagasResponse {
        sagas: saga_responses,
        pagination: PaginationResponse {
            total_count: total_i64,
            page: query.page,
            page_size: query.page_size,
            has_next,
            // keyset ページネーション使用時のみ next_cursor を返す
            next_cursor: if query.cursor.is_some() || has_next {
                next_cursor
            } else {
                None
            },
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/sagas/{saga_id}",
    params(("saga_id" = String, Path, description = "Saga ID")),
    responses(
        (status = 200, description = "Saga detail", body = SagaDetailResponse),
        (status = 404, description = "Saga not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_saga(
    State(state): State<AppState>,
    // CRIT-005 対応: Claims から tenant_id を取得する
    claims: Option<Extension<Claims>>,
    Path(saga_id): Path<String>,
) -> Result<Json<SagaDetailResponse>, SagaError> {
    let id = Uuid::parse_str(&saga_id)
        .map_err(|_| SagaError::Validation(format!("invalid saga_id: {saga_id}")))?;

    let tenant_id = claims
        .as_ref()
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string());

    let (saga, step_logs) = state
        .get_saga_uc
        .execute(id, &tenant_id)
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?
        .ok_or_else(|| SagaError::NotFound(format!("saga not found: {saga_id}")))?;

    let step_log_responses: Vec<StepLogResponse> = step_logs
        .into_iter()
        .map(|l| StepLogResponse {
            id: l.id.to_string(),
            saga_id: l.saga_id.to_string(),
            step_index: l.step_index,
            step_name: l.step_name,
            action: l.action.to_string(),
            status: l.status.to_string(),
            request_payload: l.request_payload,
            response_payload: l.response_payload,
            error_message: l.error_message,
            started_at: l.started_at.to_rfc3339(),
            completed_at: l.completed_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(SagaDetailResponse {
        saga: SagaResponse {
            saga_id: saga.saga_id.to_string(),
            workflow_name: saga.workflow_name,
            current_step: saga.current_step,
            status: saga.status.to_string(),
            payload: saga.payload,
            correlation_id: saga.correlation_id,
            initiated_by: saga.initiated_by,
            error_message: saga.error_message,
            created_at: saga.created_at.to_rfc3339(),
            updated_at: saga.updated_at.to_rfc3339(),
        },
        step_logs: step_log_responses,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/sagas/{saga_id}/cancel",
    params(("saga_id" = String, Path, description = "Saga ID")),
    responses(
        (status = 200, description = "Saga cancelled", body = CancelSagaResponse),
        (status = 404, description = "Saga not found"),
        (status = 409, description = "Already terminal"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_saga(
    State(state): State<AppState>,
    // CRIT-005 対応: Claims から tenant_id を取得する
    claims: Option<Extension<Claims>>,
    Path(saga_id): Path<String>,
) -> Result<Json<CancelSagaResponse>, SagaError> {
    let id = Uuid::parse_str(&saga_id)
        .map_err(|_| SagaError::Validation(format!("invalid saga_id: {saga_id}")))?;

    let tenant_id = claims
        .as_ref()
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string());

    state
        .cancel_saga_uc
        .execute(id, &tenant_id)
        .await
        .map_err(|e| match e {
            CancelSagaError::NotFound(_) => SagaError::NotFound(e.to_string()),
            CancelSagaError::AlreadyTerminal(_) => SagaError::Conflict(e.to_string()),
            CancelSagaError::Internal(_) => SagaError::Internal(e.to_string()),
        })?;

    Ok(Json(CancelSagaResponse {
        success: true,
        message: format!("saga {saga_id} cancelled"),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/sagas/{saga_id}/compensate",
    params(("saga_id" = String, Path, description = "Saga ID")),
    responses(
        (status = 200, description = "Compensation triggered", body = CompensateSagaResponse),
        (status = 404, description = "Saga not found"),
        (status = 409, description = "Already terminal"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn compensate_saga(
    State(state): State<AppState>,
    // CRIT-005 対応: Claims から tenant_id を取得する
    claims: Option<Extension<Claims>>,
    Path(saga_id): Path<String>,
) -> Result<Json<CompensateSagaResponse>, SagaError> {
    let id = Uuid::parse_str(&saga_id)
        .map_err(|_| SagaError::Validation(format!("invalid saga_id: {saga_id}")))?;

    let tenant_id = claims
        .as_ref()
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string());

    let updated = state
        .execute_saga_uc
        .trigger_compensate(id, &tenant_id)
        .await
        // NotFound と WorkflowNotFound は同じエラーに変換するためアームを統合する
        .map_err(|e| match e {
            CompensateSagaError::NotFound(_) | CompensateSagaError::WorkflowNotFound(_) => SagaError::NotFound(e.to_string()),
            CompensateSagaError::AlreadyTerminal(_) => SagaError::Conflict(e.to_string()),
            CompensateSagaError::Internal(_) => SagaError::Internal(e.to_string()),
        })?;

    Ok(Json(CompensateSagaResponse {
        success: true,
        saga_id: updated.saga_id.to_string(),
        status: updated.status.to_string(),
        message: format!("saga {saga_id} compensation completed"),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows",
    request_body = RegisterWorkflowRequest,
    responses(
        (status = 201, description = "Workflow registered", body = RegisterWorkflowResponse),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn register_workflow(
    State(state): State<AppState>,
    Json(req): Json<RegisterWorkflowRequest>,
) -> Result<(StatusCode, Json<RegisterWorkflowResponse>), SagaError> {
    if req.workflow_yaml.is_empty() {
        return Err(SagaError::Validation(
            "workflow_yaml is required".to_string(),
        ));
    }

    let (name, step_count) = state
        .register_workflow_uc
        .execute(req.workflow_yaml)
        .await
        .map_err(|e| SagaError::Validation(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterWorkflowResponse { name, step_count }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows",
    responses((status = 200, description = "Workflow list", body = ListWorkflowsResponse)),
    security(("bearer_auth" = []))
)]
pub async fn list_workflows(
    State(state): State<AppState>,
) -> Result<Json<ListWorkflowsResponse>, SagaError> {
    let workflows = state
        .list_workflows_uc
        .execute()
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?;

    let summaries: Vec<WorkflowSummaryResponse> = workflows
        .into_iter()
        .map(|w| WorkflowSummaryResponse {
            step_count: w.steps.len(),
            step_names: w.steps.iter().map(|s| s.name.clone()).collect(),
            name: w.name,
        })
        .collect();

    Ok(Json(ListWorkflowsResponse {
        workflows: summaries,
    }))
}
