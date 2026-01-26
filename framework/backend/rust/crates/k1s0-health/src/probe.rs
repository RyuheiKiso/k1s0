//! プローブ（Kubernetes readiness/liveness）
//!
//! Kubernetes の readiness/liveness プローブに対応する
//! ヘルスチェック機能を提供する。

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::check::{ComponentHealth, HealthResponse, HealthStatus};

/// ヘルスチェック関数の型
pub type CheckFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ComponentHealth> + Send>> + Send + Sync>;

/// ヘルスチェッカー
///
/// 各コンポーネントのヘルスチェックを行う。
pub struct HealthChecker {
    name: String,
    check_fn: CheckFn,
}

impl HealthChecker {
    /// 新しいヘルスチェッカーを作成
    pub fn new<F, Fut>(name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ComponentHealth> + Send + 'static,
    {
        let name = name.into();
        Self {
            name,
            check_fn: Box::new(move || Box::pin(check_fn())),
        }
    }

    /// 同期的なチェック関数からヘルスチェッカーを作成
    pub fn from_sync<F>(name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> ComponentHealth + Send + Sync + 'static,
    {
        let name = name.into();
        Self {
            name,
            check_fn: Box::new(move || {
                let result = check_fn();
                Box::pin(async move { result })
            }),
        }
    }

    /// コンポーネント名を取得
    pub fn name(&self) -> &str {
        &self.name
    }

    /// ヘルスチェックを実行
    pub async fn check(&self) -> ComponentHealth {
        (self.check_fn)().await
    }
}

/// プローブ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeType {
    /// Readiness プローブ（トラフィックを受け入れ可能か）
    Readiness,
    /// Liveness プローブ（プロセスが生きているか）
    Liveness,
    /// Startup プローブ（起動が完了したか）
    Startup,
}

impl ProbeType {
    /// エンドポイントパスを取得
    pub fn path(&self) -> &'static str {
        match self {
            Self::Readiness => "/readyz",
            Self::Liveness => "/livez",
            Self::Startup => "/startupz",
        }
    }

    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Readiness => "readiness",
            Self::Liveness => "liveness",
            Self::Startup => "startup",
        }
    }
}

/// プローブ設定
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    /// タイムアウト
    pub timeout: Duration,
    /// 詳細情報を含めるか
    pub include_details: bool,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            include_details: false,
        }
    }
}

impl ProbeConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// タイムアウトを設定
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 詳細情報を含めるかを設定
    pub fn with_details(mut self, include: bool) -> Self {
        self.include_details = include;
        self
    }
}

/// Readiness 状態管理
///
/// サービスの readiness 状態を管理する。
/// graceful shutdown 時に false に設定して新規リクエストを拒否する。
#[derive(Debug)]
pub struct ReadinessState {
    ready: AtomicBool,
}

impl ReadinessState {
    /// 新しい状態を作成（初期状態: not ready）
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
        }
    }

    /// ready 状態で作成
    pub fn ready() -> Self {
        Self {
            ready: AtomicBool::new(true),
        }
    }

    /// ready 状態かどうか
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    /// ready 状態に設定
    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
    }

    /// not ready 状態に設定
    pub fn set_not_ready(&self) {
        self.ready.store(false, Ordering::SeqCst);
    }

    /// ヘルスチェック結果を取得
    pub fn check(&self) -> HealthStatus {
        if self.is_ready() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }
}

impl Default for ReadinessState {
    fn default() -> Self {
        Self::new()
    }
}

/// プローブハンドラー
///
/// 複数のヘルスチェッカーを管理し、プローブリクエストに応答する。
pub struct ProbeHandler {
    service_name: String,
    version: Option<String>,
    checkers: Vec<HealthChecker>,
    readiness: Arc<ReadinessState>,
    config: ProbeConfig,
}

