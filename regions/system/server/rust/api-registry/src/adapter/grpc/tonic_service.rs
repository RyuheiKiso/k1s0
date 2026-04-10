use std::sync::Arc;

use chrono::{DateTime, Utc};
use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::apiregistry::v1::{
    api_registry_service_server::ApiRegistryService, ApiSchemaProto, ApiSchemaVersionProto,
    CheckCompatibilityRequest as ProtoCheckCompatibilityRequest,
    CheckCompatibilityResponse as ProtoCheckCompatibilityResponse, CompatibilityResultProto,
    DeleteVersionRequest as ProtoDeleteVersionRequest,
    DeleteVersionResponse as ProtoDeleteVersionResponse, DiffEntryProto, DiffModifiedEntryProto,
    GetDiffRequest as ProtoGetDiffRequest, GetDiffResponse as ProtoGetDiffResponse,
    GetSchemaRequest as ProtoGetSchemaRequest, GetSchemaResponse as ProtoGetSchemaResponse,
    GetSchemaVersionRequest as ProtoGetSchemaVersionRequest,
    GetSchemaVersionResponse as ProtoGetSchemaVersionResponse,
    ListSchemasRequest as ProtoListSchemasRequest, ListSchemasResponse as ProtoListSchemasResponse,
    ListVersionsRequest as ProtoListVersionsRequest,
    ListVersionsResponse as ProtoListVersionsResponse,
    RegisterSchemaRequest as ProtoRegisterSchemaRequest,
    RegisterSchemaResponse as ProtoRegisterSchemaResponse,
    RegisterVersionRequest as ProtoRegisterVersionRequest,
    RegisterVersionResponse as ProtoRegisterVersionResponse, SchemaChange as ProtoSchemaChange,
    SchemaDiffProto,
};
use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};

