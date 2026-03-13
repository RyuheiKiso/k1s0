use async_graphql::{Enum, SimpleObject};

#[derive(Debug, Clone, SimpleObject)]
pub struct NavigationRoute {
    pub id: String,
    pub path: String,
    pub component_id: Option<String>,
    pub guard_ids: Vec<String>,
    pub children: Vec<NavigationRoute>,
    pub transition: Option<TransitionConfig>,
    pub params: Vec<RouteParam>,
    pub redirect_to: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct NavigationGuard {
    pub id: String,
    pub guard_type: GuardType,
    pub redirect_to: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TransitionConfig {
    pub transition_type: TransitionType,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RouteParam {
    pub name: String,
    pub param_type: ParamType,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Navigation {
    pub routes: Vec<NavigationRoute>,
    pub guards: Vec<NavigationGuard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum GuardType {
    Unspecified,
    AuthRequired,
    RoleRequired,
    RedirectIfAuthenticated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TransitionType {
    Unspecified,
    Fade,
    Slide,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum ParamType {
    Unspecified,
    StringType,
    IntType,
    UuidType,
}
