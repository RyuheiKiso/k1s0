use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::domain::entity::navigation::{
    Guard, GuardType, Param, ParamType, Route, TransitionType,
};
use crate::proto::k1s0::system::navigation::v1::{
    navigation_service_server::NavigationService,
    GetNavigationRequest as ProtoGetNavigationRequest, Guard as ProtoGuard,
    GuardType as ProtoGuardType, NavigationResponse as ProtoNavigationResponse,
    Param as ProtoParam, ParamType as ProtoParamType, Route as ProtoRoute,
    TransitionConfig as ProtoTransitionConfig, TransitionType as ProtoTransitionType,
};

use super::navigation_grpc::{GrpcError, NavigationGrpcService};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::ConfigLoad(msg) => Status::internal(msg),
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
    ) -> Result<Response<ProtoNavigationResponse>, Status> {
        let inner = request.into_inner();
        let result = self
            .inner
            .get_navigation(&inner.bearer_token)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoNavigationResponse {
            routes: result.routes.into_iter().map(route_to_proto).collect(),
            guards: result.guards.into_iter().map(guard_to_proto).collect(),
        }))
    }
}

fn route_to_proto(route: Route) -> ProtoRoute {
    ProtoRoute {
        id: route.id,
        path: route.path,
        component_id: route.component_id.unwrap_or_default(),
        guard_ids: route.guards,
        children: route.children.into_iter().map(route_to_proto).collect(),
        transition: route.transition.map(transition_to_proto),
        params: route.params.into_iter().map(param_to_proto).collect(),
        redirect_to: route.redirect_to.unwrap_or_default(),
    }
}

fn guard_to_proto(guard: Guard) -> ProtoGuard {
    ProtoGuard {
        id: guard.id,
        r#type: guard_type_to_proto(&guard.guard_type) as i32,
        redirect_to: guard.redirect_to,
        roles: guard.roles,
    }
}

fn transition_to_proto(t: TransitionType) -> ProtoTransitionConfig {
    ProtoTransitionConfig {
        r#type: transition_type_to_proto(&t) as i32,
        duration_ms: 300,
    }
}

fn param_to_proto(p: Param) -> ProtoParam {
    ProtoParam {
        name: p.name,
        r#type: param_type_to_proto(&p.param_type) as i32,
    }
}

fn guard_type_to_proto(gt: &GuardType) -> ProtoGuardType {
    match gt {
        GuardType::AuthRequired => ProtoGuardType::AuthRequired,
        GuardType::RoleRequired => ProtoGuardType::RoleRequired,
        GuardType::RedirectIfAuthenticated => ProtoGuardType::RedirectIfAuthenticated,
    }
}

fn transition_type_to_proto(tt: &TransitionType) -> ProtoTransitionType {
    match tt {
        TransitionType::Fade => ProtoTransitionType::Fade,
        TransitionType::Slide => ProtoTransitionType::Slide,
        TransitionType::Modal => ProtoTransitionType::Modal,
    }
}

fn param_type_to_proto(pt: &ParamType) -> ProtoParamType {
    match pt {
        ParamType::String => ProtoParamType::String,
        ParamType::Int => ProtoParamType::Int,
        ParamType::Uuid => ProtoParamType::Uuid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_route_to_proto_conversion() {
        let route = Route {
            id: "test".to_string(),
            path: "/test".to_string(),
            component_id: Some("TestPage".to_string()),
            guards: vec!["auth".to_string()],
            transition: Some(TransitionType::Fade),
            redirect_to: None,
            children: vec![],
            params: vec![Param {
                name: "id".to_string(),
                param_type: ParamType::Uuid,
            }],
        };
        let proto = route_to_proto(route);
        assert_eq!(proto.id, "test");
        assert_eq!(proto.component_id, "TestPage");
        assert_eq!(proto.guard_ids, vec!["auth"]);
        assert!(proto.transition.is_some());
        let t = proto.transition.unwrap();
        assert_eq!(t.r#type, ProtoTransitionType::Fade as i32);
        assert_eq!(t.duration_ms, 300);
        assert_eq!(proto.params.len(), 1);
        assert_eq!(proto.params[0].r#type, ProtoParamType::Uuid as i32);
        assert_eq!(proto.redirect_to, "");
    }
}
