use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use super::{AppState, ErrorDetail, ErrorResponse};
use crate::adapter::presentation::{
    ConfigEditorSchemaDto, ConfigFieldType, UpsertConfigSchemaRequestDto,
};

#[utoipa::path(
    get,
    path = "/api/v1/config-schema",
    responses(
        (status = 200, description = "All config schemas", body = Vec<ConfigEditorSchemaDto>),
    )
)]
pub async fn list_config_schemas(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_config_schemas_uc.execute().await {
        Ok(schemas) => match schemas
            .iter()
            .map(ConfigEditorSchemaDto::try_from)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "SYS_CONFIG_SCHEMA_INVALID",
                    format!("invalid persisted config schema: {}", err),
                )),
            )
                .into_response(),
        },
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/config-schema/{service_name}",
    params(("service_name" = String, Path, description = "Service name")),
    responses(
        (status = 200, description = "Config schema found", body = ConfigEditorSchemaDto),
        (status = 404, description = "Schema not found"),
    )
)]
pub async fn get_config_schema(
    State(state): State<AppState>,
    Path(service_name): Path<String>,
) -> impl IntoResponse {
    match state.get_config_schema_uc.execute(&service_name).await {
        Ok(schema) => match ConfigEditorSchemaDto::try_from(&schema) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "SYS_CONFIG_SCHEMA_INVALID",
                    format!("invalid persisted config schema: {}", err),
                )),
            )
                .into_response(),
        },
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/config-schema/{service_name}",
    params(("service_name" = String, Path, description = "Service name")),
    request_body = UpsertConfigSchemaRequestDto,
    responses(
        (status = 200, description = "Config schema upserted", body = ConfigEditorSchemaDto),
        (status = 500, description = "Internal error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn upsert_config_schema(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(service_name): Path<String>,
    Json(req): Json<UpsertConfigSchemaRequestDto>,
) -> impl IntoResponse {
    let details = validate_upsert_request(&req);
    if !details.is_empty() {
        let err = ErrorResponse::with_details(
            "SYS_CONFIG_VALIDATION_ERROR",
            "invalid config schema request",
            details,
        );
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    }

    let updated_by = claims
        .and_then(|Extension(c)| c.preferred_username.clone())
        .unwrap_or_else(|| "api-user".to_string());

    let schema_json = req.clone().into_schema_json();
    let input = crate::usecase::upsert_config_schema::UpsertConfigSchemaInput {
        service_name,
        namespace_prefix: req.namespace_prefix,
        schema_json,
        updated_by,
    };

    match state.upsert_config_schema_uc.execute(&input).await {
        Ok(schema) => match ConfigEditorSchemaDto::try_from(&schema) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "SYS_CONFIG_SCHEMA_INVALID",
                    format!("invalid persisted config schema: {}", err),
                )),
            )
                .into_response(),
        },
        Err(e) => e.into_response(),
    }
}

