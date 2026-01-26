//! サーキットブレーカ
//!
//! 依存先の障害を検知し、呼び出しを遮断する。
//! k1s0 では既定 OFF。必要時のみ有効化。

use crate::error::ResilienceError;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;

/// デフォルトの失敗閾値
pub const DEFAULT_FAILURE_THRESHOLD: u32 = 5;

/// デフォルトの成功閾値（Half-Open → Closed）
pub const DEFAULT_SUCCESS_THRESHOLD: u32 = 3;

/// デフォルトのリセット時間（秒）
pub const DEFAULT_RESET_TIMEOUT_SECS: u64 = 30;

/// サーキットブレーカ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 有効/無効（既定: 無効）
    #[serde(default)]
    enabled: bool,
    /// 失敗閾値（Open に遷移する失敗数）
    #[serde(default = "default_failure_threshold")]
    failure_threshold: u32,
    /// 成功閾値（Half-Open → Closed に遷移する成功数）
    #[serde(default = "default_success_threshold")]
    success_threshold: u32,
    /// リセットタイムアウト（秒）
    #[serde(default = "default_reset_timeout_secs")]
    reset_timeout_secs: u64,
    /// 失敗としてカウントするエラーの判定
    #[serde(default)]
    failure_predicate: FailurePredicate,
}

fn default_failure_threshold() -> u32 {
    DEFAULT_FAILURE_THRESHOLD
}

fn default_success_threshold() -> u32 {
    DEFAULT_SUCCESS_THRESHOLD
}

fn default_reset_timeout_secs() -> u64 {
    DEFAULT_RESET_TIMEOUT_SECS
}

impl CircuitBreakerConfig {
    /// 無効な設定を作成
    pub fn disabled() -> Self {
        Self::default()
    }

    /// 有効な設定を作成
    pub fn enabled() -> CircuitBreakerConfigBuilder {
        CircuitBreakerConfigBuilder::new()
    }

    /// 有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 失敗閾値を取得
    pub fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }

    /// 成功閾値を取得
    pub fn success_threshold(&self) -> u32 {
        self.success_threshold
    }

    /// リセットタイムアウトを取得
    pub fn reset_timeout(&self) -> Duration {
        Duration::from_secs(self.reset_timeout_secs)
    }

    /// 失敗判定条件を取得
    pub fn failure_predicate(&self) -> &FailurePredicate {
        &self.failure_predicate
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_threshold: DEFAULT_FAILURE_THRESHOLD,
            success_threshold: DEFAULT_SUCCESS_THRESHOLD,
            reset_timeout_secs: DEFAULT_RESET_TIMEOUT_SECS,
            failure_predicate: FailurePredicate::default(),
        }
    }
}

/// サーキットブレーカ設定ビルダー
#[derive(Debug, Default)]
pub struct CircuitBreakerConfigBuilder {
    failure_threshold: Option<u32>,
    success_threshold: Option<u32>,
    reset_timeout_secs: Option<u64>,
    failure_predicate: Option<FailurePredicate>,
}

impl CircuitBreakerConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 失敗閾値を設定
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = Some(threshold);
        self
    }

    /// 成功閾値を設定
    pub fn success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = Some(threshold);
        self
    }

    /// リセットタイムアウトを設定（秒）
    pub fn reset_timeout_secs(mut self, secs: u64) -> Self {
        self.reset_timeout_secs = Some(secs);
        self
    }

    /// 失敗判定条件を設定
    pub fn failure_predicate(mut self, predicate: FailurePredicate) -> Self {
        self.failure_predicate = Some(predicate);
        self
    }

    /// 設定をビルド
    pub fn build(self) -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            enabled: true,
            failure_threshold: self
                .failure_threshold
                .unwrap_or_else(default_failure_threshold),
            success_threshold: self
                .success_threshold
                .unwrap_or_else(default_success_threshold),
            reset_timeout_secs: self
                .reset_timeout_secs
                .unwrap_or_else(default_reset_timeout_secs),
            failure_predicate: self.failure_predicate.unwrap_or_default(),
        }
    }
}

