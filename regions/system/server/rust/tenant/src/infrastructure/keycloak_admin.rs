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
    pub fn new(config: KeycloakAdminConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn config(&self) -> &KeycloakAdminConfig {
        &self.config
    }
}

#[cfg(test)]
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
