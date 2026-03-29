use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{AppState, ErrorResponse};
// utoipa マクロの body 型参照に必要なインポート
use crate::domain::entity::service::Service;
use crate::domain::entity::service::ServiceTier;

/// サービス検索のクエリパラメータ。
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub tags: Option<String>,
    pub tier: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/services/search",
    params(
        ("q" = Option<String>, Query, description = "Search query string"),
        ("tags" = Option<String>, Query, description = "Comma-separated tags"),
        ("tier" = Option<String>, Query, description = "Filter by tier"),
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<Service>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn search_services(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let tags = params
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let tier = params.tier.and_then(|t| t.parse::<ServiceTier>().ok());

    match state.search_services_uc.execute(params.q, tags, tier).await {
        Ok(services) => (StatusCode::OK, Json(services)).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
