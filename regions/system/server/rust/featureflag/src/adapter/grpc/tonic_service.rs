//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の FeatureFlagService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の FeatureFlagGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::featureflag::v1::{
    feature_flag_service_server::FeatureFlagService,
    CreateFlagRequest as ProtoCreateFlagRequest, CreateFlagResponse as ProtoCreateFlagResponse,
    DeleteFlagRequest as ProtoDeleteFlagRequest, DeleteFlagResponse as ProtoDeleteFlagResponse,
    EvaluateFlagRequest as ProtoEvaluateFlagRequest,
    EvaluateFlagResponse as ProtoEvaluateFlagResponse, FeatureFlag as ProtoFeatureFlag,
    FlagRule as ProtoFlagRule, FlagVariant as ProtoFlagVariant,
    GetFlagRequest as ProtoGetFlagRequest, GetFlagResponse as ProtoGetFlagResponse,
    ListFlagsRequest as ProtoListFlagsRequest, ListFlagsResponse as ProtoListFlagsResponse,
    UpdateFlagRequest as ProtoUpdateFlagRequest, UpdateFlagResponse as ProtoUpdateFlagResponse,
};

use super::featureflag_grpc::{
    CreateFlagRequest, DeleteFlagRequest, EvaluateFlagRequest, FeatureFlagGrpcService,
    GetFlagRequest, GrpcError, ListFlagsRequest, PbFlagVariant, UpdateFlagRequest,
};

fn to_proto_timestamp(
    dt: chrono::DateTime<chrono::Utc>,
) -> crate::proto::k1s0::system::common::v1::Timestamp {
    crate::proto::k1s0::system::common::v1::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

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
            user_id: ctx.user_id.unwrap_or_default(),
            tenant_id: ctx.tenant_id.unwrap_or_default(),
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
                id: resp.id,
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
                rules: resp
                    .rules
                    .into_iter()
                    .map(|r| ProtoFlagRule {
                        attribute: r.attribute,
                        operator: r.operator,
                        value: r.value,
                        variant: r.variant,
                    })
                    .collect(),
                created_at: Some(to_proto_timestamp(resp.created_at)),
                updated_at: Some(to_proto_timestamp(resp.updated_at)),
            }),
        }))
    }

    async fn list_flags(
        &self,
        _request: Request<ProtoListFlagsRequest>,
    ) -> Result<Response<ProtoListFlagsResponse>, Status> {
        let resp = self
            .inner
            .list_flags(ListFlagsRequest {})
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListFlagsResponse {
            flags: resp
                .flags
                .into_iter()
                .map(|flag| ProtoFeatureFlag {
                    id: flag.id,
                    flag_key: flag.flag_key,
                    description: flag.description,
                    enabled: flag.enabled,
                    variants: flag
                        .variants
                        .into_iter()
                        .map(|v| ProtoFlagVariant {
                            name: v.name,
                            value: v.value,
                            weight: v.weight,
                        })
                        .collect(),
                    rules: flag
                        .rules
                        .into_iter()
                        .map(|r| ProtoFlagRule {
                            attribute: r.attribute,
                            operator: r.operator,
                            value: r.value,
                            variant: r.variant,
                        })
                        .collect(),
                    created_at: Some(to_proto_timestamp(flag.created_at)),
                    updated_at: Some(to_proto_timestamp(flag.updated_at)),
                })
                .collect(),
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
                id: resp.id,
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
                rules: resp
                    .rules
                    .into_iter()
                    .map(|r| ProtoFlagRule {
                        attribute: r.attribute,
                        operator: r.operator,
                        value: r.value,
                        variant: r.variant,
                    })
                    .collect(),
                created_at: Some(to_proto_timestamp(resp.created_at)),
                updated_at: Some(to_proto_timestamp(resp.updated_at)),
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
            enabled: inner.enabled,
            description: inner.description.filter(|v| !v.is_empty()),
        };
        let resp = self
            .inner
            .update_flag(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoUpdateFlagResponse {
            flag: Some(ProtoFeatureFlag {
                id: resp.id,
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
                rules: resp
                    .rules
                    .into_iter()
                    .map(|r| ProtoFlagRule {
                        attribute: r.attribute,
                        operator: r.operator,
                        value: r.value,
                        variant: r.variant,
                    })
                    .collect(),
                created_at: Some(to_proto_timestamp(resp.created_at)),
                updated_at: Some(to_proto_timestamp(resp.updated_at)),
            }),
        }))
    }

    async fn delete_flag(
        &self,
        request: Request<ProtoDeleteFlagRequest>,
    ) -> Result<Response<ProtoDeleteFlagResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_flag(DeleteFlagRequest {
                flag_key: inner.flag_key,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeleteFlagResponse {
            success: resp.success,
            message: resp.message,
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
