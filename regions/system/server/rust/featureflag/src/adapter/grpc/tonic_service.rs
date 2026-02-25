//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の FeatureFlagService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の FeatureFlagGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::featureflag::v1::{
    feature_flag_service_server::FeatureFlagService,
    CreateFlagRequest as ProtoCreateFlagRequest, CreateFlagResponse as ProtoCreateFlagResponse,
    EvaluateFlagRequest as ProtoEvaluateFlagRequest,
    EvaluateFlagResponse as ProtoEvaluateFlagResponse, FeatureFlag as ProtoFeatureFlag,
    FlagVariant as ProtoFlagVariant, GetFlagRequest as ProtoGetFlagRequest,
    GetFlagResponse as ProtoGetFlagResponse, UpdateFlagRequest as ProtoUpdateFlagRequest,
    UpdateFlagResponse as ProtoUpdateFlagResponse,
};

use super::featureflag_grpc::{
    CreateFlagRequest, EvaluateFlagRequest, FeatureFlagGrpcService, GetFlagRequest, GrpcError,
    PbFlagVariant, UpdateFlagRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- FeatureFlagService tonic ラッパー ---

pub struct FeatureFlagServiceTonic {
    inner: Arc<FeatureFlagGrpcService>,
}

impl FeatureFlagServiceTonic {
    pub fn new(inner: Arc<FeatureFlagGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl FeatureFlagService for FeatureFlagServiceTonic {
    async fn evaluate_flag(
        &self,
        request: Request<ProtoEvaluateFlagRequest>,
    ) -> Result<Response<ProtoEvaluateFlagResponse>, Status> {
        let inner = request.into_inner();
        let ctx = inner.context.unwrap_or_default();
        let req = EvaluateFlagRequest {
            flag_key: inner.flag_key,
            user_id: ctx.user_id,
            tenant_id: ctx.tenant_id,
            attributes: ctx.attributes,
        };
        let resp = self
            .inner
            .evaluate_flag(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoEvaluateFlagResponse {
            flag_key: resp.flag_key,
            enabled: resp.enabled,
            variant: resp.variant,
            reason: resp.reason,
        }))
    }

    async fn get_flag(
        &self,
        request: Request<ProtoGetFlagRequest>,
    ) -> Result<Response<ProtoGetFlagResponse>, Status> {
        let inner = request.into_inner();
        let req = GetFlagRequest {
            flag_key: inner.flag_key,
        };
        let resp = self
            .inner
            .get_flag(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetFlagResponse {
            flag: Some(ProtoFeatureFlag {
                id: String::new(),
                flag_key: resp.flag_key,
                description: resp.description,
                enabled: resp.enabled,
                variants: resp
                    .variants
                    .into_iter()
                    .map(|v| ProtoFlagVariant {
                        name: v.name,
                        value: v.value,
                        weight: v.weight,
                    })
                    .collect(),
                created_at: None,
                updated_at: None,
            }),
        }))
    }

    async fn create_flag(
        &self,
        request: Request<ProtoCreateFlagRequest>,
    ) -> Result<Response<ProtoCreateFlagResponse>, Status> {
        let inner = request.into_inner();
        let req = CreateFlagRequest {
            flag_key: inner.flag_key,
            description: inner.description,
            enabled: inner.enabled,
            variants: inner
                .variants
                .into_iter()
                .map(|v| PbFlagVariant {
                    name: v.name,
                    value: v.value,
                    weight: v.weight,
                })
                .collect(),
        };
        let resp = self
            .inner
            .create_flag(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateFlagResponse {
            flag: Some(ProtoFeatureFlag {
                id: String::new(),
                flag_key: resp.flag_key,
                description: resp.description,
                enabled: resp.enabled,
                variants: vec![],
                created_at: None,
                updated_at: None,
            }),
        }))
    }

    async fn update_flag(
        &self,
        request: Request<ProtoUpdateFlagRequest>,
    ) -> Result<Response<ProtoUpdateFlagResponse>, Status> {
        let inner = request.into_inner();
        let req = UpdateFlagRequest {
            flag_key: inner.flag_key,
            enabled: if inner.enabled { Some(true) } else { Some(false) },
            description: if inner.description.is_empty() {
                None
            } else {
                Some(inner.description)
            },
        };
        let resp = self
            .inner
            .update_flag(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoUpdateFlagResponse {
            flag: Some(ProtoFeatureFlag {
                id: String::new(),
                flag_key: resp.flag_key,
                description: resp.description,
                enabled: resp.enabled,
                variants: vec![],
                created_at: None,
                updated_at: None,
            }),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("flag not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("flag not found"));
    }

    #[test]
    fn test_grpc_error_already_exists_to_status() {
        let err = GrpcError::AlreadyExists("flag exists".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::AlreadyExists);
        assert!(status.message().contains("flag exists"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("bad input".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("bad input"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }
}
