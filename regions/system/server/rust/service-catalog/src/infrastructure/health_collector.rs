use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tracing::{info, warn};

use crate::domain::entity::health::{HealthState, HealthStatus};
use crate::domain::repository::service_repository::ServiceListFilters;
use crate::domain::repository::{HealthRepository, ServiceRepository};

/// SSRF 攻撃を防ぐため、URL のホストがプライベート IP アドレス範囲でないことを検証する。
/// HIGH-RUST-004 監査対応: ユーザー入力の URL に内部ネットワークへのアクセスを禁止する。
///
/// 拒否するアドレス範囲:
/// - 127.0.0.0/8 (loopback)
/// - 10.0.0.0/8 (private)
/// - 172.16.0.0/12 (private)
/// - 192.168.0.0/16 (private)
/// - 169.254.0.0/16 (link-local)
/// - `::1` (IPv6 loopback)
/// - `fc00::/7` (IPv6 unique local)
/// - `fe80::/10` (IPv6 link-local)
///
/// 許可するスキーム: http および https のみ
fn is_safe_url(url: &reqwest::Url) -> bool {
    // http/https のみ許可する（file://, ftp:// 等を拒否）
    match url.scheme() {
        "http" | "https" => {}
        _ => return false,
    }

    // ホスト名が IP アドレスとして解析できる場合はプライベートアドレスを拒否する
    if let Some(host) = url.host_str() {
        // IP アドレスの場合は直接検証する
        if let Ok(ip) = host.parse::<IpAddr>() {
            return !is_private_ip(&ip);
        }

        // "localhost" 等のホスト名も拒否する
        let lower = host.to_lowercase();
        if lower == "localhost" || lower.ends_with(".localhost") {
            return false;
        }

        // .local / .internal ドメインも拒否する（内部 DNS 解決の可能性）
        if lower.ends_with(".local") || lower.ends_with(".internal") {
            return false;
        }
    }

    true
}

/// IP アドレスがプライベートアドレス範囲かどうかを判定する。
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 127.0.0.0/8 (loopback)
            octets[0] == 127
            // 10.0.0.0/8 (private)
            || octets[0] == 10
            // 172.16.0.0/12 (private)
            || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
            // 192.168.0.0/16 (private)
            || (octets[0] == 192 && octets[1] == 168)
            // 169.254.0.0/16 (link-local)
            || (octets[0] == 169 && octets[1] == 254)
        }
        IpAddr::V6(ipv6) => {
            // ::1 (loopback)
            ipv6.is_loopback()
            // fc00::/7 (unique local)
            || (ipv6.segments()[0] & 0xfe00) == 0xfc00
            // fe80::/10 (link-local)
            || (ipv6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

/// `HealthCollectorConfig` はヘルスコレクターの設定を表す。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthCollectorConfig {
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for HealthCollectorConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval_secs(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

fn default_interval_secs() -> u64 {
    60
}

fn default_timeout_secs() -> u64 {
    5
}

/// `HealthCollector` はサービスの /healthz エンドポイントを定期的にポーリングするバックグラウンドタスク。
pub struct HealthCollector {
    service_repo: Arc<dyn ServiceRepository>,
    health_repo: Arc<dyn HealthRepository>,
    http_client: reqwest::Client,
    config: HealthCollectorConfig,
}

impl HealthCollector {
    /// 新しい `HealthCollector` を生成する。
    /// HTTP クライアントの構築に失敗した場合は Err を返す。
    pub fn new(
        service_repo: Arc<dyn ServiceRepository>,
        health_repo: Arc<dyn HealthRepository>,
        config: HealthCollectorConfig,
    ) -> anyhow::Result<Self> {
        // reqwest の Client 構築: TLS バックエンドが利用不可の場合はエラーとして伝播する
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| anyhow::anyhow!("HTTP クライアントの構築に失敗: {e}"))?;

        Ok(Self {
            service_repo,
            health_repo,
            http_client,
            config,
        })
    }

    /// バックグラウンドタスクとしてヘルスチェックループを開始する。
    pub async fn run(&self) {
        info!(
            interval_secs = self.config.interval_secs,
            "starting health collector"
        );

        loop {
            self.collect().await;
            tokio::time::sleep(Duration::from_secs(self.config.interval_secs)).await;
        }
    }

    async fn collect(&self) {
        // ヘルスコレクターは全テナントのサービスを対象とするため、システム用の空文字列を使用する。
        // RLS の set_config('app.current_tenant_id', '', true) は全テナントを返すポリシーが必要。
        let services = match self.service_repo.list("", ServiceListFilters::default()).await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "failed to list services for health check");
                return;
            }
        };

        for service in services {
            let healthcheck_url = match &service.healthcheck_url {
                Some(url) if !url.is_empty() => url.clone(),
                _ => continue,
            };

            // HIGH-RUST-004 監査対応: SSRF 攻撃を防ぐため、URL を解析してプライベートアドレスや
            // 安全でないスキームへのリクエストをスキップする。
            let parsed_url = match reqwest::Url::parse(&healthcheck_url) {
                Ok(u) => u,
                Err(e) => {
                    warn!(
                        service_id = %service.id,
                        url = %healthcheck_url,
                        error = %e,
                        "skipping health check: invalid URL"
                    );
                    continue;
                }
            };
            if !is_safe_url(&parsed_url) {
                warn!(
                    service_id = %service.id,
                    url = %healthcheck_url,
                    "skipping health check: URL targets private/internal network (SSRF protection)"
                );
                continue;
            }

            let start = std::time::Instant::now();
            let (state, message, response_time_ms) =
                match self.http_client.get(&healthcheck_url).send().await {
                    Ok(resp) => {
                        let elapsed = start.elapsed().as_millis() as i64;
                        if resp.status().is_success() {
                            (HealthState::Healthy, None, Some(elapsed))
                        } else {
                            (
                                HealthState::Degraded,
                                Some(format!("HTTP {}", resp.status())),
                                Some(elapsed),
                            )
                        }
                    }
                    Err(e) => {
                        let elapsed = start.elapsed().as_millis() as i64;
                        (HealthState::Unhealthy, Some(e.to_string()), Some(elapsed))
                    }
                };

            let health = HealthStatus {
                service_id: service.id,
                status: state,
                message,
                response_time_ms,
                checked_at: Utc::now(),
            };

            if let Err(e) = self.health_repo.upsert(&health).await {
                warn!(
                    service_id = %service.id,
                    error = %e,
                    "failed to upsert health status"
                );
            }
        }
    }
}