use super::apiregistry_grpc::{
    ApiRegistryGrpcService, ApiSchemaData, ApiSchemaVersionData, ChangeDetailData,
    CheckCompatibilityRequest, DeleteVersionRequest, GetDiffRequest, GetSchemaRequest,
    GetSchemaVersionRequest, GrpcError, ListSchemasRequest, ListVersionsRequest,
    RegisterSchemaRequest, RegisterVersionRequest, SchemaDiffData,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// プロトの Timestamp フィールドは Option<ProtoTimestamp> 型を要求するため Option でラップする
#[allow(clippy::unnecessary_wraps)]
fn to_proto_timestamp(dt: &DateTime<Utc>) -> Option<ProtoTimestamp> {
    Some(ProtoTimestamp {
        seconds: dt.timestamp(),
        // LOW-008: 安全な型変換（オーバーフロー防止）
        nanos: i32::try_from(dt.timestamp_subsec_nanos()).unwrap_or(i32::MAX),
    })
}

fn to_proto_schema(schema: ApiSchemaData) -> ApiSchemaProto {
    ApiSchemaProto {
        name: schema.name,
        description: schema.description,
        schema_type: schema.schema_type,
        latest_version: schema.latest_version,
        version_count: schema.version_count,
        created_at: to_proto_timestamp(&schema.created_at),
        updated_at: to_proto_timestamp(&schema.updated_at),
    }
}

fn to_proto_schema_change(detail: ChangeDetailData) -> ProtoSchemaChange {
    ProtoSchemaChange {
        change_type: detail.change_type,
        path: detail.path,
        description: detail.description,
    }
}

fn to_proto_schema_version(version: ApiSchemaVersionData) -> ApiSchemaVersionProto {
    ApiSchemaVersionProto {
        name: version.name,
        version: version.version,
        schema_type: version.schema_type,
        content: version.content,
        content_hash: version.content_hash,
        breaking_changes: version.breaking_changes,
        registered_by: version.registered_by,
        created_at: to_proto_timestamp(&version.created_at),
        breaking_change_details: version
            .breaking_change_details
            .into_iter()
            .map(to_proto_schema_change)
            .collect(),
    }
}

fn to_proto_diff(diff: SchemaDiffData) -> SchemaDiffProto {
    SchemaDiffProto {
        added: diff
            .added
            .into_iter()
            .map(|e| DiffEntryProto {
                path: e.path,
                r#type: e.entry_type,
                description: e.description,
            })
            .collect(),
        modified: diff
            .modified
            .into_iter()
            .map(|e| DiffModifiedEntryProto {
                path: e.path,
                before: e.before,
                after: e.after,
            })
            .collect(),
        removed: diff
            .removed
            .into_iter()
            .map(|e| DiffEntryProto {
                path: e.path,
                r#type: e.entry_type,
                description: e.description,
            })
            .collect(),
    }
}

pub struct ApiRegistryServiceTonic {
    inner: Arc<ApiRegistryGrpcService>,
}

impl ApiRegistryServiceTonic {
    #[must_use]
    pub fn new(inner: Arc<ApiRegistryGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl ApiRegistryService for ApiRegistryServiceTonic {
    async fn list_schemas(
        &self,
        request: Request<ProtoListSchemasRequest>,
    ) -> Result<Response<ProtoListSchemasResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map_or((1, 20), |p| (p.page, p.page_size));
        let resp = self
            .inner
            .list_schemas(ListSchemasRequest {
                schema_type: inner.schema_type,
                page,
                page_size,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListSchemasResponse {
            schemas: resp.schemas.into_iter().map(to_proto_schema).collect(),
            pagination: Some(ProtoPaginationResult {
                // LOW-008: 安全な型変換（オーバーフロー防止）
                total_count: i64::try_from(resp.total_count).unwrap_or(i64::MAX),
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn register_schema(
        &self,
        request: Request<ProtoRegisterSchemaRequest>,
    ) -> Result<Response<ProtoRegisterSchemaResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .register_schema(RegisterSchemaRequest {
                name: inner.name,
                description: inner.description,
                schema_type: inner.schema_type,
                content: inner.content,
                registered_by: inner.registered_by,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoRegisterSchemaResponse {
            version: Some(to_proto_schema_version(resp.version)),
        }))
    }

    async fn get_schema(
        &self,
        request: Request<ProtoGetSchemaRequest>,
    ) -> Result<Response<ProtoGetSchemaResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_schema(GetSchemaRequest { name: inner.name })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetSchemaResponse {
            schema: Some(to_proto_schema(resp.schema)),
            latest_content: resp.latest_content,
        }))
    }

    async fn list_versions(
        &self,
        request: Request<ProtoListVersionsRequest>,
    ) -> Result<Response<ProtoListVersionsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map_or((1, 20), |p| (p.page, p.page_size));
        let resp = self
            .inner
            .list_versions(ListVersionsRequest {
                name: inner.name,
                page,
                page_size,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListVersionsResponse {
            name: resp.name,
            versions: resp
                .versions
                .into_iter()
                .map(to_proto_schema_version)
                .collect(),
            pagination: Some(ProtoPaginationResult {
                // LOW-008: 安全な型変換（オーバーフロー防止）
                total_count: i64::try_from(resp.total_count).unwrap_or(i64::MAX),
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn register_version(
        &self,
        request: Request<ProtoRegisterVersionRequest>,
    ) -> Result<Response<ProtoRegisterVersionResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .register_version(RegisterVersionRequest {
                name: inner.name,
                content: inner.content,
                registered_by: inner.registered_by,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoRegisterVersionResponse {
            version: Some(to_proto_schema_version(resp.version)),
        }))
    }

    async fn get_schema_version(
        &self,
        request: Request<ProtoGetSchemaVersionRequest>,
    ) -> Result<Response<ProtoGetSchemaVersionResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_schema_version(GetSchemaVersionRequest {
                name: inner.name,
                version: inner.version,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetSchemaVersionResponse {
            version: Some(to_proto_schema_version(resp.version)),
        }))
    }

    async fn delete_version(
        &self,
        request: Request<ProtoDeleteVersionRequest>,
    ) -> Result<Response<ProtoDeleteVersionResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_version(DeleteVersionRequest {
                name: inner.name,
                version: inner.version,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteVersionResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn check_compatibility(
        &self,
        request: Request<ProtoCheckCompatibilityRequest>,
    ) -> Result<Response<ProtoCheckCompatibilityResponse>, Status> {
        let inner = request.into_inner();
        let name = inner.name.clone();
        let resp = self
            .inner
            .check_compatibility(CheckCompatibilityRequest {
                name: inner.name,
                content: inner.content,
                base_version: inner.base_version,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCheckCompatibilityResponse {
            name,
            base_version: resp.base_version,
            result: Some(CompatibilityResultProto {
                compatible: resp.compatible,
                breaking_changes: resp
                    .breaking_changes
                    .into_iter()
                    .map(to_proto_schema_change)
                    .collect(),
                non_breaking_changes: resp
                    .non_breaking_changes
                    .into_iter()
                    .map(to_proto_schema_change)
                    .collect(),
            }),
        }))
    }

    async fn get_diff(
        &self,
        request: Request<ProtoGetDiffRequest>,
    ) -> Result<Response<ProtoGetDiffResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_diff(GetDiffRequest {
                name: inner.name,
                from_version: inner.from_version,
                to_version: inner.to_version,
            })
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetDiffResponse {
            name: resp.name,
            from_version: resp.from_version,
            to_version: resp.to_version,
            breaking_changes: resp.breaking_changes,
            diff: Some(to_proto_diff(resp.diff)),
        }))
    }
}
