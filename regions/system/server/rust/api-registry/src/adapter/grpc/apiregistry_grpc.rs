use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::entity::api_registration::{
    ApiSchema, ApiSchemaVersion, ChangeDetail, DiffEntry, DiffModifiedEntry, SchemaDiff, SchemaType,
};
use crate::usecase::check_compatibility::{
    CheckCompatibilityError, CheckCompatibilityInput, CheckCompatibilityUseCase,
};
use crate::usecase::delete_version::{DeleteVersionError, DeleteVersionUseCase};
use crate::usecase::get_diff::{GetDiffError, GetDiffInput, GetDiffUseCase};
use crate::usecase::get_schema::{GetSchemaError, GetSchemaUseCase};
use crate::usecase::get_schema_version::{GetSchemaVersionError, GetSchemaVersionUseCase};
use crate::usecase::list_schemas::{ListSchemasError, ListSchemasInput, ListSchemasUseCase};
use crate::usecase::list_versions::{ListVersionsError, ListVersionsInput, ListVersionsUseCase};
use crate::usecase::register_schema::{RegisterSchemaError, RegisterSchemaInput, RegisterSchemaUseCase};
use crate::usecase::register_version::{RegisterVersionError, RegisterVersionInput, RegisterVersionUseCase};

#[derive(Debug, Clone)]
pub struct ApiSchemaData {
    pub name: String,
    pub description: String,
    pub schema_type: String,
    pub latest_version: u32,
    pub version_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ChangeDetailData {
    pub change_type: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ApiSchemaVersionData {
    pub name: String,
    pub version: u32,
    pub schema_type: String,
    pub content: String,
    pub content_hash: String,
    pub breaking_changes: bool,
    pub breaking_change_details: Vec<ChangeDetailData>,
    pub registered_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DiffEntryData {
    pub path: String,
    pub entry_type: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DiffModifiedEntryData {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone)]
pub struct SchemaDiffData {
    pub added: Vec<DiffEntryData>,
    pub modified: Vec<DiffModifiedEntryData>,
    pub removed: Vec<DiffEntryData>,
}

#[derive(Debug, Clone)]
pub struct ListSchemasRequest {
    pub schema_type: String,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListSchemasResponse {
    pub schemas: Vec<ApiSchemaData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterSchemaRequest {
    pub name: String,
    pub description: String,
    pub schema_type: String,
    pub content: String,
    pub registered_by: String,
}

#[derive(Debug, Clone)]
pub struct RegisterSchemaResponse {
    pub version: ApiSchemaVersionData,
}

#[derive(Debug, Clone)]
pub struct GetSchemaRequest {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct GetSchemaResponse {
    pub schema: ApiSchemaData,
    pub latest_content: String,
}

#[derive(Debug, Clone)]
pub struct ListVersionsRequest {
    pub name: String,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct ListVersionsResponse {
    pub name: String,
    pub versions: Vec<ApiSchemaVersionData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterVersionRequest {
    pub name: String,
    pub content: String,
    pub registered_by: String,
}

#[derive(Debug, Clone)]
pub struct RegisterVersionResponse {
    pub version: ApiSchemaVersionData,
}

#[derive(Debug, Clone)]
pub struct GetSchemaVersionRequest {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct GetSchemaVersionResponse {
    pub version: ApiSchemaVersionData,
}

#[derive(Debug, Clone)]
pub struct DeleteVersionRequest {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct DeleteVersionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CheckCompatibilityRequest {
    pub name: String,
    pub content: String,
    pub base_version: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CheckCompatibilityResponse {
    pub compatible: bool,
    pub base_version: u32,
    pub breaking_changes: Vec<ChangeDetailData>,
    pub non_breaking_changes: Vec<ChangeDetailData>,
}

#[derive(Debug, Clone)]
pub struct GetDiffRequest {
    pub name: String,
    pub from_version: Option<u32>,
    pub to_version: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GetDiffResponse {
    pub name: String,
    pub from_version: u32,
    pub to_version: u32,
    pub breaking_changes: bool,
    pub diff: SchemaDiffData,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("failed precondition: {0}")]
    FailedPrecondition(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ApiRegistryGrpcService {
    list_schemas_uc: Arc<ListSchemasUseCase>,
    register_schema_uc: Arc<RegisterSchemaUseCase>,
    get_schema_uc: Arc<GetSchemaUseCase>,
    list_versions_uc: Arc<ListVersionsUseCase>,
    register_version_uc: Arc<RegisterVersionUseCase>,
    get_schema_version_uc: Arc<GetSchemaVersionUseCase>,
    delete_version_uc: Arc<DeleteVersionUseCase>,
    check_compatibility_uc: Arc<CheckCompatibilityUseCase>,
    get_diff_uc: Arc<GetDiffUseCase>,
}

impl ApiRegistryGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        list_schemas_uc: Arc<ListSchemasUseCase>,
        register_schema_uc: Arc<RegisterSchemaUseCase>,
        get_schema_uc: Arc<GetSchemaUseCase>,
        list_versions_uc: Arc<ListVersionsUseCase>,
        register_version_uc: Arc<RegisterVersionUseCase>,
        get_schema_version_uc: Arc<GetSchemaVersionUseCase>,
        delete_version_uc: Arc<DeleteVersionUseCase>,
        check_compatibility_uc: Arc<CheckCompatibilityUseCase>,
        get_diff_uc: Arc<GetDiffUseCase>,
    ) -> Self {
        Self {
            list_schemas_uc,
            register_schema_uc,
            get_schema_uc,
            list_versions_uc,
            register_version_uc,
            get_schema_version_uc,
            delete_version_uc,
            check_compatibility_uc,
            get_diff_uc,
        }
    }

    pub async fn list_schemas(
        &self,
        request: ListSchemasRequest,
    ) -> Result<ListSchemasResponse, GrpcError> {
        let page = if request.page <= 0 { 1 } else { request.page as u32 };
        let page_size = if request.page_size <= 0 {
            20
        } else {
            request.page_size as u32
        };

        let output = self
            .list_schemas_uc
            .execute(&ListSchemasInput {
                schema_type: if request.schema_type.is_empty() {
                    None
                } else {
                    Some(request.schema_type)
                },
                page,
                page_size,
            })
            .await
            .map_err(|e| match e {
                ListSchemasError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListSchemasResponse {
            schemas: output.schemas.into_iter().map(to_schema_data).collect(),
            total_count: output.total_count,
            page: output.page as i32,
            page_size: output.page_size as i32,
            has_next: output.has_next,
        })
    }

    pub async fn register_schema(
        &self,
        request: RegisterSchemaRequest,
    ) -> Result<RegisterSchemaResponse, GrpcError> {
        let version = self
            .register_schema_uc
            .execute(&RegisterSchemaInput {
                name: request.name,
                description: request.description,
                schema_type: SchemaType::from_str(&request.schema_type),
                content: request.content,
                registered_by: request.registered_by,
            })
            .await
            .map_err(|e| match e {
                RegisterSchemaError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("schema already exists: {}", name))
                }
                RegisterSchemaError::Validation(msg) => GrpcError::InvalidArgument(msg),
                RegisterSchemaError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(RegisterSchemaResponse {
            version: to_schema_version_data(version),
        })
    }

    pub async fn get_schema(
        &self,
        request: GetSchemaRequest,
    ) -> Result<GetSchemaResponse, GrpcError> {
        if request.name.is_empty() {
            return Err(GrpcError::InvalidArgument("name is required".to_string()));
        }
        let output = self.get_schema_uc.execute(&request.name).await.map_err(|e| match e {
            GetSchemaError::NotFound(n) => GrpcError::NotFound(format!("schema not found: {}", n)),
            GetSchemaError::Internal(msg) => GrpcError::Internal(msg),
        })?;
        Ok(GetSchemaResponse {
            schema: to_schema_data(output.schema),
            latest_content: output.latest_content.map(|v| v.content).unwrap_or_default(),
        })
    }

    pub async fn list_versions(
        &self,
        request: ListVersionsRequest,
    ) -> Result<ListVersionsResponse, GrpcError> {
        let page = if request.page <= 0 { 1 } else { request.page as u32 };
        let page_size = if request.page_size <= 0 {
            20
        } else {
            request.page_size as u32
        };
        let output = self
            .list_versions_uc
            .execute(&ListVersionsInput {
                name: request.name,
                page,
                page_size,
            })
            .await
            .map_err(|e| match e {
                ListVersionsError::NotFound(name) => {
                    GrpcError::NotFound(format!("schema not found: {}", name))
                }
                ListVersionsError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListVersionsResponse {
            name: output.name,
            versions: output.versions.into_iter().map(to_schema_version_data).collect(),
            total_count: output.total_count,
            page: output.page as i32,
            page_size: output.page_size as i32,
            has_next: output.has_next,
        })
    }

    pub async fn register_version(
        &self,
        request: RegisterVersionRequest,
    ) -> Result<RegisterVersionResponse, GrpcError> {
        let version = self
            .register_version_uc
            .execute(&RegisterVersionInput {
                name: request.name,
                content: request.content,
                registered_by: request.registered_by,
            })
            .await
            .map_err(|e| match e {
                RegisterVersionError::NotFound(name) => {
                    GrpcError::NotFound(format!("schema not found: {}", name))
                }
                RegisterVersionError::Validation(msg) => GrpcError::InvalidArgument(msg),
                RegisterVersionError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(RegisterVersionResponse {
            version: to_schema_version_data(version),
        })
    }

    pub async fn get_schema_version(
        &self,
        request: GetSchemaVersionRequest,
    ) -> Result<GetSchemaVersionResponse, GrpcError> {
        if request.name.is_empty() {
            return Err(GrpcError::InvalidArgument("name is required".to_string()));
        }
        let output = self
            .get_schema_version_uc
            .execute(&request.name, request.version)
            .await
            .map_err(|e| match e {
                GetSchemaVersionError::NotFound { name, version } => {
                    GrpcError::NotFound(format!("{}@{} not found", name, version))
                }
                GetSchemaVersionError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(GetSchemaVersionResponse {
            version: to_schema_version_data(output),
        })
    }

    pub async fn delete_version(
        &self,
        request: DeleteVersionRequest,
    ) -> Result<DeleteVersionResponse, GrpcError> {
        self.delete_version_uc
            .execute(&request.name, request.version)
            .await
            .map_err(|e| match e {
                DeleteVersionError::SchemaNotFound(name) => {
                    GrpcError::NotFound(format!("schema not found: {}", name))
                }
                DeleteVersionError::VersionNotFound { name, version } => {
                    GrpcError::NotFound(format!("{}@{} not found", name, version))
                }
                DeleteVersionError::CannotDeleteLatest(name) => {
                    GrpcError::FailedPrecondition(format!(
                        "cannot delete the only remaining version of schema: {}",
                        name
                    ))
                }
                DeleteVersionError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(DeleteVersionResponse {
            success: true,
            message: format!("deleted schema version {}@{}", request.name, request.version),
        })
    }

    pub async fn check_compatibility(
        &self,
        request: CheckCompatibilityRequest,
    ) -> Result<CheckCompatibilityResponse, GrpcError> {
        if request.name.is_empty() {
            return Err(GrpcError::InvalidArgument("name is required".to_string()));
        }
        if request.content.is_empty() {
            return Err(GrpcError::InvalidArgument("content is required".to_string()));
        }
        let input = CheckCompatibilityInput {
            name: request.name,
            content: request.content,
            base_version: request.base_version,
        };
        let output = self
            .check_compatibility_uc
            .execute(&input)
            .await
            .map_err(|e| match e {
                CheckCompatibilityError::SchemaNotFound(n) => {
                    GrpcError::NotFound(format!("schema not found: {}", n))
                }
                CheckCompatibilityError::VersionNotFound { name, version } => {
                    GrpcError::NotFound(format!("{}@{} not found", name, version))
                }
                CheckCompatibilityError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CheckCompatibilityResponse {
            compatible: output.result.compatible,
            base_version: output.base_version,
            breaking_changes: output
                .result
                .breaking_changes
                .into_iter()
                .map(|c| ChangeDetailData {
                    change_type: c.change_type,
                    path: c.path,
                    description: c.description,
                })
                .collect(),
            non_breaking_changes: output
                .result
                .non_breaking_changes
                .into_iter()
                .map(to_change_detail_data)
                .collect(),
        })
    }

    pub async fn get_diff(&self, request: GetDiffRequest) -> Result<GetDiffResponse, GrpcError> {
        let output = self
            .get_diff_uc
            .execute(&GetDiffInput {
                name: request.name,
                from_version: request.from_version,
                to_version: request.to_version,
            })
            .await
            .map_err(|e| match e {
                GetDiffError::SchemaNotFound(name) => {
                    GrpcError::NotFound(format!("schema not found: {}", name))
                }
                GetDiffError::VersionNotFound { name, version } => {
                    GrpcError::NotFound(format!("{}@{} not found", name, version))
                }
                GetDiffError::ValidationError(msg) => GrpcError::InvalidArgument(msg),
                GetDiffError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(GetDiffResponse {
            name: output.name,
            from_version: output.from_version,
            to_version: output.to_version,
            breaking_changes: output.breaking_changes,
            diff: to_schema_diff_data(output.diff),
        })
    }
}

fn to_schema_data(schema: ApiSchema) -> ApiSchemaData {
    ApiSchemaData {
        name: schema.name,
        description: schema.description,
        schema_type: schema.schema_type.to_string(),
        latest_version: schema.latest_version,
        version_count: schema.version_count,
        created_at: schema.created_at,
        updated_at: schema.updated_at,
    }
}

fn to_schema_version_data(version: ApiSchemaVersion) -> ApiSchemaVersionData {
    ApiSchemaVersionData {
        name: version.name,
        version: version.version,
        schema_type: version.schema_type.to_string(),
        content: version.content,
        content_hash: version.content_hash,
        breaking_changes: version.breaking_changes,
        breaking_change_details: version
            .breaking_change_details
            .into_iter()
            .map(|c| ChangeDetailData {
                change_type: c.change_type,
                path: c.path,
                description: c.description,
            })
            .collect(),
        registered_by: version.registered_by,
        created_at: version.created_at,
    }
}

fn to_change_detail_data(detail: ChangeDetail) -> ChangeDetailData {
    ChangeDetailData {
        change_type: detail.change_type,
        path: detail.path,
        description: detail.description,
    }
}

fn to_schema_diff_data(diff: SchemaDiff) -> SchemaDiffData {
    SchemaDiffData {
        added: diff.added.into_iter().map(to_diff_entry_data).collect(),
        modified: diff.modified.into_iter().map(to_diff_modified_entry_data).collect(),
        removed: diff.removed.into_iter().map(to_diff_entry_data).collect(),
    }
}

fn to_diff_entry_data(entry: DiffEntry) -> DiffEntryData {
    DiffEntryData {
        path: entry.path,
        entry_type: entry.entry_type,
        description: entry.description,
    }
}

fn to_diff_modified_entry_data(entry: DiffModifiedEntry) -> DiffModifiedEntryData {
    DiffModifiedEntryData {
        path: entry.path,
        before: entry.before,
        after: entry.after,
    }
}
