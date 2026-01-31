//! gRPC ストリームバックプレッシャー制御
//!
//! サーバストリーミング RPC におけるフロー制御を提供する。
//! セマフォベースのバッファ管理により、プロデューサの過剰送信を防止する。
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_grpc_server::stream::{FlowControlledSender, StreamBackpressureConfig};
//!
//! # tokio_test::block_on(async {
//! let sender = FlowControlledSender::new(64);
//!
//! // バッファに空きがあるまで待機してから送信
//! let permit = sender.acquire().await.unwrap();
//! // ... 実際の送信処理 ...
//! drop(permit); // 送信完了後にバッファスロットを解放
//!
//! // 非ブロッキングで試行
//! match sender.try_acquire() {
//!     Ok(permit) => { /* 送信可能 */ drop(permit); }
//!     Err(_) => { /* バッファ満杯 */ }
//! }
//! # });
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Semaphore;

/// デフォルトの送信バッファサイズ
fn default_send_buffer_size() -> usize {
    64
}

/// デフォルトの低速プロデューサタイムアウト（ミリ秒）
fn default_slow_producer_timeout_ms() -> u64 {
    30_000
}

/// ストリームバックプレッシャーエラー
#[derive(Debug, Error)]
pub enum StreamBackpressureError {
    /// セマフォがクローズされた
    #[error("stream sender closed")]
    Closed,

    /// バッファが満杯（非ブロッキング試行時）
    #[error("send buffer full")]
    BufferFull,

    /// タイムアウト
    #[error("slow producer timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}

/// ストリームバックプレッシャーメトリクス
#[derive(Debug)]
pub struct StreamMetrics {
    /// バッファ使用率（f64 のビット表現を格納）
    buffer_usage: AtomicU64,
    /// バックプレッシャー発生回数
    backpressure_count: AtomicU64,
}

impl StreamMetrics {
    /// 新しいメトリクスを作成
    fn new() -> Self {
        Self {
            buffer_usage: AtomicU64::new(0),
            backpressure_count: AtomicU64::new(0),
        }
    }

    /// バッファ使用率を設定
    fn set_buffer_usage(&self, usage: f64) {
        self.buffer_usage
            .store(usage.to_bits(), Ordering::Relaxed);
    }

    /// バッファ使用率を取得（0.0 - 1.0）
    pub fn buffer_usage(&self) -> f64 {
        f64::from_bits(self.buffer_usage.load(Ordering::Relaxed))
    }

    /// バックプレッシャー発生回数をインクリメント
    fn increment_backpressure(&self) {
        self.backpressure_count.fetch_add(1, Ordering::Relaxed);
    }

    /// バックプレッシャー発生回数を取得
    pub fn backpressure_count(&self) -> u64 {
        self.backpressure_count.load(Ordering::Relaxed)
    }
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// フロー制御付き gRPC ストリーム設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamBackpressureConfig {
    /// 送信バッファサイズ（同時に保持可能なメッセージ数）
    #[serde(default = "default_send_buffer_size")]
    pub send_buffer_size: usize,
    /// 低速プロデューサタイムアウト（ミリ秒）
    #[serde(default = "default_slow_producer_timeout_ms")]
    pub slow_producer_timeout_ms: u64,
}

impl Default for StreamBackpressureConfig {
    fn default() -> Self {
        Self {
            send_buffer_size: default_send_buffer_size(),
            slow_producer_timeout_ms: default_slow_producer_timeout_ms(),
        }
    }
}

/// フロー制御付き送信バッファ
///
/// セマフォを使用して送信バッファの容量を管理し、
/// バックプレッシャーを実現する。
pub struct FlowControlledSender {
    semaphore: Arc<Semaphore>,
    capacity: usize,
    metrics: StreamMetrics,
}

impl FlowControlledSender {
    /// 指定したバッファサイズで作成
    pub fn new(buffer_size: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(buffer_size)),
            capacity: buffer_size,
            metrics: StreamMetrics::new(),
        }
    }

    /// 設定から作成
    pub fn from_config(config: &StreamBackpressureConfig) -> Self {
        Self::new(config.send_buffer_size)
    }

    /// バッファに空きがあるまで待機してから送信許可を取得
    pub async fn acquire(&self) -> Result<FlowControlPermit, StreamBackpressureError> {
        // バッファが満杯の場合はバックプレッシャーとしてカウント
        if self.semaphore.available_permits() == 0 {
            self.metrics.increment_backpressure();
        }

        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => {
                let used = self.capacity - self.semaphore.available_permits();
                self.metrics
                    .set_buffer_usage(used as f64 / self.capacity as f64);
                Ok(FlowControlPermit { _permit: permit })
            }
            Err(_) => Err(StreamBackpressureError::Closed),
        }
    }

    /// タイムアウト付きでバッファの送信許可を取得
    pub async fn acquire_timeout(
        &self,
        timeout_ms: u64,
    ) -> Result<FlowControlPermit, StreamBackpressureError> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        match tokio::time::timeout(timeout, self.acquire()).await {
            Ok(result) => result,
            Err(_) => Err(StreamBackpressureError::Timeout { timeout_ms }),
        }
    }

    /// 非ブロッキングで送信許可を試行
    pub fn try_acquire(&self) -> Result<FlowControlPermit, StreamBackpressureError> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let used = self.capacity - self.semaphore.available_permits();
                self.metrics
                    .set_buffer_usage(used as f64 / self.capacity as f64);
                Ok(FlowControlPermit { _permit: permit })
            }
            Err(_) => {
                self.metrics.increment_backpressure();
                Err(StreamBackpressureError::BufferFull)
            }
        }
    }

    /// 現在の利用可能なバッファスロット数を取得
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// バッファ容量を取得
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// メトリクスへの参照を取得
    pub fn metrics(&self) -> &StreamMetrics {
        &self.metrics
    }
}