/// 失敗判定条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePredicate {
    /// タイムアウトを失敗としてカウント
    #[serde(default = "default_true")]
    count_timeout: bool,
    /// 接続エラーを失敗としてカウント
    #[serde(default = "default_true")]
    count_connection_error: bool,
    /// サーバーエラー（5xx/INTERNAL等）を失敗としてカウント
    #[serde(default = "default_true")]
    count_server_error: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FailurePredicate {
    fn default() -> Self {
        Self {
            count_timeout: true,
            count_connection_error: true,
            count_server_error: true,
        }
    }
}

impl FailurePredicate {
    /// すべての種類のエラーをカウント
    pub fn all() -> Self {
        Self {
            count_timeout: true,
            count_connection_error: true,
            count_server_error: true,
        }
    }

    /// タイムアウトのみカウント
    pub fn timeout_only() -> Self {
        Self {
            count_timeout: true,
            count_connection_error: false,
            count_server_error: false,
        }
    }

    /// エラーが失敗としてカウントされるかどうか
    pub fn should_count(&self, error: &ResilienceError) -> bool {
        match error {
            ResilienceError::Timeout { .. } => self.count_timeout,
            ResilienceError::Connection { .. } => self.count_connection_error,
            ResilienceError::CircuitOpen { .. } => false,
            ResilienceError::ConcurrencyLimit { .. } => false,
            _ => self.count_server_error,
        }
    }
}

/// サーキットブレーカ状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 閉じている（正常）
    Closed,
    /// 開いている（遮断）
    Open,
    /// 半開き（テスト中）
    HalfOpen,
}

impl CircuitState {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

/// サーキットブレーカ
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: RwLock<Option<std::time::Instant>>,
    metrics: CircuitBreakerMetrics,
}

impl CircuitBreaker {
    /// 新しいサーキットブレーカを作成
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            metrics: CircuitBreakerMetrics::new(),
        }
    }

    /// 無効なサーキットブレーカを作成（常に許可）
    pub fn disabled() -> Self {
        Self::new(CircuitBreakerConfig::disabled())
    }

    /// 呼び出しを許可するかどうか
    pub fn allow_request(&self) -> bool {
        if !self.config.is_enabled() {
            return true;
        }

        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // タイムアウト経過で Half-Open に遷移
                if self.should_try_reset() {
                    *self.state.write().unwrap() = CircuitState::HalfOpen;
                    self.success_count.store(0, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// リセットを試行すべきかどうか
    fn should_try_reset(&self) -> bool {
        let last_failure = self.last_failure_time.read().unwrap();
        if let Some(time) = *last_failure {
            time.elapsed() >= self.config.reset_timeout()
        } else {
            true
        }
    }

    /// 成功を記録
    pub fn record_success(&self) {
        if !self.config.is_enabled() {
            return;
        }

        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => {
                // 連続失敗をリセット
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.success_threshold() as u64 {
                    // Closed に遷移
                    *self.state.write().unwrap() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.metrics.increment_state_transition();
                }
            }
            CircuitState::Open => {}
        }
    }

    /// 失敗を記録
    pub fn record_failure(&self, error: &ResilienceError) {
        if !self.config.is_enabled() {
            return;
        }

        if !self.config.failure_predicate().should_count(error) {
            return;
        }

        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.failure_threshold() as u64 {
                    // Open に遷移
                    *self.state.write().unwrap() = CircuitState::Open;
                    *self.last_failure_time.write().unwrap() = Some(std::time::Instant::now());
                    self.metrics.increment_state_transition();
                }
            }
            CircuitState::HalfOpen => {
                // 即座に Open に戻る
                *self.state.write().unwrap() = CircuitState::Open;
                *self.last_failure_time.write().unwrap() = Some(std::time::Instant::now());
                self.metrics.increment_state_transition();
            }
            CircuitState::Open => {}
        }
    }

    /// 処理を実行
    pub async fn execute<F, T>(&self, f: F) -> Result<T, ResilienceError>
    where
        F: std::future::Future<Output = Result<T, ResilienceError>>,
    {
        if !self.allow_request() {
            self.metrics.increment_rejected();
            return Err(ResilienceError::circuit_open(
                self.state().as_str().to_string(),
            ));
        }

        match f.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure(&e);
                Err(e)
            }
        }
    }

    /// 現在の状態を取得
    pub fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    /// 失敗数を取得
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::SeqCst)
    }

    /// 設定を取得
    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }

    /// メトリクスを取得
    pub fn metrics(&self) -> &CircuitBreakerMetrics {
        &self.metrics
    }
}

