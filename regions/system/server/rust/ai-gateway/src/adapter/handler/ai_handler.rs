// AI Gateway REST APIハンドラーの実装。
// テキスト補完、エンベディング、モデル一覧、使用量取得のエンドポイント。

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use k1s0_server_common::ErrorResponse;
use serde::Deserialize;

use crate::adapter::middleware::auth::AuthState;
use crate::usecase::complete::{CompleteError, CompleteInput};
use crate::usecase::embed::{EmbedError, EmbedInput};
use crate::usecase::get_usage::GetUsageInput;
use crate::usecase::{CompleteUseCase, EmbedUseCase, GetUsageUseCase, ListModelsUseCase};

/// アプリケーション共有状態。
/// 各ユースケースとメトリクスを保持する。
#[derive(Clone)]
pub struct AppState {
    /// テキスト補完ユースケース
    pub complete_uc: Arc<CompleteUseCase>,
    /// エンベディングユースケース
    pub embed_uc: Arc<EmbedUseCase>,
    /// モデル一覧ユースケース
    pub list_models_uc: Arc<ListModelsUseCase>,
    /// 使用量取得ユースケース
    pub get_usage_uc: Arc<GetUsageUseCase>,
    /// メトリクス
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    /// 認証状態（オプション）
    pub auth_state: Option<AuthState>,
    /// DB 接続確認用のコネクションプール（CRITICAL-003 対応: /readyz で SELECT 1 チェックに使用）
    pub db_pool: Option<sqlx::PgPool>,
}

impl AppState {
    /// 認証状態を設定する。
    #[must_use]
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

/// テキスト補完エンドポイント。
/// POST /api/v1/complete
pub async fn complete(
    State(state): State<AppState>,
    Json(input): Json<CompleteInput>,
) -> impl IntoResponse {
    match state.complete_uc.execute(input).await {
        Ok(output) => (StatusCode::OK, Json(output)).into_response(),
        Err(e) => match e {
            CompleteError::GuardrailViolation(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("AI_GUARDRAIL_VIOLATION", &msg)),
            )
                .into_response(),
            CompleteError::ModelNotFound(msg) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("AI_MODEL_NOT_FOUND", &msg)),
            )
                .into_response(),
            CompleteError::LlmError(msg) => (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::new("AI_LLM_ERROR", &msg)),
            )
                .into_response(),
            CompleteError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("AI_INTERNAL_ERROR", &msg)),
            )
                .into_response(),
        },
    }
}

/// エンベディングエンドポイント。
/// POST /api/v1/embed
pub async fn embed(
    State(state): State<AppState>,
    Json(input): Json<EmbedInput>,
) -> impl IntoResponse {
    match state.embed_uc.execute(input).await {
        Ok(output) => (StatusCode::OK, Json(output)).into_response(),
        Err(EmbedError::LlmError(msg)) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse::new("AI_LLM_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// モデル一覧エンドポイント。
/// GET /api/v1/models
pub async fn list_models(State(state): State<AppState>) -> impl IntoResponse {
    let output = state.list_models_uc.execute().await;
    (StatusCode::OK, Json(output))
}

/// 使用量取得クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    pub tenant_id: String,
    pub start: String,
    pub end: String,
}

/// 使用量取得エンドポイント。
/// GET /`api/v1/usage?tenant_id=xxx&start=xxx&end=xxx`
pub async fn get_usage(
    State(state): State<AppState>,
    Query(query): Query<UsageQuery>,
) -> impl IntoResponse {
    let input = GetUsageInput {
        tenant_id: query.tenant_id,
        start: query.start,
        end: query.end,
    };
    let output = state.get_usage_uc.execute(input).await;
    (StatusCode::OK, Json(output))
}
