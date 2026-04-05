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
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// タイムアウト設定（ミリ秒）。health_check のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl NavigationGrpcClient {
    /// バックエンド設定からクライアントを生成する。
    /// connect_lazy() により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: NavigationServiceClient::new(channel),
            address: cfg.address.clone(),
            timeout_ms: cfg.timeout_ms,
        })
    }

    /// gRPC Health Check Protocol を使ってサービスの疎通確認を行う。
    /// Bearer token なしで接続できるため readyz ヘルスチェックに適している。
    /// tonic-health サービスが登録されているサーバーに対して Check RPC を送信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let channel = Channel::from_shared(self.address.clone())?
            .timeout(Duration::from_millis(self.timeout_ms))
            .connect_lazy();
        let mut health_client = tonic_health::pb::health_client::HealthClient::new(channel);
        let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
            service: "k1s0.system.navigation.v1.NavigationService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("navigation gRPC Health Check 失敗: {}", e))?;
        Ok(())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_navigation(&self, bearer_token: &str) -> anyhow::Result<Navigation> {
        let request =
            tonic::Request::new(proto::k1s0::system::navigation::v1::GetNavigationRequest {
                bearer_token: bearer_token.to_owned(),
            });

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
