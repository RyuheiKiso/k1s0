//! gRPC service implementation for ApiRegistry.
//! Proto-generated code is not yet available, so types are manually defined.

use std::sync::Arc;

use crate::usecase::get_schema::{GetSchemaError, GetSchemaUseCase};
use crate::usecase::get_schema_version::{GetSchemaVersionError, GetSchemaVersionUseCase};
use crate::usecase::check_compatibility::{CheckCompatibilityError, CheckCompatibilityInput, CheckCompatibilityUseCase};

// --- Manually defined gRPC request/response types (pending proto codegen) ---

#[derive(Debug, Clone)]
pub struct GetSchemaRequest {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct GetSchemaResponse {
    pub name: String,
    pub description: String,
    pub schema_type: String,
    pub latest_version: u32,
    pub version_count: u32,
}

#[derive(Debug, Clone)]
pub struct GetSchemaVersionRequest {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct GetSchemaVersionResponse {
    pub name: String,
    pub version: u32,
    pub schema_type: String,
    pub content: String,
    pub content_hash: String,
    pub breaking_changes: bool,
    pub registered_by: String,
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
    pub breaking_change_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("internal error: {0}")]
    Internal(String),
}

// --- Service implementation ---

pub struct ApiRegistryGrpcService {
    get_schema_uc: Arc<GetSchemaUseCase>,
    get_schema_version_uc: Arc<GetSchemaVersionUseCase>,
    check_compatibility_uc: Arc<CheckCompatibilityUseCase>,
}

impl ApiRegistryGrpcService {
    pub fn new(
        get_schema_uc: Arc<GetSchemaUseCase>,
        get_schema_version_uc: Arc<GetSchemaVersionUseCase>,
        check_compatibility_uc: Arc<CheckCompatibilityUseCase>,
    ) -> Self {
        Self { get_schema_uc, get_schema_version_uc, check_compatibility_uc }
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
            name: output.schema.name,
            description: output.schema.description,
            schema_type: output.schema.schema_type.to_string(),
            latest_version: output.schema.latest_version,
            version_count: output.schema.version_count,
        })
    }

    pub async fn get_schema_version(
        &self,
        request: GetSchemaVersionRequest,
    ) -> Result<GetSchemaVersionResponse, GrpcError> {
        if request.name.is_empty() {
            return Err(GrpcError::InvalidArgument("name is required".to_string()));
        }
        let output = self.get_schema_version_uc.execute(&request.name, request.version).await
            .map_err(|e| match e {
                GetSchemaVersionError::NotFound { name, version } => GrpcError::NotFound(format!("{}@{} not found", name, version)),
                GetSchemaVersionError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(GetSchemaVersionResponse {
            name: output.name,
            version: output.version,
            schema_type: output.schema_type.to_string(),
            content: output.content,
            content_hash: output.content_hash,
            breaking_changes: output.breaking_changes,
            registered_by: output.registered_by,
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
        let output = self.check_compatibility_uc.execute(&input).await
            .map_err(|e| match e {
                CheckCompatibilityError::SchemaNotFound(n) => GrpcError::NotFound(format!("schema not found: {}", n)),
                CheckCompatibilityError::VersionNotFound { name, version } => GrpcError::NotFound(format!("{}@{} not found", name, version)),
                CheckCompatibilityError::Internal(msg) => GrpcError::Internal(msg),
            })?;
        Ok(CheckCompatibilityResponse {
            compatible: output.result.compatible,
            base_version: output.base_version,
            breaking_change_count: output.result.breaking_changes.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("schema not found: test-api".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), Code::NotFound);
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("name is required".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), Code::Internal);
    }
}
