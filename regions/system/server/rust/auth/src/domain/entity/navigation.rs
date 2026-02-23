use serde::{Deserialize, Serialize};

/// navigation.yaml のデシリアライズ用構造体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationConfig {
    pub version: u32,
    #[serde(default)]
    pub guards: Vec<NavigationGuard>,
    pub routes: Vec<NavigationRoute>,
}

/// ルートガード定義。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationGuard {
    pub id: String,
    #[serde(rename = "type")]
    pub guard_type: String,
    pub redirect_to: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

/// ルーティング定義。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationRoute {
    pub id: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    #[serde(default)]
    pub guards: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_to: Option<String>,
    #[serde(default)]
    pub children: Vec<NavigationRoute>,
    #[serde(default)]
    pub params: Vec<NavigationParam>,
}

/// ルートパラメータ定義。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}
