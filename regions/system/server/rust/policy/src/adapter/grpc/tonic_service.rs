use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::policy::v1::{
    policy_service_server::PolicyService, CreateBundleRequest as ProtoCreateBundleRequest,
    CreateBundleResponse as ProtoCreateBundleResponse, CreatePolicyRequest as ProtoCreatePolicyRequest,
    CreatePolicyResponse as ProtoCreatePolicyResponse, DeletePolicyRequest as ProtoDeletePolicyRequest,
    DeletePolicyResponse as ProtoDeletePolicyResponse, EvaluatePolicyRequest as ProtoEvaluatePolicyRequest,
    EvaluatePolicyResponse as ProtoEvaluatePolicyResponse, GetPolicyRequest as ProtoGetPolicyRequest,
    GetPolicyResponse as ProtoGetPolicyResponse, ListBundlesRequest as ProtoListBundlesRequest,
    ListBundlesResponse as ProtoListBundlesResponse, ListPoliciesRequest as ProtoListPoliciesRequest,
    ListPoliciesResponse as ProtoListPoliciesResponse, Policy as ProtoPolicy,
    PolicyBundle as ProtoPolicyBundle, UpdatePolicyRequest as ProtoUpdatePolicyRequest,
    UpdatePolicyResponse as ProtoUpdatePolicyResponse,
};

use super::policy_grpc::{
    CreateBundleRequest, CreatePolicyRequest, DeletePolicyRequest, EvaluatePolicyRequest,
    GetPolicyRequest, GrpcError, ListBundlesRequest, ListPoliciesRequest, PolicyBundleData,
    PolicyData, PolicyGrpcService, UpdatePolicyRequest,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unimplemented(msg) => Status::unimplemented(msg),
        }
    }
}

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> Option<ProtoTimestamp> {
    Some(ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

fn to_proto_policy(policy: PolicyData) -> ProtoPolicy {
    ProtoPolicy {
        id: policy.id,
        name: policy.name,
        description: policy.description,
        package_path: policy.package_path,
        rego_content: policy.rego_content,
        bundle_id: policy.bundle_id,
        enabled: policy.enabled,
        version: policy.version,
        created_at: to_proto_timestamp(policy.created_at),
        updated_at: to_proto_timestamp(policy.updated_at),
    }
}

fn to_proto_bundle(bundle: PolicyBundleData) -> ProtoPolicyBundle {
    ProtoPolicyBundle {
        id: bundle.id,
        name: bundle.name,
        policy_ids: bundle.policy_ids,
        created_at: to_proto_timestamp(bundle.created_at),
        updated_at: to_proto_timestamp(bundle.updated_at),
        description: bundle.description.unwrap_or_default(),
        enabled: bundle.enabled,
    }
}

pub struct PolicyServiceTonic {
    inner: Arc<PolicyGrpcService>,
}

impl PolicyServiceTonic {
    pub fn new(inner: Arc<PolicyGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl PolicyService for PolicyServiceTonic {
    async fn evaluate_policy(
        &self,
        request: Request<ProtoEvaluatePolicyRequest>,
    ) -> Result<Response<ProtoEvaluatePolicyResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .evaluate_policy(EvaluatePolicyRequest {
                policy_id: inner.policy_id,
                input_json: inner.input_json,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoEvaluatePolicyResponse {
            allowed: resp.allowed,
            package_path: resp.package_path,
            decision_id: resp.decision_id,
            cached: resp.cached,
        }))
    }

    async fn get_policy(
        &self,
        request: Request<ProtoGetPolicyRequest>,
    ) -> Result<Response<ProtoGetPolicyResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .get_policy(GetPolicyRequest { id: inner.id })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetPolicyResponse {
            policy: Some(to_proto_policy(resp.policy)),
        }))
    }

    async fn list_policies(
        &self,
        request: Request<ProtoListPoliciesRequest>,
    ) -> Result<Response<ProtoListPoliciesResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner.pagination.map(|p| (p.page, p.page_size)).unwrap_or((1, 20));
        let resp = self
            .inner
            .list_policies(ListPoliciesRequest {
                page,
                page_size,
                bundle_id: inner.bundle_id,
                enabled_only: inner.enabled_only,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListPoliciesResponse {
            policies: resp.policies.into_iter().map(to_proto_policy).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count.min(i32::MAX as u64) as i32,
                page: resp.page,
                page_size: resp.page_size,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn create_policy(
        &self,
        request: Request<ProtoCreatePolicyRequest>,
    ) -> Result<Response<ProtoCreatePolicyResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .create_policy(CreatePolicyRequest {
                name: inner.name,
                description: inner.description,
                rego_content: inner.rego_content,
                package_path: inner.package_path,
                bundle_id: inner.bundle_id,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreatePolicyResponse {
            policy: Some(to_proto_policy(resp.policy)),
        }))
    }

    async fn update_policy(
        &self,
        request: Request<ProtoUpdatePolicyRequest>,
    ) -> Result<Response<ProtoUpdatePolicyResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .update_policy(UpdatePolicyRequest {
                id: inner.id,
                description: inner.description,
                rego_content: inner.rego_content,
                enabled: inner.enabled,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoUpdatePolicyResponse {
            policy: Some(to_proto_policy(resp.policy)),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<ProtoDeletePolicyRequest>,
    ) -> Result<Response<ProtoDeletePolicyResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .delete_policy(DeletePolicyRequest { id: inner.id })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeletePolicyResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn create_bundle(
        &self,
        request: Request<ProtoCreateBundleRequest>,
    ) -> Result<Response<ProtoCreateBundleResponse>, Status> {
        let inner = request.into_inner();
        let resp = self
            .inner
            .create_bundle(CreateBundleRequest {
                name: inner.name,
                description: inner.description,
                enabled: inner.enabled,
                policy_ids: inner.policy_ids,
            })
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateBundleResponse {
            bundle: Some(to_proto_bundle(resp.bundle)),
        }))
    }

    async fn list_bundles(
        &self,
        _request: Request<ProtoListBundlesRequest>,
    ) -> Result<Response<ProtoListBundlesResponse>, Status> {
        let resp = self
            .inner
            .list_bundles(ListBundlesRequest)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListBundlesResponse {
            bundles: resp.bundles.into_iter().map(to_proto_bundle).collect(),
        }))
    }
}
