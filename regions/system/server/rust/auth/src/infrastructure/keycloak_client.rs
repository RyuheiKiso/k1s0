use std::sync::Arc;

use async_trait::async_trait;
use k1s0_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
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
    #[serde(default = "default_admin_realm")]
    pub admin_realm: String,
    #[serde(default = "default_admin_client_id")]
    pub admin_client_id: String,
    #[serde(default)]
    pub admin_username: String,
    #[serde(default)]
    pub admin_password: String,
}

fn default_admin_realm() -> String {
    "master".to_string()
}

fn default_admin_client_id() -> String {
    "admin-cli".to_string()
}

impl KeycloakConfig {
    fn uses_admin_password_grant(&self) -> bool {
        !self.admin_username.is_empty() && !self.admin_password.is_empty()
    }

    pub(crate) fn admin_token_url(&self) -> String {
        let realm = if self.uses_admin_password_grant() {
            self.admin_realm.as_str()
        } else {
            self.realm.as_str()
        };
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.base_url, realm
        )
    }

    pub(crate) fn admin_token_form(&self) -> Vec<(&'static str, String)> {
        if self.uses_admin_password_grant() {
            vec![
                ("grant_type", "password".to_string()),
                ("client_id", self.admin_client_id.clone()),
                ("username", self.admin_username.clone()),
                ("password", self.admin_password.clone()),
            ]
        } else {
            vec![
                ("grant_type", "client_credentials".to_string()),
                ("client_id", self.client_id.clone()),
                ("client_secret", self.client_secret.clone()),
            ]
        }
    }
}

/// CachedToken はキャッシュされた管理トークンを表す。
struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

/// KeycloakClient は Keycloak Admin API クライアント。
/// サーキットブレーカーにより、Keycloak 障害時の連鎖的な障害伝播を防止する。
pub struct KeycloakClient {
    config: KeycloakConfig,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
    /// Keycloak への HTTP リクエストを保護するサーキットブレーカー。
    /// 外部サービス障害時にリクエストを遮断し、システム全体の安定性を確保する。
    circuit_breaker: CircuitBreaker,
}

impl KeycloakClient {
    pub fn new(config: KeycloakConfig) -> Self {
        // サーキットブレーカー設定:
        // - failure_threshold: 5回連続失敗でOpen状態に遷移（Keycloakの一時的な遅延を許容）
        // - success_threshold: 3回連続成功でClosed状態に復帰（安定性を確認）
        // - timeout: 30秒後にHalfOpen状態で再試行（Keycloakの再起動時間を考慮）
        let cb_config = CircuitBreakerConfig::default();

        Self {
            config,
            // HTTP クライアントを構築する（デフォルト設定では失敗しない）
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest::Client の構築に失敗: デフォルト TLS バックエンド未対応"),
            admin_token: Arc::new(RwLock::new(None)),
            circuit_breaker: CircuitBreaker::new(cb_config),
        }
    }

    /// Keycloak のヘルスチェックを行う。
    /// サーキットブレーカーで保護し、Keycloak 停止時の不要なリクエストを抑制する。
    #[allow(dead_code)]
    pub async fn healthy(&self) -> anyhow::Result<()> {
        let url = format!("{}/realms/{}", self.config.base_url, self.config.realm);
        let http = &self.http_client;
        self.circuit_breaker
            .call(|| async {
                let resp = http.get(&url).send().await?;
                resp.error_for_status()?;
                Ok::<(), reqwest::Error>(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak health check failed: {}", e))
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

        let token_url = self.config.admin_token_url();
        let form = self.config.admin_token_form();

        // サーキットブレーカーでトークン取得リクエストを保護する。
        // Keycloak 障害時にトークン取得の連続失敗を防ぎ、キャッシュ期限切れ時の負荷集中を回避する。
        let http = &self.http_client;
        let body: serde_json::Value = self
            .circuit_breaker
            .call(|| async {
                let resp = http.post(&token_url).form(&form).send().await?;
                let body: serde_json::Value = resp.error_for_status()?.json().await?;
                Ok::<serde_json::Value, reqwest::Error>(body)
            })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak token request failed: {}", e))?;

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

        // サーキットブレーカーでユーザー取得リクエストを保護する
        let http = &self.http_client;
        let resp = self
            .circuit_breaker
            .call(|| async { http.get(&url).bearer_auth(&token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak find_by_id failed: {}", e))?;

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

        // サーキットブレーカーでユーザー一覧取得リクエストを保護する
        let http = &self.http_client;
        let resp = self
            .circuit_breaker
            .call(|| async { http.get(&url).bearer_auth(&token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak list users failed: {}", e))?;

        let kc_users: Vec<KeycloakUser> = resp.error_for_status()?.json().await?;

        // total count
        let count_url = format!(
            "{}/admin/realms/{}/users/count",
            self.config.base_url, self.config.realm
        );
        let count_token = self.get_admin_token().await?;
        // サーキットブレーカーでユーザー数取得リクエストを保護する
        let count_resp = self
            .circuit_breaker
            .call(|| async { http.get(&count_url).bearer_auth(&count_token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak user count failed: {}", e))?;
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
        // サーキットブレーカーでロール取得リクエストを保護する
        let http = &self.http_client;
        let resp = self
            .circuit_breaker
            .call(|| async { http.get(&realm_url).bearer_auth(&token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak get_roles failed: {}", e))?;

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
            user.attributes
                .get("department")
                .expect("department attribute should exist"),
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
        let config: KeycloakConfig =
            serde_yaml::from_str(yaml).expect("YAML deserialization should succeed");
        assert_eq!(config.base_url, "https://auth.k1s0.internal.example.com");
        assert_eq!(config.realm, "k1s0");
    }
}
