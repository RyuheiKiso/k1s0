use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::service::RolePermissionSource;
use crate::infrastructure::keycloak_client::KeycloakConfig;

struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct KeycloakRolePermissionSource {
    config: KeycloakConfig,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
    token_cache_ttl_secs: u64,
}

impl KeycloakRolePermissionSource {
    pub fn new(config: KeycloakConfig, token_cache_ttl_secs: u64) -> Self {
        Self {
            config,
            // HTTP クライアントを構築する（デフォルト設定では失敗しない）
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest::Client の構築に失敗: デフォルト TLS バックエンド未対応"),
            admin_token: Arc::new(RwLock::new(None)),
            token_cache_ttl_secs,
        }
    }

    async fn get_admin_token(&self) -> anyhow::Result<String> {
        let cache = self.admin_token.read().await;
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }
        drop(cache);

        let mut cache = self.admin_token.write().await;
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }

        let token_url = self.config.admin_token_url();
        let form = self.config.admin_token_form();
        let resp = self.http_client.post(&token_url).form(&form).send().await?;
        let body: serde_json::Value = resp.error_for_status()?.json().await?;
        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing access_token in keycloak response"))?
            .to_string();
        let expires_in = body["expires_in"].as_i64().unwrap_or(300);
        let expires_in = std::cmp::min(expires_in, self.token_cache_ttl_secs as i64);
        let cache_secs = if expires_in > 30 { expires_in - 30 } else { 1 };
        *cache = Some(CachedToken {
            token: token.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(cache_secs),
        });
        Ok(token)
    }

    async fn fetch_roles(&self) -> anyhow::Result<Vec<KeycloakRole>> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/roles",
            self.config.base_url, self.config.realm
        );
        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;
        Ok(resp.error_for_status()?.json().await?)
    }

    async fn fetch_role_composites(&self, role_name: &str) -> anyhow::Result<Vec<KeycloakRole>> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/roles/{}/composites",
            self.config.base_url, self.config.realm, role_name
        );
        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;
        Ok(resp.error_for_status()?.json().await?)
    }
}

#[async_trait]
impl RolePermissionSource for KeycloakRolePermissionSource {
    async fn fetch_role_permissions(&self) -> anyhow::Result<HashMap<String, Vec<String>>> {
        let roles = self.fetch_roles().await?;
        let mut table = HashMap::new();

        for role in roles {
            let mut permissions = permissions_from_role(&role);

            if role.composite {
                if let Ok(composites) = self.fetch_role_composites(&role.name).await {
                    for composite in composites {
                        permissions.extend(permissions_from_role(&composite));
                        if let Some(p) = normalize_permission(&composite.name) {
                            permissions.push(p);
                        }
                    }
                }
            }

            if permissions.is_empty() {
                permissions.extend(default_permissions_for_role(&role.name));
            }

            permissions.sort();
            permissions.dedup();
            table.insert(role.name, permissions);
        }

        Ok(table)
    }
}

fn permissions_from_role(role: &KeycloakRole) -> Vec<String> {
    let mut permissions = Vec::new();

    for key in ["permissions", "permission"] {
        if let Some(values) = role.attributes.get(key) {
            permissions.extend(values.iter().filter_map(|v| normalize_permission(v)));
        }
    }

    permissions
}

fn normalize_permission(raw: &str) -> Option<String> {
    let candidate = raw.trim().to_ascii_lowercase();
    if candidate.is_empty() {
        return None;
    }
    if candidate == "*" || candidate == "*:*" {
        return Some("*:*".to_string());
    }
    if candidate.contains(':') {
        return Some(candidate);
    }
    if candidate.contains('/') {
        return Some(candidate.replace('/', ":"));
    }
    if candidate.starts_with("sys_") {
        return None;
    }

    let parts: Vec<&str> = candidate.split('_').collect();
    if parts.len() < 2 {
        return None;
    }
    let action = parts[parts.len() - 1];
    if !matches!(
        action,
        "read" | "write" | "delete" | "admin" | "create" | "update" | "execute" | "cancel"
    ) {
        return None;
    }
    let resource = parts[..parts.len() - 1].join("_");
    if resource.is_empty() {
        return None;
    }
    Some(format!("{}:{}", resource, action))
}

fn default_permissions_for_role(role: &str) -> Vec<String> {
    match role {
        "sys_admin" => vec!["*:*".to_string()],
        "sys_operator" => vec![
            "users:read".to_string(),
            "users:write".to_string(),
            "auth_config:read".to_string(),
            "auth_config:write".to_string(),
            "audit_logs:read".to_string(),
            "audit_logs:write".to_string(),
            "api_keys:read".to_string(),
            "api_keys:write".to_string(),
        ],
        "sys_auditor" => vec![
            "users:read".to_string(),
            "auth_config:read".to_string(),
            "audit_logs:read".to_string(),
            "api_keys:read".to_string(),
        ],
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct KeycloakRole {
    name: String,
    #[serde(default)]
    attributes: HashMap<String, Vec<String>>,
    #[serde(default)]
    composite: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_permission() {
        assert_eq!(
            normalize_permission("users:read"),
            Some("users:read".to_string())
        );
        assert_eq!(
            normalize_permission("users/read"),
            Some("users:read".to_string())
        );
        assert_eq!(
            normalize_permission("users_delete"),
            Some("users:delete".to_string())
        );
        assert_eq!(normalize_permission("sys_admin"), None);
    }
}
