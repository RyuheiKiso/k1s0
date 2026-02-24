//! tonic gRPC service wrapper for ApiRegistry.
//! Until proto-codegen is run, this wraps the hand-typed service impl.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::apiregistry_grpc::{
    ApiRegistryGrpcService, CheckCompatibilityRequest, CheckCompatibilityResponse,
    GetSchemaRequest, GetSchemaResponse, GetSchemaVersionRequest, GetSchemaVersionResponse,
    GrpcError,
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

    pub async fn get_schema(
        &self,
        request: Request<GetSchemaRequest>,
    ) -> Result<Response<GetSchemaResponse>, Status> {
        let resp = self.inner.get_schema(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn get_schema_version(
        &self,
        request: Request<GetSchemaVersionRequest>,
    ) -> Result<Response<GetSchemaVersionResponse>, Status> {
        let resp = self.inner.get_schema_version(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn check_compatibility(
        &self,
        request: Request<CheckCompatibilityRequest>,
    ) -> Result<Response<CheckCompatibilityResponse>, Status> {
        let resp = self.inner.check_compatibility(request.into_inner()).await?;
        Ok(Response::new(resp))
    }
}
