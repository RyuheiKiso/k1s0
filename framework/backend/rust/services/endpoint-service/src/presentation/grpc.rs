//! gRPCサービス実装

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tonic::{Request, Response, Status};

use crate::application::EndpointService;
use crate::domain::EndpointRepository;

// Generated code from proto
pub mod endpoint_v1 {
    tonic::include_proto!("k1s0.endpoint.v1");
}

use endpoint_v1::endpoint_service_server::EndpointService as EndpointServiceTrait;
use endpoint_v1::{
    Endpoint as ProtoEndpoint, GetEndpointRequest, GetEndpointResponse, ListEndpointsRequest,
    ListEndpointsResponse, ResolveEndpointRequest, ResolveEndpointResponse,
};

/// gRPCサービス実装
pub struct GrpcEndpointService<R>
where
    R: EndpointRepository + 'static,
{
    service: Arc<EndpointService<R>>,
}

impl<R> GrpcEndpointService<R>
where
    R: EndpointRepository + 'static,
{
    /// 新しいgRPCサービスを作成
    pub fn new(service: Arc<EndpointService<R>>) -> Self {
        Self { service }
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn endpoint_to_proto(endpoint: crate::domain::Endpoint) -> ProtoEndpoint {
    ProtoEndpoint {
        endpoint_id: endpoint.id,
        service_name: endpoint.service_name,
        path: endpoint.path,
        method: endpoint.method,
        created_at: system_time_to_rfc3339(endpoint.created_at),
        updated_at: system_time_to_rfc3339(endpoint.updated_at),
    }
}

#[tonic::async_trait]
impl<R> EndpointServiceTrait for GrpcEndpointService<R>
where
    R: EndpointRepository + 'static,
{
    async fn get_endpoint(
        &self,
        request: Request<GetEndpointRequest>,
    ) -> Result<Response<GetEndpointResponse>, Status> {
        let req = request.into_inner();

        let method = if req.method.is_empty() {
            None
        } else {
            Some(req.method.as_str())
        };
        let path = if req.path.is_empty() {
            None
        } else {
            Some(req.path.as_str())
        };

        let endpoint = self
            .service
            .get_endpoint(&req.service_name, method, path)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(GetEndpointResponse {
            endpoint: Some(endpoint_to_proto(endpoint)),
        }))
    }

    async fn list_endpoints(
        &self,
        request: Request<ListEndpointsRequest>,
    ) -> Result<Response<ListEndpointsResponse>, Status> {
        let req = request.into_inner();

        let mut query = crate::domain::EndpointQuery::new();
        if !req.service_name.is_empty() {
            query = query.with_service_name(&req.service_name);
        }
        if req.page_size > 0 {
            query = query.with_page_size(req.page_size as u32);
        }
        if !req.page_token.is_empty() {
            query = query.with_page_token(&req.page_token);
        }

        let result = self
            .service
            .list_endpoints(&query)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(ListEndpointsResponse {
            endpoints: result.endpoints.into_iter().map(endpoint_to_proto).collect(),
            next_page_token: result.next_page_token.unwrap_or_default(),
        }))
    }

    async fn resolve_endpoint(
        &self,
        request: Request<ResolveEndpointRequest>,
    ) -> Result<Response<ResolveEndpointResponse>, Status> {
        let req = request.into_inner();

        let resolved = self
            .service
            .resolve_endpoint(&req.service_name, &req.protocol)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(ResolveEndpointResponse {
            address: resolved.address,
            use_tls: resolved.use_tls,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_time_to_rfc3339() {
        let time = SystemTime::UNIX_EPOCH;
        let result = system_time_to_rfc3339(time);
        assert_eq!(result, "1970-01-01T00:00:00Z");
    }
}
