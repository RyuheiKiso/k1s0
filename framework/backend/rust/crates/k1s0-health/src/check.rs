//! ヘルスチェック型
//!
//! 各コンポーネントのヘルスチェック結果を表現する。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// ヘルスステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// 正常
    Healthy,
    /// 劣化（一部機能が利用不可）
    Degraded,
    /// 異常
    Unhealthy,
}

impl HealthStatus {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }

    /// HTTP ステータスコードに変換
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            Self::Healthy => 200,
            Self::Degraded => 200,
            Self::Unhealthy => 503,
        }
    }

    /// 正常かどうか
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// リクエストを受け付け可能かどうか
    pub fn is_serving(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// 2つのステータスをマージ（悪い方を採用）
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unhealthy, _) | (_, Self::Unhealthy) => Self::Unhealthy,
            (Self::Degraded, _) | (_, Self::Degraded) => Self::Degraded,
            _ => Self::Healthy,
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Healthy
    }
}

/// 個別コンポーネントのヘルスチェック結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// コンポーネント名
    pub name: String,
    /// ステータス
    pub status: HealthStatus,
    /// チェックにかかった時間（ミリ秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// エラーメッセージ（異常時のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 追加のメタデータ
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl ComponentHealth {
    /// 正常なヘルス結果を作成
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            latency_ms: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// 劣化したヘルス結果を作成
    pub fn degraded(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            latency_ms: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    /// 異常なヘルス結果を作成
    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    /// レイテンシを設定
    pub fn with_latency(mut self, duration: Duration) -> Self {
        self.latency_ms = Some(duration.as_millis() as u64);
        self
    }

    /// レイテンシをミリ秒で設定
    pub fn with_latency_ms(mut self, ms: u64) -> Self {
        self.latency_ms = Some(ms);
        self
    }

    /// メタデータを追加
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// サービス全体のヘルスレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// 全体のステータス
    pub status: HealthStatus,
    /// サービス名
    pub service: String,
    /// バージョン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// 各コンポーネントのヘルス
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<ComponentHealth>,
    /// チェック実行時刻（Unix タイムスタンプ）
    pub timestamp: i64,
}

impl HealthResponse {
    /// 新しいヘルスレスポンスを作成
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            service: service.into(),
            version: None,
            components: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        }
    }

    /// バージョンを設定
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// コンポーネントのヘルスを追加
    pub fn add_component(&mut self, component: ComponentHealth) {
        self.status = self.status.merge(component.status);
        self.components.push(component);
    }

    /// コンポーネントのヘルスを追加（ビルダー形式）
    pub fn with_component(mut self, component: ComponentHealth) -> Self {
        self.add_component(component);
        self
    }

    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// JSON 文字列に変換（整形版）
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// シンプルなヘルスレスポンス（Kubernetes readiness/liveness用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleHealthResponse {
    /// ステータス
    pub status: HealthStatus,
}

impl SimpleHealthResponse {
    /// 正常なレスポンスを作成
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
        }
    }

    /// 異常なレスポンスを作成
    pub fn unhealthy() -> Self {
        Self {
            status: HealthStatus::Unhealthy,
        }
    }

    /// ステータスから作成
    pub fn from_status(status: HealthStatus) -> Self {
        Self { status }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
    }

    #[test]
    fn test_health_status_http_code() {
        assert_eq!(HealthStatus::Healthy.to_http_status_code(), 200);
        assert_eq!(HealthStatus::Degraded.to_http_status_code(), 200);
        assert_eq!(HealthStatus::Unhealthy.to_http_status_code(), 503);
    }

    #[test]
    fn test_health_status_is_serving() {
        assert!(HealthStatus::Healthy.is_serving());
        assert!(HealthStatus::Degraded.is_serving());
        assert!(!HealthStatus::Unhealthy.is_serving());
    }

    #[test]
    fn test_health_status_merge() {
        assert_eq!(
            HealthStatus::Healthy.merge(HealthStatus::Healthy),
            HealthStatus::Healthy
        );
        assert_eq!(
            HealthStatus::Healthy.merge(HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            HealthStatus::Degraded.merge(HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_component_health_healthy() {
        let health = ComponentHealth::healthy("database")
            .with_latency_ms(50)
            .with_metadata("version", "14.2");

        assert_eq!(health.name, "database");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency_ms, Some(50));
        assert!(health.error.is_none());
        assert_eq!(health.metadata.get("version"), Some(&"14.2".to_string()));
    }

    #[test]
    fn test_component_health_unhealthy() {
        let health = ComponentHealth::unhealthy("database", "connection refused");

        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.error, Some("connection refused".to_string()));
    }

    #[test]
    fn test_health_response() {
        let response = HealthResponse::new("my-service")
            .with_version("1.0.0")
            .with_component(ComponentHealth::healthy("database"))
            .with_component(ComponentHealth::degraded("cache", "high latency"));

        assert_eq!(response.service, "my-service");
        assert_eq!(response.version, Some("1.0.0".to_string()));
        assert_eq!(response.status, HealthStatus::Degraded);
        assert_eq!(response.components.len(), 2);
    }

    #[test]
    fn test_health_response_json() {
        let response = HealthResponse::new("test-service").with_version("1.0.0");

        let json = response.to_json().unwrap();
        assert!(json.contains("test-service"));
        assert!(json.contains("healthy"));
    }

    #[test]
    fn test_simple_health_response() {
        let healthy = SimpleHealthResponse::healthy();
        assert_eq!(healthy.status, HealthStatus::Healthy);

        let unhealthy = SimpleHealthResponse::unhealthy();
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    }
}
