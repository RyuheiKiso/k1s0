use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;

use super::AppState;
use crate::domain::entity::config_schema::ConfigSchema;

/// PUT /api/v1/config-schema/:service_name のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertConfigSchemaRequest {
    pub namespace_prefix: String,
    pub schema_json: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/api/v1/config-schema/{service_name}",
    params(("service_name" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "Config schema found", body = ConfigSchema),
        (status = 404, description = "Schema not found"),
    )
)]
pub async fn get_config_schema(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> impl IntoResponse {
    match state.get_config_schema_uc.execute(&service_name).await {
        Ok(schema) => (StatusCode::OK, Json(serde_json::to_value(schema).unwrap())).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/config-schema/{service_name}",
    params(("service_name" = String, Path, description = "Service name")),
    request_body = UpsertConfigSchemaRequest,
    responses(
        (status = 200, description = "Config schema upserted", body = ConfigSchema),
        (status = 500, description = "Internal error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn upsert_config_schema(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(service_name): Path<String>,
    Json(req): Json<UpsertConfigSchemaRequest>,
) -> impl IntoResponse {
    let updated_by = claims
        .and_then(|Extension(c)| c.preferred_username.clone())
        .unwrap_or_else(|| "api-user".to_string());

    let input = crate::usecase::upsert_config_schema::UpsertConfigSchemaInput {
        service_name,
        namespace_prefix: req.namespace_prefix,
        schema_json: req.schema_json,
        updated_by,
    };

    match state.upsert_config_schema_uc.execute(&input).await {
        Ok(schema) => (StatusCode::OK, Json(serde_json::to_value(schema).unwrap())).into_response(),
        Err(e) => e.into_response(),
    }
}