impl ProbeHandler {
    /// 新しいハンドラーを作成
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            version: None,
            checkers: Vec::new(),
            readiness: Arc::new(ReadinessState::ready()),
            config: ProbeConfig::default(),
        }
    }

    /// バージョンを設定
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// ヘルスチェッカーを追加
    pub fn add_checker(&mut self, checker: HealthChecker) {
        self.checkers.push(checker);
    }

    /// ヘルスチェッカーを追加（ビルダー形式）
    pub fn with_checker(mut self, checker: HealthChecker) -> Self {
        self.add_checker(checker);
        self
    }

    /// Readiness 状態を設定
    pub fn with_readiness(mut self, readiness: Arc<ReadinessState>) -> Self {
        self.readiness = readiness;
        self
    }

    /// 設定を設定
    pub fn with_config(mut self, config: ProbeConfig) -> Self {
        self.config = config;
        self
    }

    /// Readiness 状態への参照を取得
    pub fn readiness(&self) -> Arc<ReadinessState> {
        Arc::clone(&self.readiness)
    }

    /// Liveness プローブを実行
    ///
    /// プロセスが生きているかどうかだけをチェックする。
    /// 常に成功を返す（プロセスが動作している証拠）。
    pub fn liveness(&self) -> HealthResponse {
        let mut response = HealthResponse::new(&self.service_name);
        if let Some(ref version) = self.version {
            response = response.with_version(version);
        }
        response
    }

    /// Readiness プローブを実行
    ///
    /// トラフィックを受け入れ可能かどうかをチェックする。
    pub async fn readiness_check(&self) -> HealthResponse {
        let mut response = HealthResponse::new(&self.service_name);
        if let Some(ref version) = self.version {
            response = response.with_version(version);
        }

        // Readiness 状態をチェック
        if !self.readiness.is_ready() {
            response.status = HealthStatus::Unhealthy;
            return response;
        }

        // 各コンポーネントをチェック
        for checker in &self.checkers {
            let health = checker.check().await;
            response.add_component(health);
        }

        response
    }

    /// Startup プローブを実行
    ///
    /// サービスの起動が完了したかどうかをチェックする。
    pub fn startup(&self) -> HealthResponse {
        let mut response = HealthResponse::new(&self.service_name);
        if let Some(ref version) = self.version {
            response = response.with_version(version);
        }

        if !self.readiness.is_ready() {
            response.status = HealthStatus::Unhealthy;
        }

        response
    }

    /// 詳細なヘルスチェックを実行
    ///
    /// すべてのコンポーネントの詳細な状態を返す。
    pub async fn detailed(&self) -> HealthResponse {
        self.readiness_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_type() {
        assert_eq!(ProbeType::Readiness.path(), "/readyz");
        assert_eq!(ProbeType::Liveness.path(), "/livez");
        assert_eq!(ProbeType::Startup.path(), "/startupz");
    }

    #[test]
    fn test_probe_config() {
        let config = ProbeConfig::new()
            .with_timeout(Duration::from_secs(10))
            .with_details(true);

        assert_eq!(config.timeout, Duration::from_secs(10));
        assert!(config.include_details);
    }

    #[test]
    fn test_readiness_state() {
        let state = ReadinessState::new();
        assert!(!state.is_ready());
        assert_eq!(state.check(), HealthStatus::Unhealthy);

        state.set_ready();
        assert!(state.is_ready());
        assert_eq!(state.check(), HealthStatus::Healthy);

        state.set_not_ready();
        assert!(!state.is_ready());
    }

    #[test]
    fn test_readiness_state_ready() {
        let state = ReadinessState::ready();
        assert!(state.is_ready());
    }

    #[tokio::test]
    async fn test_probe_handler_liveness() {
        let handler = ProbeHandler::new("test-service").with_version("1.0.0");

        let response = handler.liveness();
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.service, "test-service");
        assert_eq!(response.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_probe_handler_readiness_not_ready() {
        let readiness = Arc::new(ReadinessState::new()); // not ready
        let handler = ProbeHandler::new("test-service").with_readiness(readiness);

        let response = handler.readiness_check().await;
        assert_eq!(response.status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_probe_handler_readiness_ready() {
        let readiness = Arc::new(ReadinessState::ready());
        let handler = ProbeHandler::new("test-service").with_readiness(readiness);

        let response = handler.readiness_check().await;
        assert_eq!(response.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_probe_handler_with_checker() {
        let checker = HealthChecker::from_sync("test", || ComponentHealth::healthy("test"));
        let handler = ProbeHandler::new("test-service")
            .with_readiness(Arc::new(ReadinessState::ready()))
            .with_checker(checker);

        let response = handler.readiness_check().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.components.len(), 1);
        assert_eq!(response.components[0].name, "test");
    }

    #[tokio::test]
    async fn test_health_checker_sync() {
        let checker = HealthChecker::from_sync("database", || {
            ComponentHealth::healthy("database").with_latency_ms(10)
        });

        assert_eq!(checker.name(), "database");
        let health = checker.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency_ms, Some(10));
    }

    #[tokio::test]
    async fn test_health_checker_async() {
        let checker = HealthChecker::new("cache", || async {
            ComponentHealth::healthy("cache").with_latency_ms(5)
        });

        assert_eq!(checker.name(), "cache");
        let health = checker.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency_ms, Some(5));
    }
}