/// フロー制御許可
///
/// ドロップ時にセマフォのスロットが解放される。
pub struct FlowControlPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_backpressure_config_default() {
        let config = StreamBackpressureConfig::default();
        assert_eq!(config.send_buffer_size, 64);
        assert_eq!(config.slow_producer_timeout_ms, 30_000);
    }

    #[test]
    fn test_stream_backpressure_config_serde() {
        let json = r#"{"send_buffer_size":128,"slow_producer_timeout_ms":60000}"#;
        let config: StreamBackpressureConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.send_buffer_size, 128);
        assert_eq!(config.slow_producer_timeout_ms, 60_000);
    }

    #[test]
    fn test_stream_metrics_default() {
        let metrics = StreamMetrics::new();
        assert!((metrics.buffer_usage() - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.backpressure_count(), 0);
    }

    #[test]
    fn test_stream_metrics_buffer_usage() {
        let metrics = StreamMetrics::new();
        metrics.set_buffer_usage(0.75);
        assert!((metrics.buffer_usage() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_flow_controlled_sender_new() {
        let sender = FlowControlledSender::new(32);
        assert_eq!(sender.capacity(), 32);
        assert_eq!(sender.available_permits(), 32);
    }

    #[test]
    fn test_flow_controlled_sender_from_config() {
        let config = StreamBackpressureConfig {
            send_buffer_size: 128,
            slow_producer_timeout_ms: 10_000,
        };
        let sender = FlowControlledSender::from_config(&config);
        assert_eq!(sender.capacity(), 128);
    }

    #[tokio::test]
    async fn test_acquire_and_release() {
        let sender = FlowControlledSender::new(2);
        assert_eq!(sender.available_permits(), 2);

        let permit1 = sender.acquire().await.unwrap();
        assert_eq!(sender.available_permits(), 1);

        let permit2 = sender.acquire().await.unwrap();
        assert_eq!(sender.available_permits(), 0);

        drop(permit1);
        assert_eq!(sender.available_permits(), 1);

        drop(permit2);
        assert_eq!(sender.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_try_acquire_buffer_full() {
        let sender = FlowControlledSender::new(1);

        let _permit = sender.try_acquire().unwrap();
        assert_eq!(sender.available_permits(), 0);

        let result = sender.try_acquire();
        assert!(result.is_err());
        assert_eq!(sender.metrics().backpressure_count(), 1);
    }

    #[tokio::test]
    async fn test_acquire_timeout() {
        let sender = FlowControlledSender::new(1);
        let _permit = sender.acquire().await.unwrap();

        let result = sender.acquire_timeout(50).await;
        assert!(matches!(
            result,
            Err(StreamBackpressureError::Timeout { timeout_ms: 50 })
        ));
    }

    #[tokio::test]
    async fn test_backpressure_metrics() {
        let sender = FlowControlledSender::new(1);
        let _permit = sender.acquire().await.unwrap();

        // バッファ満杯で try_acquire するとバックプレッシャーカウントが増加
        let _ = sender.try_acquire();
        let _ = sender.try_acquire();
        assert_eq!(sender.metrics().backpressure_count(), 2);
    }

    #[tokio::test]
    async fn test_buffer_usage_tracking() {
        let sender = FlowControlledSender::new(4);

        let _p1 = sender.acquire().await.unwrap();
        // 1/4 = 0.25 使用中
        assert!((sender.metrics().buffer_usage() - 0.25).abs() < f64::EPSILON);

        let _p2 = sender.acquire().await.unwrap();
        // 2/4 = 0.5 使用中
        assert!((sender.metrics().buffer_usage() - 0.5).abs() < f64::EPSILON);
    }
}
