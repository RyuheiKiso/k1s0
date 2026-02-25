//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の TenantService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の TenantGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::tenant::v1::{
    tenant_service_server::TenantService,
    AddMemberRequest as ProtoAddMemberRequest, AddMemberResponse as ProtoAddMemberResponse,
    CreateTenantRequest as ProtoCreateTenantRequest,
    CreateTenantResponse as ProtoCreateTenantResponse,
    GetProvisioningStatusRequest as ProtoGetProvisioningStatusRequest,
    GetProvisioningStatusResponse as ProtoGetProvisioningStatusResponse,
    GetTenantRequest as ProtoGetTenantRequest, GetTenantResponse as ProtoGetTenantResponse,
    ListTenantsRequest as ProtoListTenantsRequest,
    ListTenantsResponse as ProtoListTenantsResponse, ProvisioningJob as ProtoProvisioningJob,
    RemoveMemberRequest as ProtoRemoveMemberRequest,
    RemoveMemberResponse as ProtoRemoveMemberResponse, Tenant as ProtoTenant,
    TenantMember as ProtoTenantMember,
};

use super::tenant_grpc::{
    AddMemberRequest, CreateTenantRequest, GetProvisioningStatusRequest, GetTenantRequest,
    GrpcError, ListTenantsRequest, RemoveMemberRequest, TenantGrpcService,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- 変換ヘルパー ---

fn pb_timestamp_to_proto(
    ts: &super::tenant_grpc::PbTimestamp,
) -> ::prost_types::Timestamp {
    ::prost_types::Timestamp {
        seconds: ts.seconds,
        nanos: ts.nanos,
    }
}

fn pb_tenant_to_proto(t: &super::tenant_grpc::PbTenant) -> ProtoTenant {
    ProtoTenant {
        id: t.id.clone(),
        name: t.name.clone(),
        display_name: t.display_name.clone(),
        status: t.status.clone(),
        plan: t.plan.clone(),
        created_at: t.created_at.as_ref().map(pb_timestamp_to_proto),
    }
}

fn pb_member_to_proto(m: &super::tenant_grpc::PbTenantMember) -> ProtoTenantMember {
    ProtoTenantMember {
        id: m.id.clone(),
        tenant_id: m.tenant_id.clone(),
        user_id: m.user_id.clone(),
        role: m.role.clone(),
        joined_at: m.joined_at.as_ref().map(pb_timestamp_to_proto),
    }
}

fn pb_job_to_proto(j: &super::tenant_grpc::PbProvisioningJob) -> ProtoProvisioningJob {
    ProtoProvisioningJob {
        id: j.id.clone(),
        tenant_id: j.tenant_id.clone(),
        status: j.status.clone(),
        current_step: j.current_step.clone(),
        error_message: j.error_message.clone(),
        created_at: j.created_at.as_ref().map(pb_timestamp_to_proto),
        updated_at: j.updated_at.as_ref().map(pb_timestamp_to_proto),
    }
}

// --- TenantService tonic ラッパー ---

pub struct TenantServiceTonic {
    inner: Arc<TenantGrpcService>,
}

impl TenantServiceTonic {
    pub fn new(inner: Arc<TenantGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl TenantService for TenantServiceTonic {
    async fn create_tenant(
        &self,
        request: Request<ProtoCreateTenantRequest>,
    ) -> Result<Response<ProtoCreateTenantResponse>, Status> {
        let inner = request.into_inner();
        let req = CreateTenantRequest {
            name: inner.name,
            display_name: inner.display_name,
            plan: inner.plan,
            owner_id: inner.owner_user_id,
        };
        let resp = self
            .inner
            .create_tenant(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateTenantResponse {
            tenant: resp.tenant.as_ref().map(pb_tenant_to_proto),
        }))
    }

    async fn get_tenant(
        &self,
        request: Request<ProtoGetTenantRequest>,
    ) -> Result<Response<ProtoGetTenantResponse>, Status> {
        let req = GetTenantRequest {
            tenant_id: request.into_inner().tenant_id,
        };
        let resp = self
            .inner
            .get_tenant(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetTenantResponse {
            tenant: resp.tenant.as_ref().map(pb_tenant_to_proto),
        }))
    }

    async fn list_tenants(
        &self,
        request: Request<ProtoListTenantsRequest>,
    ) -> Result<Response<ProtoListTenantsResponse>, Status> {
        let inner = request.into_inner();
        let pagination = inner.pagination.unwrap_or_default();
        let req = ListTenantsRequest {
            page: pagination.page,
            page_size: pagination.page_size,
        };
        let resp = self
            .inner
            .list_tenants(req)
            .await
            .map_err(Into::<Status>::into)?;

        let tenants = resp.tenants.iter().map(pb_tenant_to_proto).collect();

        Ok(Response::new(ProtoListTenantsResponse {
            tenants,
            pagination: Some(
                crate::proto::k1s0::system::common::v1::PaginationResult {
                    total_count: resp.total_count as i32,
                    page: 0,
                    page_size: 0,
                    has_next: false,
                },
            ),
        }))
    }

    async fn add_member(
        &self,
        request: Request<ProtoAddMemberRequest>,
    ) -> Result<Response<ProtoAddMemberResponse>, Status> {
        let inner = request.into_inner();
        let req = AddMemberRequest {
            tenant_id: inner.tenant_id,
            user_id: inner.user_id,
            role: inner.role,
        };
        let resp = self
            .inner
            .add_member(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoAddMemberResponse {
            member: resp.member.as_ref().map(pb_member_to_proto),
        }))
    }

    async fn remove_member(
        &self,
        request: Request<ProtoRemoveMemberRequest>,
    ) -> Result<Response<ProtoRemoveMemberResponse>, Status> {
        let inner = request.into_inner();
        let req = RemoveMemberRequest {
            tenant_id: inner.tenant_id,
            user_id: inner.user_id,
        };
        let resp = self
            .inner
            .remove_member(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRemoveMemberResponse {
            success: resp.success,
        }))
    }

    async fn get_provisioning_status(
        &self,
        request: Request<ProtoGetProvisioningStatusRequest>,
    ) -> Result<Response<ProtoGetProvisioningStatusResponse>, Status> {
        let req = GetProvisioningStatusRequest {
            job_id: request.into_inner().job_id,
        };
        let resp = self
            .inner
            .get_provisioning_status(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetProvisioningStatusResponse {
            job: resp.job.as_ref().map(pb_job_to_proto),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("tenant not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("tenant not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid input".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_already_exists_to_status() {
        let err = GrpcError::AlreadyExists("already exists".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::AlreadyExists);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("internal error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
