use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::domain::entity::api_registration::SchemaType;
use crate::usecase::{
    check_compatibility::{CheckCompatibilityError, CheckCompatibilityInput},
    delete_version::DeleteVersionError,
    get_diff::{GetDiffError, GetDiffInput},
    get_schema::GetSchemaError,
    get_schema_version::GetSchemaVersionError,
    list_schemas::{ListSchemasError, ListSchemasInput},
    list_versions::{ListVersionsError, ListVersionsInput},
    register_schema::{RegisterSchemaError, RegisterSchemaInput},
    register_version::{RegisterVersionError, RegisterVersionInput},
};

use super::{error::ApiError, AppState};

// JWT Claims から tenant_id を取得するヘルパー関数
// Claims が存在しない場合は "system" を返す
fn extract_tenant_id(claims: &Option<axum::extract::Extension<k1s0_auth::Claims>>) -> String {
    claims
        .as_ref().map_or_else(|| "system".to_string(), |c| c.0.tenant_id().to_string())
}

// Request types

#[derive(Debug, Deserialize)]
pub struct RegisterSchemaRequest {
    pub name: String,
    pub description: String,
    pub schema_type: String,
    pub content: String,
    pub registered_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterVersionRequest {
    pub content: String,
    pub registered_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckCompatibilityRequest {
    pub content: String,
    pub base_version: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListSchemasQuery {
    pub schema_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListVersionsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GetDiffQuery {
    pub from: Option<u32>,
    pub to: Option<u32>,
}

// Handler functions

/// GET /api/v1/schemas
/// JWT Claims から `tenant_id` を取得してテナントスコープで一覧を返す
pub async fn list_schemas(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Query(query): Query<ListSchemasQuery>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    let input = ListSchemasInput {
        tenant_id,
        schema_type: query.schema_type,
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20).min(100),
    };
    match state.list_schemas_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schemas": output.schemas.iter().map(|s| serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "schema_type": s.schema_type.to_string(),
                    "latest_version": s.latest_version,
                    "version_count": s.version_count,
                    "created_at": s.created_at,
                    "updated_at": s.updated_at,
                })).collect::<Vec<_>>(),
                "pagination": {
                    "total_count": output.total_count,
                    "page": output.page,
                    "page_size": output.page_size,
                    "has_next": output.has_next,
                },
            })),
        )
            .into_response(),
        Err(ListSchemasError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// POST /api/v1/schemas
/// JWT Claims から `tenant_id` を取得してテナントスコープでスキーマを登録する
pub async fn register_schema(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Json(body): Json<RegisterSchemaRequest>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    let input = RegisterSchemaInput {
        tenant_id,
        name: body.name,
        description: body.description,
        schema_type: SchemaType::from_str(&body.schema_type),
        content: body.content,
        registered_by: body
            .registered_by
            .unwrap_or_else(|| "anonymous".to_string()),
    };
    match state.register_schema_uc.execute(&input).await {
        Ok(version) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "name": version.name,
                "version": version.version,
                "schema_type": version.schema_type.to_string(),
                "content_hash": version.content_hash,
                "created_at": version.created_at,
            })),
        )
            .into_response(),
        Err(RegisterSchemaError::AlreadyExists(_)) => {
            ApiError::already_exists("Schema already exists").into_response()
        }
        Err(RegisterSchemaError::Validation(msg)) => {
            ApiError::unprocessable_entity(msg).into_response()
        }
        Err(RegisterSchemaError::ValidatorError(msg)) => {
            ApiError::validator_error(msg).into_response()
        }
        Err(RegisterSchemaError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// GET /api/v1/schemas/:name
/// JWT Claims から `tenant_id` を取得してテナントスコープでスキーマを取得する
pub async fn get_schema(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    match state.get_schema_uc.execute(&tenant_id, &name).await {
        Ok(output) => {
            let latest_content = output
                .latest_content
                .as_ref()
                .map_or("", |v| v.content.as_str());
            let latest_content_hash = output
                .latest_content
                .as_ref()
                .map_or("", |v| v.content_hash.as_str());
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "name": output.schema.name,
                    "description": output.schema.description,
                    "schema_type": output.schema.schema_type.to_string(),
                    "latest_version": output.schema.latest_version,
                    "version_count": output.schema.version_count,
                    "latest_content": latest_content,
                    "content_hash": latest_content_hash,
                    "created_at": output.schema.created_at,
                    "updated_at": output.schema.updated_at,
                })),
            )
                .into_response()
        }
        Err(GetSchemaError::NotFound(_)) => {
            ApiError::schema_not_found("Schema not found").into_response()
        }
        Err(GetSchemaError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// GET /api/v1/schemas/:name/versions
/// JWT Claims から `tenant_id` を取得してテナントスコープでバージョン一覧を取得する
pub async fn list_versions(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(name): Path<String>,
    Query(query): Query<ListVersionsQuery>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    let input = ListVersionsInput {
        tenant_id,
        name,
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20).min(100),
    };
    match state.list_versions_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "name": input.name,
                "versions": output.versions.iter().map(|v| serde_json::json!({
                    "version": v.version,
                    "content_hash": v.content_hash,
                    "breaking_changes": v.breaking_changes,
                    "breaking_change_details": v.breaking_change_details,
                    "registered_by": v.registered_by,
                    "created_at": v.created_at,
                })).collect::<Vec<_>>(),
                "pagination": {
                    "total_count": output.total_count,
                    "page": output.page,
                    "page_size": output.page_size,
                    "has_next": output.has_next,
                },
            })),
        )
            .into_response(),
        Err(ListVersionsError::NotFound(_)) => {
            ApiError::schema_not_found("Schema not found").into_response()
        }
        Err(ListVersionsError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// POST /api/v1/schemas/:name/versions
/// JWT Claims から `tenant_id` を取得してテナントスコープでバージョンを登録する
pub async fn register_version(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(name): Path<String>,
    Json(body): Json<RegisterVersionRequest>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    let input = RegisterVersionInput {
        tenant_id,
        name,
        content: body.content,
        registered_by: body
            .registered_by
            .unwrap_or_else(|| "anonymous".to_string()),
    };
    match state.register_version_uc.execute(&input).await {
        Ok(version) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "name": version.name,
                "version": version.version,
                "content_hash": version.content_hash,
                "breaking_changes": version.breaking_changes,
                "breaking_change_details": version.breaking_change_details,
                "created_at": version.created_at,
            })),
        )
            .into_response(),
        Err(RegisterVersionError::NotFound(_)) => {
            ApiError::schema_not_found("Schema not found").into_response()
        }
        Err(RegisterVersionError::Validation(msg)) => {
            ApiError::unprocessable_entity(msg).into_response()
        }
        Err(RegisterVersionError::ValidatorError(msg)) => {
            ApiError::validator_error(msg).into_response()
        }
        Err(RegisterVersionError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// GET /api/v1/schemas/:name/versions/:version
/// JWT Claims から `tenant_id` を取得してテナントスコープでスキーマバージョンを取得する
pub async fn get_schema_version(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path((name, version)): Path<(String, u32)>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    match state.get_schema_version_uc.execute(&tenant_id, &name, version).await {
        Ok(v) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "name": v.name,
                "version": v.version,
                "schema_type": v.schema_type.to_string(),
                "content": v.content,
                "content_hash": v.content_hash,
                "breaking_changes": v.breaking_changes,
                "breaking_change_details": v.breaking_change_details,
                "registered_by": v.registered_by,
                "created_at": v.created_at,
            })),
        )
            .into_response(),
        Err(GetSchemaVersionError::NotFound { .. }) => {
            ApiError::version_not_found("Schema version not found").into_response()
        }
        Err(GetSchemaVersionError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// DELETE /api/v1/schemas/:name/versions/:version
/// JWT Claims から `tenant_id` と `deleted_by` を取得してテナントスコープでバージョンを削除する
pub async fn delete_version(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, u32)>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id と削除者情報を取得する
    let tenant_id = extract_tenant_id(&claims);
    let deleted_by = claims.map(|c| c.0.sub);
    match state
        .delete_version_uc
        .execute(&tenant_id, &name, version, deleted_by)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteVersionError::SchemaNotFound(_)) => {
            ApiError::schema_not_found("Schema not found").into_response()
        }
        Err(DeleteVersionError::VersionNotFound { .. }) => {
            ApiError::version_not_found("Schema version not found").into_response()
        }
        Err(DeleteVersionError::CannotDeleteLatest(_)) => {
            ApiError::cannot_delete_latest("Cannot delete the only remaining version")
                .into_response()
        }
        Err(DeleteVersionError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// POST /api/v1/schemas/:name/compatibility
/// JWT Claims から `tenant_id` を取得してテナントスコープで互換性チェックを行う
pub async fn check_compatibility(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(name): Path<String>,
    Json(body): Json<CheckCompatibilityRequest>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    let input = CheckCompatibilityInput {
        tenant_id,
        name,
        content: body.content,
        base_version: body.base_version,
    };
    match state.check_compatibility_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "name": output.name,
                "base_version": output.base_version,
                "compatible": output.result.compatible,
                "breaking_changes": output.result.breaking_changes,
                "non_breaking_changes": output.result.non_breaking_changes,
            })),
        )
            .into_response(),
        Err(CheckCompatibilityError::SchemaNotFound(_)) => {
            ApiError::schema_not_found("Schema not found").into_response()
        }
        Err(CheckCompatibilityError::VersionNotFound { .. }) => {
            ApiError::version_not_found("Schema version not found").into_response()
        }
        Err(CheckCompatibilityError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}

/// GET /api/v1/schemas/:name/diff
/// JWT Claims から `tenant_id` を取得してテナントスコープで差分を取得する
pub async fn get_diff(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(name): Path<String>,
    Query(query): Query<GetDiffQuery>,
) -> impl IntoResponse {
    // JWT Claims から tenant_id を取得する
    let tenant_id = extract_tenant_id(&claims);
    let input = GetDiffInput {
        tenant_id,
        name,
        from_version: query.from,
        to_version: query.to,
    };
    match state.get_diff_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "name": output.name,
                "from_version": output.from_version,
                "to_version": output.to_version,
                "breaking_changes": output.breaking_changes,
                "diff": output.diff,
            })),
        )
            .into_response(),
        Err(GetDiffError::SchemaNotFound(_)) => {
            ApiError::schema_not_found("Schema not found").into_response()
        }
        Err(GetDiffError::VersionNotFound { .. }) => {
            ApiError::version_not_found("Schema version not found").into_response()
        }
        Err(GetDiffError::ValidationError(msg)) => ApiError::bad_request(msg).into_response(),
        Err(GetDiffError::Internal(msg)) => ApiError::internal(msg).into_response(),
    }
}
