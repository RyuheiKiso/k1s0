//! tonic gRPC service wrapper for ApiRegistry.
//! proto 生成コードの ApiRegistryService トレイトを実装する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::apiregistry::v1::{
    api_registry_service_server::ApiRegistryService,
    ApiSchemaProto, ApiSchemaVersionProto, CheckCompatibilityRequest as ProtoCheckCompatibilityRequest,
    CheckCompatibilityResponse as ProtoCheckCompatibilityResponse,
    CompatibilityResultProto, GetSchemaRequest as ProtoGetSchemaRequest,
    GetSchemaResponse as ProtoGetSchemaResponse,
    GetSchemaVersionRequest as ProtoGetSchemaVersionRequest,
    GetSchemaVersionResponse as ProtoGetSchemaVersionResponse,
};

use super::apiregistry_grpc::{
    ApiRegistryGrpcService, CheckCompatibilityRequest,
    GetSchemaRequest, GetSchemaVersionRequest, GrpcError,
};

// --- GrpcError -> tonic::Status conversion ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- tonic wrapper ---

pub struct ApiRegistryServiceTonic {
    inner: Arc<ApiRegistryGrpcService>,
}

impl ApiRegistryServiceTonic {
    pub fn new(inner: Arc<ApiRegistryGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl ApiRegistryService for ApiRegistryServiceTonic {
    async fn get_schema(
        &self,
        request: Request<ProtoGetSchemaRequest>,
    ) -> Result<Response<ProtoGetSchemaResponse>, Status> {
        let inner = request.into_inner();
        let req = GetSchemaRequest { name: inner.name };
        let resp = self.inner.get_schema(req).await.map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetSchemaResponse {
            schema: Some(ApiSchemaProto {
                name: resp.name,
                description: resp.description,
                schema_type: resp.schema_type,
                latest_version: resp.latest_version,
                version_count: resp.version_count,
                created_at: None,
                updated_at: None,
            }),
            latest_content: String::new(),
        }))
    }

    async fn get_schema_version(
        &self,
        request: Request<ProtoGetSchemaVersionRequest>,
    ) -> Result<Response<ProtoGetSchemaVersionResponse>, Status> {
        let inner = request.into_inner();
        let req = GetSchemaVersionRequest {
            name: inner.name,
            version: inner.version,
        };
        let resp = self.inner.get_schema_version(req).await.map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetSchemaVersionResponse {
            version: Some(ApiSchemaVersionProto {
                name: resp.name,
                version: resp.version,
                schema_type: resp.schema_type,
                content: resp.content,
                content_hash: resp.content_hash,
                breaking_changes: resp.breaking_changes,
                registered_by: resp.registered_by,
                created_at: None,
            }),
        }))
    }

    async fn check_compatibility(
        &self,
        request: Request<ProtoCheckCompatibilityRequest>,
    ) -> Result<Response<ProtoCheckCompatibilityResponse>, Status> {
        let inner = request.into_inner();
        let name = inner.name.clone();
        let req = CheckCompatibilityRequest {
            name: inner.name,
            content: inner.content,
            base_version: inner.base_version,
        };
        let resp = self.inner.check_compatibility(req).await.map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCheckCompatibilityResponse {
            name,
            base_version: resp.base_version,
            result: Some(CompatibilityResultProto {
                compatible: resp.compatible,
                breaking_changes: Vec::new(),
                non_breaking_changes: Vec::new(),
            }),
        }))
    }
}
