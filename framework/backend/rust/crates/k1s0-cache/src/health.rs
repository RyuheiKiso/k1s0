//! キャッシュヘルスチェック
//!
//! k1s0-health との統合を提供する。

#[cfg(feature = "health")]
use std::sync::Arc;
#[cfg(feature = "health")]
use k1s0_health::{ComponentHealth, HealthStatus};

#[cfg(all(feature = "redis", feature = "health"))]
use crate::client::CacheClient;

/// キャッシュヘルスチェッカー
#[cfg(all(feature = "redis", feature = "health"))]
pub struct CacheHealthChecker {
    client: Arc<CacheClient>,
    component_name: String,
}

#[cfg(all(feature = "redis", feature = "health"))]
impl CacheHealthChecker {
    /// 新しいヘルスチェッカーを作成
    pub fn new(client: Arc<CacheClient>) -> Self {
        Self {
            client,
            component_name: "redis".to_string(),
        }
    }

    /// コンポーネント名を設定
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.component_name = name.into();
        self
    }

    /// ヘルスチェックを実行
    pub async fn check(&self) -> ComponentHealth {
        match self.client.pool_status().connections {
            0 => {
                // No connections - try to connect
                match self.perform_ping().await {
                    Ok(_) => ComponentHealth::healthy(&self.component_name),
                    Err(e) => ComponentHealth::unhealthy(&self.component_name, e.to_string()),
                }
            }
            _ => {
                // Has connections - verify they work
                match self.perform_ping().await {
                    Ok(_) => {
                        let metrics = self.client.metrics().snapshot();
                        ComponentHealth::healthy(&self.component_name)
                            .with_metadata("hit_rate", format!("{:.2}%", metrics.hit_rate * 100.0))
                            .with_metadata("operations", metrics.operations.to_string())
                            .with_metadata("errors", metrics.errors.to_string())
                    }
                    Err(e) => ComponentHealth::degraded(&self.component_name, e.to_string()),
                }
            }
        }
    }

    /// PING コマンドを実行
    async fn perform_ping(&self) -> crate::error::CacheResult<()> {
        use crate::operations::CacheOperations;

        // Simple existence check as a health indicator
        // The actual PING is done at pool level
        let _: bool = self.client.exists("__health_check__").await?;
        Ok(())
    }

    /// k1s0-health の CheckFn として返す
    pub fn as_check_fn(self: Arc<Self>) -> k1s0_health::CheckFn {
        Box::new(move || {
            let checker = self.clone();
            Box::pin(async move { checker.check().await })
        })
    }
}

/// ヘルスチェック結果
#[derive(Debug, Clone)]
pub struct CacheHealthStatus {
    /// 正常かどうか
    pub healthy: bool,
    /// 接続数
    pub connections: u32,
    /// アイドル接続数
    pub idle_connections: u32,
    /// ヒット率
    pub hit_rate: f64,
    /// エラーメッセージ（あれば）
    pub error: Option<String>,
}

impl CacheHealthStatus {
    /// 正常なステータスを作成
    pub fn healthy(connections: u32, idle_connections: u32, hit_rate: f64) -> Self {
        Self {
            healthy: true,
            connections,
            idle_connections,
            hit_rate,
            error: None,
        }
    }

    /// 異常なステータスを作成
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            healthy: false,
            connections: 0,
            idle_connections: 0,
            hit_rate: 0.0,
            error: Some(error.into()),
        }
    }
}

#[cfg(all(feature = "redis", feature = "health"))]
impl From<CacheHealthStatus> for HealthStatus {
    fn from(status: CacheHealthStatus) -> Self {
        if status.healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_health_status_healthy() {
        let status = CacheHealthStatus::healthy(5, 3, 0.85);
        assert!(status.healthy);
        assert_eq!(status.connections, 5);
        assert_eq!(status.idle_connections, 3);
        assert!((status.hit_rate - 0.85).abs() < 0.01);
        assert!(status.error.is_none());
    }

    #[test]
    fn test_cache_health_status_unhealthy() {
        let status = CacheHealthStatus::unhealthy("Connection refused");
        assert!(!status.healthy);
        assert_eq!(status.connections, 0);
        assert_eq!(status.error, Some("Connection refused".to_string()));
    }
}
