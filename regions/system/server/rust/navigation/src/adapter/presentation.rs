use serde::Serialize;

use crate::domain::entity::navigation::{
    Guard, GuardType, Param, ParamType, Route, TransitionType,
};
use crate::domain::service::navigation_filter::FilteredNavigation;
use crate::proto::k1s0::system::navigation::v1::{
    GetNavigationResponse as ProtoGetNavigationResponse, Guard as ProtoGuard,
    GuardType as ProtoGuardType, Param as ProtoParam, ParamType as ProtoParamType,
    Route as ProtoRoute, TransitionConfig as ProtoTransitionConfig,
    TransitionType as ProtoTransitionType,
};

#[derive(Debug, Serialize)]
pub struct NavigationResponseBody {
    pub routes: Vec<Route>,
    pub guards: Vec<Guard>,
}

impl From<FilteredNavigation> for NavigationResponseBody {
    fn from(value: FilteredNavigation) -> Self {
        Self {
            routes: value.routes,
            guards: value.guards,
        }
    }
}

impl From<FilteredNavigation> for ProtoGetNavigationResponse {
    fn from(value: FilteredNavigation) -> Self {
        Self {
            routes: value.routes.into_iter().map(route_to_proto).collect(),
            guards: value.guards.into_iter().map(guard_to_proto).collect(),
        }
    }
}

fn route_to_proto(route: Route) -> ProtoRoute {
    ProtoRoute {
        id: route.id,
        path: route.path,
        component_id: route.component_id,
        guard_ids: route.guards,
        children: route.children.into_iter().map(route_to_proto).collect(),
        transition: route
            .transition
            .map(|t| transition_to_proto(t, route.transition_duration_ms)),
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

fn transition_to_proto(t: TransitionType, duration_ms: u32) -> ProtoTransitionConfig {
    ProtoTransitionConfig {
        r#type: transition_type_to_proto(&t) as i32,
        duration_ms,
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
    fn filtered_navigation_converts_to_rest_body() {
        let filtered = FilteredNavigation {
            routes: vec![Route {
                id: "root".to_string(),
                path: "/".to_string(),
                component_id: None,
                guards: vec![],
                transition: None,
                transition_duration_ms: 300,
                redirect_to: Some("/dashboard".to_string()),
                children: vec![],
                params: vec![],
            }],
            guards: vec![],
        };

        let body = NavigationResponseBody::from(filtered);
        assert_eq!(body.routes.len(), 1);
        assert_eq!(body.routes[0].id, "root");
    }

    #[test]
    fn filtered_navigation_converts_to_proto_response() {
        let filtered = FilteredNavigation {
            routes: vec![Route {
                id: "test".to_string(),
                path: "/test".to_string(),
                component_id: Some("TestPage".to_string()),
                guards: vec!["auth".to_string()],
                transition: Some(TransitionType::Fade),
                transition_duration_ms: 450,
                redirect_to: None,
                children: vec![],
                params: vec![Param {
                    name: "id".to_string(),
                    param_type: ParamType::Uuid,
                }],
            }],
            guards: vec![],
        };

        let response = ProtoGetNavigationResponse::from(filtered);
        assert_eq!(response.routes.len(), 1);
        assert_eq!(response.routes[0].id, "test");
        assert_eq!(response.routes[0].guard_ids, vec!["auth"]);
        assert_eq!(
            response.routes[0]
                .transition
                .as_ref()
                .map(|t| t.duration_ms),
            Some(450)
        );
    }
}
