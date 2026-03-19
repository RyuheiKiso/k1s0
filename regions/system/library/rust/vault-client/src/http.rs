use std::collections::HashMap;

use async_trait::async_trait;
use k1s0_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use moka::future::Cache;
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::warn;

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

/// Vault サーバーへの HTTP クライアント。
/// サーキットブレーカーにより、Vault 障害時の連鎖的な障害伝播を防止する。
pub struct HttpVaultClient {
    config: VaultClientConfig,
    http: reqwest::Client,
    cache: Cache<String, Secret>,
    /// Vault への HTTP リクエストを保護するサーキットブレーカー。
    /// 外部サービス障害時にリクエストを遮断し、システム全体の安定性を確保する。
    circuit_breaker: CircuitBreaker,
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

        // サーキットブレーカー設定:
        // - failure_threshold: 5回連続失敗でOpen状態に遷移（Vault の一時的な遅延を許容）
        // - success_threshold: 3回連続成功でClosed状態に復帰（安定性を確認）
        // - timeout: 30秒後にHalfOpen状態で再試行（Vault の再起動・フェイルオーバー時間を考慮）
        let cb_config = CircuitBreakerConfig::default();

        Self {
            config,
            http,
            cache,
            circuit_breaker: CircuitBreaker::new(cb_config),
        }
    }

    /// Vault 接続の必須設定を返す。
    pub fn vault_required(&self) -> bool {
        self.config.vault_required
    }

    /// サーキットブレーカーまたは HTTP リクエストのエラーを vault_required フラグに
    /// 基づいて処理する。
    ///
    /// フォールバック戦略:
    /// - vault_required=true (本番): エラーをそのまま ServerError として返す。
    ///   シークレットなしでの起動はセキュリティリスクとなるため、致命的エラーとして扱う。
    /// - vault_required=false (開発): 接続不可系エラーの場合は警告ログを出力し
    ///   ConnectionUnavailable を返す。呼び出し元はこのバリアントを判定して
    ///   ローカル設定へのフォールバックを実行できる。
    /// - 認証エラーやアプリケーションレベルのエラーは vault_required に関わらず
    ///   常にそのまま返す（設定ミスは黙殺すべきでないため）。
    fn handle_connection_error(&self, error_msg: String) -> VaultError {
        if !self.config.vault_required {
            warn!(
                vault_url = %self.config.server_url,
                error = %error_msg,
                "Vault への接続に失敗しました。vault_required=false のためローカル設定で続行します。\
                 本番環境では vault_required=true に設定してください。"
            );
            VaultError::ConnectionUnavailable(error_msg)
        } else {
            VaultError::ServerError(error_msg)
        }
    }
}

#[async_trait]
impl VaultClient for HttpVaultClient {
    async fn get_secret(&self, path: &str) -> Result<Secret, VaultError> {
        // キャッシュにヒットした場合はサーキットブレーカーを経由せず即座に返す
        if let Some(secret) = self.cache.get(path).await {
            return Ok(secret);
        }

        let url = format!("{}/api/v1/secrets/{}", self.config.server_url, path);
        // サーキットブレーカーでシークレット取得リクエストを保護する。
        // Vault 障害時の連続リクエストを遮断し、キャッシュミス時の負荷集中を防ぐ。
        // 接続失敗時は vault_required フラグに基づいてフォールバック判定を行う。
        let http = &self.http;
        let resp = self
            .circuit_breaker
            .call(|| async {
                http.get(&url)
                    .send()
                    .await
            })
            .await
            .map_err(|e| self.handle_connection_error(format!("circuit breaker: {}", e)))?;

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
        // サーキットブレーカーでシークレット一覧取得リクエストを保護する。
        // 接続失敗時は vault_required フラグに基づいてフォールバック判定を行う。
        let http = &self.http;
        let resp = self
            .circuit_breaker
            .call(|| async {
                http.get(&url)
                    .send()
                    .await
            })
            .await
            .map_err(|e| self.handle_connection_error(format!("circuit breaker: {}", e)))?;

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
