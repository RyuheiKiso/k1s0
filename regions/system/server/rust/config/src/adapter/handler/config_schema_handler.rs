use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;

use super::{AppState, ErrorDetail, ErrorResponse};
use crate::domain::entity::config_schema::ConfigSchema;

/// PUT /api/v1/config-schema/:service_name のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertConfigSchemaRequest {
    pub namespace_prefix: String,
    pub categories: Vec<ConfigCategorySchemaRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ConfigCategorySchemaRequest {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub namespaces: Vec<String>,
    #[serde(default)]
    pub fields: Vec<ConfigFieldSchemaRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ConfigFieldSchemaRequest {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub field_type: i32,
    #[serde(default)]
    pub min: i64,
    #[serde(default)]
    pub max: i64,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub default_value: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/api/v1/config-schema",
    responses(
        (status = 200, description = "All config schemas", body = Vec<ConfigSchema>),
    )
)]
pub async fn list_config_schemas(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_config_schemas_uc.execute().await {
        Ok(schemas) => {
            (StatusCode::OK, Json(serde_json::to_value(schemas).unwrap())).into_response()
        }
        Err(e) => e.into_response(),
    }
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
    let details = validate_upsert_request(&req);
    if !details.is_empty() {
        let err = ErrorResponse::with_details(
            "SYS_CONFIG_VALIDATION_ERROR",
            "invalid config schema request",
            details,
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(err).unwrap()),
        )
            .into_response();
    }

    let updated_by = claims
        .and_then(|Extension(c)| c.preferred_username.clone())
        .unwrap_or_else(|| "api-user".to_string());

    let schema_json = request_to_schema_json(&req);
    let input = crate::usecase::upsert_config_schema::UpsertConfigSchemaInput {
        service_name,
        namespace_prefix: req.namespace_prefix,
        schema_json,
        updated_by,
    };

    match state.upsert_config_schema_uc.execute(&input).await {
        Ok(schema) => (StatusCode::OK, Json(serde_json::to_value(schema).unwrap())).into_response(),
        Err(e) => e.into_response(),
    }
}

fn validate_upsert_request(req: &UpsertConfigSchemaRequest) -> Vec<ErrorDetail> {
    let mut details = Vec::new();

    if req.namespace_prefix.trim().is_empty() {
        details.push(ErrorDetail {
            field: "namespace_prefix".to_string(),
            message: "namespace_prefix is required".to_string(),
        });
    }

    if req.categories.is_empty() {
        details.push(ErrorDetail {
            field: "categories".to_string(),
            message: "at least one category is required".to_string(),
        });
    }

    for (cat_idx, category) in req.categories.iter().enumerate() {
        if category.id.trim().is_empty() {
            details.push(ErrorDetail {
                field: format!("categories[{}].id", cat_idx),
                message: "category id is required".to_string(),
            });
        }
        if category.label.trim().is_empty() {
            details.push(ErrorDetail {
                field: format!("categories[{}].label", cat_idx),
                message: "category label is required".to_string(),
            });
        }
        if category.namespaces.is_empty() {
            details.push(ErrorDetail {
                field: format!("categories[{}].namespaces", cat_idx),
                message: "at least one namespace is required".to_string(),
            });
        }
        for (ns_idx, namespace) in category.namespaces.iter().enumerate() {
            if namespace.trim().is_empty() {
                details.push(ErrorDetail {
                    field: format!("categories[{}].namespaces[{}]", cat_idx, ns_idx),
                    message: "namespace must not be empty".to_string(),
                });
            }
        }
        for (field_idx, field) in category.fields.iter().enumerate() {
            if field.key.trim().is_empty() {
                details.push(ErrorDetail {
                    field: format!("categories[{}].fields[{}].key", cat_idx, field_idx),
                    message: "field key is required".to_string(),
                });
            }
            if field.label.trim().is_empty() {
                details.push(ErrorDetail {
                    field: format!("categories[{}].fields[{}].label", cat_idx, field_idx),
                    message: "field label is required".to_string(),
                });
            }
            if !(1..=7).contains(&field.field_type) {
                details.push(ErrorDetail {
                    field: format!("categories[{}].fields[{}].type", cat_idx, field_idx),
                    message: "field type must be in range 1..=7".to_string(),
                });
            }
            if field.field_type == 5 && field.options.is_empty() {
                details.push(ErrorDetail {
                    field: format!("categories[{}].fields[{}].options", cat_idx, field_idx),
                    message: "enum field requires at least one option".to_string(),
                });
            }
        }
    }

    details
}

fn request_to_schema_json(req: &UpsertConfigSchemaRequest) -> serde_json::Value {
    let categories: Vec<serde_json::Value> = req
        .categories
        .iter()
        .map(|category| {
            let fields: Vec<serde_json::Value> = category
                .fields
                .iter()
                .map(|field| {
                    serde_json::json!({
                        "key": field.key,
                        "label": field.label,
                        "description": field.description,
                        "type": field.field_type,
                        "min": field.min,
                        "max": field.max,
                        "options": field.options,
                        "pattern": field.pattern,
                        "unit": field.unit,
                        "default_value": field.default_value,
                    })
                })
                .collect();

            serde_json::json!({
                "id": category.id,
                "label": category.label,
                "icon": category.icon,
                "namespaces": category.namespaces,
                "fields": fields,
            })
        })
        .collect();

    serde_json::json!({ "categories": categories })
}
