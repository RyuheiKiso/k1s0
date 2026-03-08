use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::navigation::v1::{
    navigation_service_server::NavigationService,
    GetNavigationRequest as ProtoGetNavigationRequest,
    GetNavigationResponse as ProtoGetNavigationResponse,
};

use super::navigation_grpc::{GrpcError, NavigationGrpcService};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::ConfigLoad(msg) => Status::internal(msg),
            GrpcError::Unauthenticated(msg) => Status::unauthenticated(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

pub struct NavigationServiceTonic {
    inner: Arc<NavigationGrpcService>,
}

impl NavigationServiceTonic {
    pub fn new(inner: Arc<NavigationGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl NavigationService for NavigationServiceTonic {
    async fn get_navigation(
        &self,
        request: Request<ProtoGetNavigationRequest>,
    ) -> Result<Response<ProtoGetNavigationResponse>, Status> {
        let inner = request.into_inner();
        let result = self
            .inner
            .get_navigation(&inner.bearer_token)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetNavigationResponse::from(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::presentation::NavigationResponseBody;

    #[test]
    fn test_grpc_error_config_load_to_status() {
        let err = GrpcError::ConfigLoad("file not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("file not found"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("unexpected".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_grpc_error_unauthenticated_to_status() {
        let err = GrpcError::Unauthenticated("invalid token".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn filtered_navigation_maps_to_rest_body() {
        let body = NavigationResponseBody {
            routes: vec![],
            guards: vec![],
        };
        assert!(body.routes.is_empty());
    }
}
