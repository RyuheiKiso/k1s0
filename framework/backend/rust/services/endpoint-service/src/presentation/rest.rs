//! REST API実装
//!
//! axumを使用したREST APIハンドラーを提供する。

use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::application::EndpointService;
use crate::domain::{EndpointQuery as DomainEndpointQuery, EndpointRepository};

/// REST APIの状態
pub struct RestState<R>
where
    R: EndpointRepository + 'static,
{
    pub service: Arc<EndpointService<R>>,
}

impl<R> Clone for RestState<R>
where
    R: EndpointRepository + 'static,
{
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
        }
    }
}

/// エンドポイントレスポンス
#[derive(Debug, Serialize)]
pub struct EndpointResponse {
    pub endpoint_id: i64,
    pub service_name: String,
    pub path: String,
    pub method: String,
    pub description: Option<String>,
    pub is_active: bool,
}

/// エンドポイントリストレスポンス
#[derive(Debug, Serialize)]
pub struct EndpointListResponse {
    pub endpoints: Vec<EndpointResponse>,
    pub next_page_token: Option<String>,
}

/// 解決アドレスレスポンス
#[derive(Debug, Serialize)]
pub struct ResolvedAddressResponse {
    pub address: String,
    pub use_tls: bool,
}

/// エンドポイント取得クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct GetEndpointQuery {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

/// エンドポイントリストクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ListEndpointsQuery {
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path_prefix: Option<String>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub page_token: Option<String>,
}

/// アドレス解決クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ResolveQuery {
    pub protocol: String,
}

/// エラーレスポンス
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// REST APIルーターを作成
pub fn create_router<R>(service: Arc<EndpointService<R>>) -> Router
where
    R: EndpointRepository + Send + Sync + 'static,
{
    let state = RestState { service };

    Router::new()
        .route("/v1/endpoints", get(list_endpoints))
        .route("/v1/endpoints/:service_name", get(get_endpoint))
        .route("/v1/resolve/:service_name", get(resolve_endpoint))
        .route("/health", get(health_check))
        .with_state(state)
}

/// ヘルスチェック
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// エンドポイント取得
async fn get_endpoint<R>(
    State(state): State<RestState<R>>,
    Path(service_name): Path<String>,
    Query(query): Query<GetEndpointQuery>,
) -> impl IntoResponse
where
    R: EndpointRepository + Send + Sync + 'static,
{
    match state
        .service
        .get_endpoint(&service_name, query.method.as_deref(), query.path.as_deref())
        .await
    {
        Ok(endpoint) => (
            StatusCode::OK,
            Json(serde_json::to_value(EndpointResponse {
                endpoint_id: endpoint.id,
                service_name: endpoint.service_name,
                path: endpoint.path,
                method: endpoint.method,
                description: endpoint.description,
                is_active: endpoint.is_active,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "endpoint_not_found".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// エンドポイントリスト取得
async fn list_endpoints<R>(
    State(state): State<RestState<R>>,
    Query(query): Query<ListEndpointsQuery>,
) -> impl IntoResponse
where
    R: EndpointRepository + Send + Sync + 'static,
{
    let mut domain_query = DomainEndpointQuery::new();
    if let Some(ref service_name) = query.service_name {
        domain_query = domain_query.with_service_name(service_name);
    }
    if let Some(ref method) = query.method {
        domain_query = domain_query.with_method(method);
    }
    if let Some(ref path_prefix) = query.path_prefix {
        domain_query = domain_query.with_path_prefix(path_prefix);
    }
    if let Some(page_size) = query.page_size {
        domain_query = domain_query.with_page_size(page_size);
    }
    if let Some(ref page_token) = query.page_token {
        domain_query = domain_query.with_page_token(page_token);
    }

    match state.service.list_endpoints(&domain_query).await {
        Ok(result) => {
            let endpoints: Vec<EndpointResponse> = result
                .endpoints
                .into_iter()
                .map(|e| EndpointResponse {
                    endpoint_id: e.id,
                    service_name: e.service_name,
                    path: e.path,
                    method: e.method,
                    description: e.description,
                    is_active: e.is_active,
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::to_value(EndpointListResponse {
                    endpoints,
                    next_page_token: result.next_page_token,
                })
                .unwrap()),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse {
                error: "list_endpoints_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// アドレス解決
async fn resolve_endpoint<R>(
    State(state): State<RestState<R>>,
    Path(service_name): Path<String>,
    Query(query): Query<ResolveQuery>,
) -> impl IntoResponse
where
    R: EndpointRepository + Send + Sync + 'static,
{
    match state
        .service
        .resolve_endpoint(&service_name, &query.protocol)
        .await
    {
        Ok(resolved) => (
            StatusCode::OK,
            Json(serde_json::to_value(ResolvedAddressResponse {
                address: resolved.address,
                use_tls: resolved.use_tls,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "resolve_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}
