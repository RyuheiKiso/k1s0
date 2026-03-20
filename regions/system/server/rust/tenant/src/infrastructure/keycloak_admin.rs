use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakAdminConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

#[allow(dead_code)]
pub struct KeycloakAdminClient {
    config: KeycloakAdminConfig,
    http_client: reqwest::Client,
}

impl KeycloakAdminClient {
    /// 新しい KeycloakAdminClient を生成する。
    /// デフォルトタイムアウト30秒でHTTPクライアントを構築する。
    /// TLS バックエンドの初期化に失敗した場合は Err を返す。
    pub fn new(config: KeycloakAdminConfig) -> anyhow::Result<Self> {
        // reqwest の Client 構築: TLS バックエンドが利用不可の場合はエラーとして伝播する
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow::anyhow!("HTTP クライアントの構築に失敗: {}", e))?;
        Ok(Self {
            config,
            http_client,
        })
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &KeycloakAdminConfig {
        &self.config
    }

    async fn admin_token(&self) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token_url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.config.base_url.trim_end_matches('/'),
            self.config.realm
        );
        let response = self
            .http_client
            .post(&token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("failed to get keycloak admin token: {} {}", status, body);
        }

        let token: TokenResponse = response.json().await?;
        Ok(token.access_token)
    }
}

#[async_trait]
pub trait KeycloakAdmin: Send + Sync {
    async fn create_realm(&self, realm_name: &str) -> Result<()>;
    async fn delete_realm(&self, realm_name: &str) -> Result<()>;
    #[allow(dead_code)]
    async fn add_user(&self, realm_name: &str, user_id: &str) -> Result<()>;
    #[allow(dead_code)]
    async fn remove_user(&self, realm_name: &str, user_id: &str) -> Result<()>;
}

#[async_trait]
impl KeycloakAdmin for KeycloakAdminClient {
    async fn create_realm(&self, realm_name: &str) -> Result<()> {
        let token = self.admin_token().await?;
        let url = format!(
            "{}/admin/realms",
            self.config.base_url.trim_end_matches('/')
        );
        let response = self
            .http_client
            .post(&url)
            .bearer_auth(token)
            .json(&serde_json::json!({
                "realm": realm_name,
                "enabled": true
            }))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 409 {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "failed to create keycloak realm '{}': {} {}",
            realm_name,
            status,
            body
        );
    }

    async fn delete_realm(&self, realm_name: &str) -> Result<()> {
        let token = self.admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}",
            self.config.base_url.trim_end_matches('/'),
            realm_name
        );
        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 404 {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "failed to delete keycloak realm '{}': {} {}",
            realm_name,
            status,
            body
        );
    }

    async fn add_user(&self, realm_name: &str, user_id: &str) -> Result<()> {
        let token = self.admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.base_url.trim_end_matches('/'),
            realm_name,
            user_id
        );
        let response = self
            .http_client
            .put(&url)
            .bearer_auth(token)
            .json(&serde_json::json!({ "enabled": true }))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 409 {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "failed to add user '{}' to realm '{}': {} {}",
            user_id,
            realm_name,
            status,
            body
        );
    }

    async fn remove_user(&self, realm_name: &str, user_id: &str) -> Result<()> {
        let token = self.admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.base_url.trim_end_matches('/'),
            realm_name,
            user_id
        );
        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 404 {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "failed to remove user '{}' from realm '{}': {} {}",
            user_id,
            realm_name,
            status,
            body
        );
    }
}

pub struct NoopKeycloakAdmin;

#[async_trait]
impl KeycloakAdmin for NoopKeycloakAdmin {
    async fn create_realm(&self, realm_name: &str) -> Result<()> {
        tracing::debug!(realm_name = %realm_name, "noop keycloak create_realm");
        Ok(())
    }

    async fn delete_realm(&self, realm_name: &str) -> Result<()> {
        tracing::debug!(realm_name = %realm_name, "noop keycloak delete_realm");
        Ok(())
    }

    async fn add_user(&self, realm_name: &str, user_id: &str) -> Result<()> {
        tracing::debug!(realm_name = %realm_name, user_id = %user_id, "noop keycloak add_user");
        Ok(())
    }

    async fn remove_user(&self, realm_name: &str, user_id: &str) -> Result<()> {
        tracing::debug!(realm_name = %realm_name, user_id = %user_id, "noop keycloak remove_user");
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_keycloak_admin_config_deserialization() {
        let yaml = r#"
base_url: "http://localhost:8080"
realm: "master"
client_id: "admin-cli"
client_secret: "secret"
"#;
        let config: KeycloakAdminConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.realm, "master");
        assert_eq!(config.client_id, "admin-cli");
    }

    #[test]
    fn test_keycloak_admin_client_creation() {
        let config = KeycloakAdminConfig {
            base_url: "http://localhost:8080".to_string(),
            realm: "master".to_string(),
            client_id: "admin-cli".to_string(),
            client_secret: "secret".to_string(),
        };
        let client = KeycloakAdminClient::new(config);
        assert_eq!(client.config().base_url, "http://localhost:8080");
    }
}
