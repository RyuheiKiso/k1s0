//! REST API実装
//!
//! axumを使用したREST APIハンドラーを提供する。

use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::application::ConfigService;
use crate::domain::{SettingQuery as DomainSettingQuery, SettingRepository};
use crate::infrastructure::cache::SettingCache;

/// REST APIの状態
pub struct RestState<R, C>
where
    R: SettingRepository + 'static,
    C: SettingCache + 'static,
{
    pub service: Arc<ConfigService<R, C>>,
}

impl<R, C> Clone for RestState<R, C>
where
    R: SettingRepository + 'static,
    C: SettingCache + 'static,
{
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
        }
    }
}

/// 設定レスポンス
#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub setting_id: i64,
    pub service_name: String,
    pub env: String,
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub description: Option<String>,
    pub is_active: bool,
}

/// 設定リストレスポンス
#[derive(Debug, Serialize)]
pub struct SettingListResponse {
    pub settings: Vec<SettingResponse>,
    pub next_page_token: Option<String>,
}

/// 設定取得クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct GetSettingQuery {
    #[serde(default)]
    pub env: Option<String>,
}

/// 設定リストクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ListSettingsQuery {
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub key_prefix: Option<String>,
    #[serde(default)]
    pub env: Option<String>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub page_token: Option<String>,
}

/// 設定更新リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: String,
    #[serde(default)]
    pub value_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// エラーレスポンス
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// REST APIルーターを作成
pub fn create_router<R, C>(service: Arc<ConfigService<R, C>>) -> Router
where
    R: SettingRepository + Send + Sync + 'static,
    C: SettingCache + Send + Sync + 'static,
{
    let state = RestState { service };

    Router::new()
        .route("/v1/settings", get(list_settings))
        .route(
            "/v1/settings/:service_name/:key",
            get(get_setting).put(update_setting).delete(delete_setting),
        )
        .route("/v1/settings/:service_name/:key/refresh", post(refresh_setting))
        .route("/health", get(health_check))
        .with_state(state)
}

/// ヘルスチェック
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// 設定取得
async fn get_setting<R, C>(
    State(state): State<RestState<R, C>>,
    Path((service_name, key)): Path<(String, String)>,
    Query(query): Query<GetSettingQuery>,
) -> impl IntoResponse
where
    R: SettingRepository + Send + Sync + 'static,
    C: SettingCache + Send + Sync + 'static,
{
    match state
        .service
        .get_setting(&service_name, &key, query.env.as_deref())
        .await
    {
        Ok(setting) => (
            StatusCode::OK,
            Json(serde_json::to_value(SettingResponse {
                setting_id: setting.id,
                service_name: setting.service_name,
                env: setting.env,
                key: setting.key,
                value: setting.value,
                value_type: setting.value_type.as_str().to_string(),
                description: setting.description,
                is_active: setting.is_active,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "setting_not_found".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// 設定リスト取得
async fn list_settings<R, C>(
    State(state): State<RestState<R, C>>,
    Query(query): Query<ListSettingsQuery>,
) -> impl IntoResponse
where
    R: SettingRepository + Send + Sync + 'static,
    C: SettingCache + Send + Sync + 'static,
{
    let mut domain_query = DomainSettingQuery::new();
    if let Some(ref service_name) = query.service_name {
        domain_query = domain_query.with_service_name(service_name);
    }
    if let Some(ref key_prefix) = query.key_prefix {
        domain_query = domain_query.with_key_prefix(key_prefix);
    }
    if let Some(ref env) = query.env {
        domain_query = domain_query.with_env(env);
    }
    if let Some(page_size) = query.page_size {
        domain_query = domain_query.with_page_size(page_size);
    }
    if let Some(ref page_token) = query.page_token {
        domain_query = domain_query.with_page_token(page_token);
    }

    match state.service.list_settings(&domain_query).await {
        Ok(result) => {
            let settings: Vec<SettingResponse> = result
                .settings
                .into_iter()
                .map(|s| SettingResponse {
                    setting_id: s.id,
                    service_name: s.service_name,
                    env: s.env,
                    key: s.key,
                    value: s.value,
                    value_type: s.value_type.as_str().to_string(),
                    description: s.description,
                    is_active: s.is_active,
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::to_value(SettingListResponse {
                    settings,
                    next_page_token: result.next_page_token,
                })
                .unwrap()),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse {
                error: "list_settings_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// 設定更新
async fn update_setting<R, C>(
    State(state): State<RestState<R, C>>,
    Path((service_name, key)): Path<(String, String)>,
    Query(query): Query<GetSettingQuery>,
    Json(request): Json<UpdateSettingRequest>,
) -> impl IntoResponse
where
    R: SettingRepository + Send + Sync + 'static,
    C: SettingCache + Send + Sync + 'static,
{
    // 既存の設定を取得
    let setting = match state
        .service
        .get_setting(&service_name, &key, query.env.as_deref())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::to_value(ErrorResponse {
                    error: "setting_not_found".to_string(),
                    message: e.to_string(),
                })
                .unwrap()),
            );
        }
    };

    // 設定を更新
    let mut updated = setting.clone();
    updated.value = request.value;
    if let Some(desc) = request.description {
        updated.description = Some(desc);
    }
    updated.updated_at = std::time::SystemTime::now();

    match state.service.update_setting(&updated).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::to_value(SettingResponse {
                setting_id: updated.id,
                service_name: updated.service_name,
                env: updated.env,
                key: updated.key,
                value: updated.value,
                value_type: updated.value_type.as_str().to_string(),
                description: updated.description,
                is_active: updated.is_active,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse {
                error: "update_setting_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// 設定削除
async fn delete_setting<R, C>(
    State(state): State<RestState<R, C>>,
    Path((service_name, key)): Path<(String, String)>,
    Query(query): Query<GetSettingQuery>,
) -> impl IntoResponse
where
    R: SettingRepository + Send + Sync + 'static,
    C: SettingCache + Send + Sync + 'static,
{
    match state
        .service
        .delete_setting(&service_name, &key, query.env.as_deref())
        .await
    {
        Ok(()) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse {
                error: "delete_setting_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// 設定リフレッシュ
async fn refresh_setting<R, C>(
    State(state): State<RestState<R, C>>,
    Path((service_name, key)): Path<(String, String)>,
    Query(query): Query<GetSettingQuery>,
) -> impl IntoResponse
where
    R: SettingRepository + Send + Sync + 'static,
    C: SettingCache + Send + Sync + 'static,
{
    match state
        .service
        .refresh_setting(&service_name, &key, query.env.as_deref())
        .await
    {
        Ok(setting) => (
            StatusCode::OK,
            Json(serde_json::to_value(SettingResponse {
                setting_id: setting.id,
                service_name: setting.service_name,
                env: setting.env,
                key: setting.key,
                value: setting.value,
                value_type: setting.value_type.as_str().to_string(),
                description: setting.description,
                is_active: setting.is_active,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "setting_not_found".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}
