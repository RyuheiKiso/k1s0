use std::time::Duration;

use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{
    GuardType, Navigation, NavigationGuard, NavigationRoute, ParamType, RouteParam,
    TransitionConfig, TransitionType,
};
use crate::infrastructure::config::BackendConfig;

#[allow(dead_code)]
pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod navigation {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.navigation.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::navigation::v1::navigation_service_client::NavigationServiceClient;
use proto::k1s0::system::navigation::v1::{
    Guard as ProtoGuard, Param as ProtoParam, Route as ProtoRoute,
    TransitionConfig as ProtoTransitionConfig,
};

pub struct NavigationGrpcClient {
    client: NavigationServiceClient<Channel>,
}

impl NavigationGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: NavigationServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_navigation(&self, bearer_token: &str) -> anyhow::Result<Navigation> {
        let request = tonic::Request::new(
            proto::k1s0::system::navigation::v1::GetNavigationRequest {
                bearer_token: bearer_token.to_owned(),
            },
        );

        let resp = self
            .client
            .clone()
            .get_navigation(request)
            .await
            .map_err(|e| anyhow::anyhow!("NavigationService.GetNavigation failed: {}", e))?
            .into_inner();

        let routes = resp.routes.into_iter().map(route_from_proto).collect();
        let guards = resp.guards.into_iter().map(guard_from_proto).collect();

        Ok(Navigation { routes, guards })
    }
}

fn route_from_proto(r: ProtoRoute) -> NavigationRoute {
    NavigationRoute {
        id: r.id,
        path: r.path,
        component_id: optional_string(r.component_id),
        guard_ids: r.guard_ids,
        children: r.children.into_iter().map(route_from_proto).collect(),
        transition: r.transition.map(transition_from_proto),
        params: r.params.into_iter().map(param_from_proto).collect(),
        redirect_to: non_empty_string(r.redirect_to),
    }
}

fn guard_from_proto(g: ProtoGuard) -> NavigationGuard {
    NavigationGuard {
        id: g.id,
        guard_type: guard_type_from_i32(g.r#type),
        redirect_to: non_empty_string(g.redirect_to),
        roles: g.roles,
    }
}

fn transition_from_proto(t: ProtoTransitionConfig) -> TransitionConfig {
    TransitionConfig {
        transition_type: transition_type_from_i32(t.r#type),
        duration_ms: t.duration_ms,
    }
}

fn param_from_proto(p: ProtoParam) -> RouteParam {
    RouteParam {
        name: p.name,
        param_type: param_type_from_i32(p.r#type),
    }
}

fn guard_type_from_i32(v: i32) -> GuardType {
    match v {
        1 => GuardType::AuthRequired,
        2 => GuardType::RoleRequired,
        3 => GuardType::RedirectIfAuthenticated,
        _ => GuardType::Unspecified,
    }
}

fn transition_type_from_i32(v: i32) -> TransitionType {
    match v {
        1 => TransitionType::Fade,
        2 => TransitionType::Slide,
        3 => TransitionType::Modal,
        _ => TransitionType::Unspecified,
    }
}

fn param_type_from_i32(v: i32) -> ParamType {
    match v {
        1 => ParamType::StringType,
        2 => ParamType::IntType,
        3 => ParamType::UuidType,
        _ => ParamType::Unspecified,
    }
}

fn optional_string(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.is_empty())
}

fn non_empty_string(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
