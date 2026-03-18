use std::collections::HashMap;

use async_trait::async_trait;
use moka::future::Cache;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::client::VaultClient;
use crate::config::VaultClientConfig;
use crate::error::VaultError;
use crate::secret::{Secret, SecretRotatedEvent};

#[derive(Deserialize)]
struct SecretResponse {
    path: String,
    data: HashMap<String, String>,
    version: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub struct HttpVaultClient {
    config: VaultClientConfig,
    http: reqwest::Client,
    cache: Cache<String, Secret>,
}

impl HttpVaultClient {
    /// 新しい HttpVaultClient を生成する。
    /// デフォルトタイムアウト30秒でHTTPクライアントを構築する。
    pub fn new(config: VaultClientConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.cache_max_capacity as u64)
            .time_to_live(config.cache_ttl)
            .build();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("HTTP client の作成に失敗");
        Self {
            config,
            http,
            cache,
        }
    }
}

#[async_trait]
impl VaultClient for HttpVaultClient {
    async fn get_secret(&self, path: &str) -> Result<Secret, VaultError> {
        if let Some(secret) = self.cache.get(path).await {
            return Ok(secret);
        }

        let url = format!("{}/api/v1/secrets/{}", self.config.server_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| VaultError::ServerError(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let body: SecretResponse = resp
                    .json()
                    .await
                    .map_err(|e| VaultError::ServerError(e.to_string()))?;
                let secret = Secret {
                    path: body.path,
                    data: body.data,
                    version: body.version,
                    created_at: body.created_at,
                };
                self.cache.insert(path.to_string(), secret.clone()).await;
                Ok(secret)
            }
            StatusCode::NOT_FOUND => Err(VaultError::NotFound(path.to_string())),
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                Err(VaultError::PermissionDenied(path.to_string()))
            }
            status => Err(VaultError::ServerError(format!(
                "unexpected status: {}",
                status
            ))),
        }
    }

    async fn get_secret_value(&self, path: &str, key: &str) -> Result<String, VaultError> {
        let secret = self.get_secret(path).await?;
        secret
            .data
            .get(key)
            .cloned()
            .ok_or_else(|| VaultError::NotFound(format!("{path}/{key}")))
    }

    async fn list_secrets(&self, path_prefix: &str) -> Result<Vec<String>, VaultError> {
        let url = format!(
            "{}/api/v1/secrets?prefix={}",
            self.config.server_url, path_prefix
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| VaultError::ServerError(e.to_string()))?;

        if resp.status() == StatusCode::OK {
            resp.json::<Vec<String>>()
                .await
                .map_err(|e| VaultError::ServerError(e.to_string()))
        } else {
            Err(VaultError::ServerError(format!(
                "list_secrets failed: {}",
                resp.status()
            )))
        }
    }

    async fn watch_secret(
        &self,
        path: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<SecretRotatedEvent>, VaultError> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let path = path.to_string();
        let url = format!("{}/api/v1/secrets/{}", self.config.server_url, path);
        let http = self.http.clone();
        let ttl = self.config.cache_ttl;

        tokio::spawn(async move {
            let mut last_version: Option<i64> = None;
            let mut interval = tokio::time::interval(ttl);
            loop {
                interval.tick().await;
                let resp = match http.get(&url).send().await {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                if resp.status() != StatusCode::OK {
                    continue;
                }
                let body: SecretResponse = match resp.json().await {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                if let Some(prev) = last_version {
                    if body.version != prev {
                        let event = SecretRotatedEvent {
                            path: body.path.clone(),
                            version: body.version,
                        };
                        if tx.send(event).await.is_err() {
                            break;
                        }
                    }
                }
                last_version = Some(body.version);
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_config(url: &str) -> VaultClientConfig {
        VaultClientConfig::new(url)
            .cache_ttl(Duration::from_secs(600))
            .cache_max_capacity(100)
    }

    fn secret_json(path_val: &str, version: i64) -> serde_json::Value {
        serde_json::json!({
            "path": path_val,
            "data": { "password": "s3cr3t", "username": "admin" },
            "version": version,
            "created_at": Utc::now().to_rfc3339()
        })
    }

    #[tokio::test]
    async fn test_get_secret_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/secrets/system/db"))
            .respond_with(ResponseTemplate::new(200).set_body_json(secret_json("system/db", 1)))
            .mount(&server)
            .await;

        let client = HttpVaultClient::new(make_config(&server.uri()));
        let secret = client.get_secret("system/db").await.unwrap();
        assert_eq!(secret.path, "system/db");
        assert_eq!(secret.data.get("password").unwrap(), "s3cr3t");
    }

    #[tokio::test]
    async fn test_get_secret_cache_hit() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/secrets/system/db"))
            .respond_with(ResponseTemplate::new(200).set_body_json(secret_json("system/db", 1)))
            .expect(1)
            .mount(&server)
            .await;

        let client = HttpVaultClient::new(make_config(&server.uri()));
        client.get_secret("system/db").await.unwrap();
        client.get_secret("system/db").await.unwrap();
        server.verify().await;
    }

    #[tokio::test]
    async fn test_get_secret_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/secrets/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = HttpVaultClient::new(make_config(&server.uri()));
        let err = client.get_secret("missing").await.unwrap_err();
        assert!(matches!(err, VaultError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_list_secrets_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/secrets"))
            .and(query_param("prefix", "system/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec!["system/db", "system/api"]))
            .mount(&server)
            .await;

        let client = HttpVaultClient::new(make_config(&server.uri()));
        let paths = client.list_secrets("system/").await.unwrap();
        assert_eq!(paths.len(), 2);
    }
}