/// サーキットブレーカメトリクス
#[derive(Debug)]
pub struct CircuitBreakerMetrics {
    /// 拒否数
    rejected: AtomicU64,
    /// 状態遷移数
    state_transitions: AtomicU64,
}

impl CircuitBreakerMetrics {
    /// 新しいメトリクスを作成
    pub fn new() -> Self {
        Self {
            rejected: AtomicU64::new(0),
            state_transitions: AtomicU64::new(0),
        }
    }

    /// 拒否数をインクリメント
    fn increment_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::SeqCst);
    }

    /// 状態遷移数をインクリメント
    fn increment_state_transition(&self) {
        self.state_transitions.fetch_add(1, Ordering::SeqCst);
    }

    /// 拒否数を取得
    pub fn rejected(&self) -> u64 {
        self.rejected.load(Ordering::SeqCst)
    }

    /// 状態遷移数を取得
    pub fn state_transitions(&self) -> u64 {
        self.state_transitions.load(Ordering::SeqCst)
    }
}

impl Default for CircuitBreakerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_disabled() {
        let config = CircuitBreakerConfig::disabled();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_circuit_breaker_config_enabled() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(3)
            .success_threshold(2)
            .reset_timeout_secs(10)
            .build();

        assert!(config.is_enabled());
        assert_eq!(config.failure_threshold(), 3);
        assert_eq!(config.success_threshold(), 2);
    }

    #[test]
    fn test_circuit_breaker_disabled() {
        let cb = CircuitBreaker::disabled();
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_state_transition() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(2)
            .build();
        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.state(), CircuitState::Closed);

        // 1回目の失敗
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Closed);

        // 2回目の失敗 → Open
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_state_as_str() {
        assert_eq!(CircuitState::Closed.as_str(), "closed");
        assert_eq!(CircuitState::Open.as_str(), "open");
        assert_eq!(CircuitState::HalfOpen.as_str(), "half_open");
    }

    #[test]
    fn test_failure_predicate() {
        let predicate = FailurePredicate::all();
        assert!(predicate.should_count(&ResilienceError::timeout(1000)));
        assert!(predicate.should_count(&ResilienceError::connection("refused")));

        let predicate = FailurePredicate::timeout_only();
        assert!(predicate.should_count(&ResilienceError::timeout(1000)));
        assert!(!predicate.should_count(&ResilienceError::connection("refused")));
    }

    #[tokio::test]
    async fn test_circuit_breaker_execute_success() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::enabled().build());

        let result = cb.execute(async { Ok::<_, ResilienceError>(42) }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_circuit_breaker_execute_open() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(1)
            .build();
        let cb = CircuitBreaker::new(config);

        // 1回失敗して Open に
        cb.record_failure(&ResilienceError::timeout(1000));

        // 次のリクエストは拒否
        let result = cb.execute(async { Ok::<_, ResilienceError>(42) }).await;
        assert!(matches!(result.unwrap_err(), ResilienceError::CircuitOpen { .. }));
    }
}