/// スキーマ upsert リクエストのバリデーションを実行し、ErrorDetail のリストを返す。
fn validate_upsert_request(req: &UpsertConfigSchemaRequestDto) -> Vec<ErrorDetail> {
    let mut details = Vec::new();

    // namespace_prefix が空でないことを検証する
    if req.namespace_prefix.trim().is_empty() {
        details.push(ErrorDetail::new(
            "namespace_prefix",
            "required",
            "namespace_prefix is required",
        ));
    }

    // categories が空でないことを検証する
    if req.categories.is_empty() {
        details.push(ErrorDetail::new(
            "categories",
            "required",
            "at least one category is required",
        ));
    }

    for (cat_idx, category) in req.categories.iter().enumerate() {
        // カテゴリ ID が空でないことを検証する
        if category.id.trim().is_empty() {
            details.push(ErrorDetail::new(
                format!("categories[{}].id", cat_idx),
                "required",
                "category id is required",
            ));
        }
        // カテゴリ label が空でないことを検証する
        if category.label.trim().is_empty() {
            details.push(ErrorDetail::new(
                format!("categories[{}].label", cat_idx),
                "required",
                "category label is required",
            ));
        }
        // namespaces が空でないことを検証する
        if category.namespaces.is_empty() {
            details.push(ErrorDetail::new(
                format!("categories[{}].namespaces", cat_idx),
                "required",
                "at least one namespace is required",
            ));
        }
        for (ns_idx, namespace) in category.namespaces.iter().enumerate() {
            // 各 namespace が空でないことを検証する
            if namespace.trim().is_empty() {
                details.push(ErrorDetail::new(
                    format!("categories[{}].namespaces[{}]", cat_idx, ns_idx),
                    "required",
                    "namespace must not be empty",
                ));
            }
        }
        for (field_idx, field) in category.fields.iter().enumerate() {
            // フィールド key が空でないことを検証する
            if field.key.trim().is_empty() {
                details.push(ErrorDetail::new(
                    format!("categories[{}].fields[{}].key", cat_idx, field_idx),
                    "required",
                    "field key is required",
                ));
            }
            // フィールド label が空でないことを検証する
            if field.label.trim().is_empty() {
                details.push(ErrorDetail::new(
                    format!("categories[{}].fields[{}].label", cat_idx, field_idx),
                    "required",
                    "field label is required",
                ));
            }
            // Enum フィールドには options が必要であることを検証する
            if field.field_type == ConfigFieldType::Enum && field.options.is_empty() {
                details.push(ErrorDetail::new(
                    format!("categories[{}].fields[{}].options", cat_idx, field_idx),
                    "required",
                    "enum field requires at least one option",
                ));
            }
        }
    }

    details
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::adapter::handler::{router, AppState};
    use crate::domain::entity::config_schema::ConfigSchema;
    use crate::domain::repository::config_repository::MockConfigRepository;
    use crate::domain::repository::config_schema_repository::MockConfigSchemaRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_schema() -> ConfigSchema {
        ConfigSchema {
            id: Uuid::new_v4(),
            service_name: "task-server".to_string(),
            namespace_prefix: "service.order".to_string(),
            schema_json: serde_json::json!({
                "categories": [{
                    "id": "database",
                    "label": "Database",
                    "namespaces": ["service.order.database"],
                    "fields": [{
                        "key": "timeout",
                        "label": "Timeout",
                        "type": 2,
                        "default_value": 30
                    }]
                }]
            }),
            updated_by: "tester".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn get_config_schema_returns_rest_dto_shape() {
        let config_repo = Arc::new(MockConfigRepository::new());
        let mut schema_repo = MockConfigSchemaRepository::new();
        let schema = make_schema();
        schema_repo
            .expect_find_by_service_name()
            .withf(|service| service == "task-server")
            .return_once(move |_| Ok(Some(schema)));

        let app = router(AppState::new(config_repo, Arc::new(schema_repo)));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/config-schema/task-server")
                    .body(Body::empty())
                    .expect("request build should succeed"),
            )
            .await
            .expect("test operation should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("test operation should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("test operation should succeed");
        assert_eq!(json["service"], "task-server");
        assert_eq!(json["categories"][0]["fields"][0]["type"], "integer");
        assert_eq!(json["categories"][0]["fields"][0]["default"], 30);
        assert!(json.get("schema_json").is_none());
        assert!(json.get("updated_by").is_none());
    }

    #[tokio::test]
    async fn upsert_config_schema_accepts_legacy_numeric_type_and_normalizes_response() {
        let config_repo = Arc::new(MockConfigRepository::new());
        let mut schema_repo = MockConfigSchemaRepository::new();
        schema_repo
            .expect_upsert()
            .returning(|schema| Ok(schema.clone()));

        let app = router(AppState::new(config_repo, Arc::new(schema_repo)));
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/config-schema/task-server")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "namespace_prefix": "service.order",
                            "categories": [{
                                "id": "database",
                                "label": "Database",
                                "namespaces": ["service.order.database"],
                                "fields": [{
                                    "key": "timeout",
                                    "label": "Timeout",
                                    "type": 2,
                                    "default_value": 30
                                }]
                            }]
                        })
                        .to_string(),
                    ))
                    .expect("request build should succeed"),
            )
            .await
            .expect("test operation should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("test operation should succeed");
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("test operation should succeed");
        assert_eq!(json["service"], "task-server");
        assert_eq!(json["categories"][0]["fields"][0]["type"], "integer");
        assert_eq!(json["categories"][0]["fields"][0]["default"], 30);
    }
}
