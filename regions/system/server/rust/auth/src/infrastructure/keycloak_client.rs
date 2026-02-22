use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::entity::user::{Pagination, Role, User, UserListResult, UserRoles};
use crate::domain::repository::UserRepository;

/// KeycloakConfig は Keycloak 接続の設定を表す。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

/// CachedToken はキャッシュされた管理トークンを表す。
struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

/// KeycloakClient は Keycloak Admin API クライアント。
pub struct KeycloakClient {
    config: KeycloakConfig,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
}

impl KeycloakClient {
    pub fn new(config: KeycloakConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            admin_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Keycloak のヘルスチェックを行う。
    pub async fn healthy(&self) -> anyhow::Result<()> {
        let url = format!("{}/realms/{}", self.config.base_url, self.config.realm);
        let resp = self.http_client.get(&url).send().await?;
        resp.error_for_status()?;
        Ok(())
    }

    /// Admin API トークンを取得する（キャッシュ付き）。
    async fn get_admin_token(&self) -> anyhow::Result<String> {
        let cache = self.admin_token.read().await;
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }
        drop(cache);

        let mut cache = self.admin_token.write().await;
        // 二重チェック
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }

        let token_url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.config.base_url, self.config.realm
        );

        let resp = self
            .http_client
            .post(&token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        let body: serde_json::Value = resp.error_for_status()?.json().await?;
        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing access_token in response"))?
            .to_string();
        let expires_in = body["expires_in"].as_i64().unwrap_or(300);

        *cache = Some(CachedToken {
            token: token.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(expires_in - 30), // 30 sec buffer
        });

        Ok(token)
    }
}

#[async_trait]
impl UserRepository for KeycloakClient {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.base_url, self.config.realm, user_id
        );

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("user not found: {}", user_id);
        }

        let kc_user: KeycloakUser = resp.error_for_status()?.json().await?;
        Ok(kc_user.into())
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult> {
        let token = self.get_admin_token().await?;
        let first = (page - 1) * page_size;
        let mut url = format!(
            "{}/admin/realms/{}/users?first={}&max={}",
            self.config.base_url, self.config.realm, first, page_size
        );

        if let Some(ref q) = search {
            url.push_str(&format!("&search={}", q));
        }
        if let Some(e) = enabled {
            url.push_str(&format!("&enabled={}", e));
        }

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        let kc_users: Vec<KeycloakUser> = resp.error_for_status()?.json().await?;

        // total count
        let count_url = format!(
            "{}/admin/realms/{}/users/count",
            self.config.base_url, self.config.realm
        );
        let count_resp = self
            .http_client
            .get(&count_url)
            .bearer_auth(&self.get_admin_token().await?)
            .send()
            .await?;
        let total_count: i64 = count_resp.error_for_status()?.json().await?;

        let users: Vec<User> = kc_users.into_iter().map(|u| u.into()).collect();
        let has_next = (page as i64 * page_size as i64) < total_count;

        Ok(UserListResult {
            users,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles> {
        let token = self.get_admin_token().await?;

        // Realm roles
        let realm_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.config.base_url, self.config.realm, user_id
        );
        let resp = self
            .http_client
            .get(&realm_url)
            .bearer_auth(&token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("user not found: {}", user_id);
        }

        let realm_roles: Vec<KeycloakRole> = resp.error_for_status()?.json().await?;

        Ok(UserRoles {
            user_id: user_id.to_string(),
            realm_roles: realm_roles.into_iter().map(|r| r.into()).collect(),
            client_roles: std::collections::HashMap::new(),
        })
    }
}

/// KeycloakUser は Keycloak Admin API のユーザー表現。
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeycloakUser {
    id: String,
    username: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    email_verified: bool,
    #[serde(default)]
    created_timestamp: i64,
    #[serde(default)]
    attributes: std::collections::HashMap<String, Vec<String>>,
}

fn default_true() -> bool {
    true
}

impl From<KeycloakUser> for User {
    fn from(kc: KeycloakUser) -> Self {
        let created_at = chrono::DateTime::from_timestamp_millis(kc.created_timestamp)
            .unwrap_or_else(chrono::Utc::now);

        User {
            id: kc.id,
            username: kc.username,
            email: kc.email,
            first_name: kc.first_name,
            last_name: kc.last_name,
            enabled: kc.enabled,
            email_verified: kc.email_verified,
            created_at,
            attributes: kc.attributes,
        }
    }
}

/// KeycloakRole は Keycloak Admin API のロール表現。
#[derive(Debug, serde::Deserialize)]
struct KeycloakRole {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
}

impl From<KeycloakRole> for Role {
    fn from(kc: KeycloakRole) -> Self {
        Role {
            id: kc.id,
            name: kc.name,
            description: kc.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycloak_user_to_domain_user() {
        let kc_user = KeycloakUser {
            id: "user-1".to_string(),
            username: "taro.yamada".to_string(),
            email: "taro@example.com".to_string(),
            first_name: "Taro".to_string(),
            last_name: "Yamada".to_string(),
            enabled: true,
            email_verified: true,
            created_timestamp: 1710000000000,
            attributes: std::collections::HashMap::from([(
                "department".to_string(),
                vec!["engineering".to_string()],
            )]),
        };

        let user: User = kc_user.into();
        assert_eq!(user.id, "user-1");
        assert_eq!(user.username, "taro.yamada");
        assert!(user.enabled);
        assert_eq!(
            user.attributes.get("department").unwrap(),
            &vec!["engineering".to_string()]
        );
    }

    #[test]
    fn test_keycloak_role_to_domain_role() {
        let kc_role = KeycloakRole {
            id: "role-1".to_string(),
            name: "sys_admin".to_string(),
            description: "System administrator".to_string(),
        };

        let role: Role = kc_role.into();
        assert_eq!(role.id, "role-1");
        assert_eq!(role.name, "sys_admin");
    }

    #[test]
    fn test_keycloak_config_deserialization() {
        let yaml = r#"
base_url: "https://auth.k1s0.internal.example.com"
realm: "k1s0"
client_id: "auth-server"
client_secret: "secret"
"#;
        let config: KeycloakConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.base_url, "https://auth.k1s0.internal.example.com");
        assert_eq!(config.realm, "k1s0");
    }
}
