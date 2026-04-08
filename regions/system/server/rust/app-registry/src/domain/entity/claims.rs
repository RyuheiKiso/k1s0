use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Claims は JWT トークンの Claims を表す。
/// auth server の Claims と同一構造。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub jti: String,
    #[serde(default)]
    pub typ: String,
    #[serde(default)]
    pub azp: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub preferred_username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub realm_access: RealmAccess,
    #[serde(default)]
    pub resource_access: HashMap<String, ResourceAccess>,
    #[serde(default)]
    pub tier_access: Vec<String>,
    /// RLS テナント分離のために使用するテナント ID。
    /// Keycloak のカスタムクレームとして付与される。JWT に含まれない場合は空文字列。
    #[serde(default)]
    pub tenant_id: String,
}

/// `RealmAccess` は Keycloak の `realm_access` クレームを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RealmAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}

/// `ResourceAccess` は Keycloak の `resource_access` クレームを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourceAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}
