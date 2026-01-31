//! gRPC クライアントストリームバックプレッシャー制御
//!
//! クライアントサイドのストリーム受信におけるフロー制御を提供する。
//! セマフォベースのバッファ管理により、コンシューマの処理遅延を検知する。
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_grpc_client::stream::{FlowControlledReceiver, StreamRecvConfig};
//!
//! # tokio_test::block_on(async {
//! let receiver = FlowControlledReceiver::new(32);
//!
//! // メッセージ受信前にスロットを確保
//! let permit = receiver.acquire().await.unwrap();
//! // ... 実際の受信処理 ...
//! drop(permit); // 処理完了後にスロットを解放
//! # });
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Semaphore;

/// デフォルトの受信バッファサイズ
fn default_recv_buffer_size() -> usize {
    32
}

/// デフォルトの低速コンシューマタイムアウト（ミリ秒）
fn default_slow_consumer_timeout_ms() -> u64 {
    30_000
}

/// ストリーム受信バックプレッシャーエラー
#[derive(Debug, Error)]
pub enum StreamRecvError {
    /// セマフォがクローズされた
    #[error("stream receiver closed")]
    Closed,

    /// バッファが満杯（非ブロッキング試行時）
    #[error("recv buffer full")]
    BufferFull,

    /// タイムアウト
    #[error("slow consumer timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}

/// 受信ストリームメトリクス
#[derive(Debug)]
pub struct RecvStreamMetrics {
    /// バッファ使用率（f64 のビット表現を格納）
    buffer_usage: AtomicU64,
    /// バックプレッシャー発生回数
    backpressure_count: AtomicU64,
}

impl RecvStreamMetrics {
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

impl Default for RecvStreamMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// クライアント受信ストリーム設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRecvConfig {
    /// 受信バッファサイズ（同時に保持可能なメッセージ数）
    #[serde(default = "default_recv_buffer_size")]
    pub recv_buffer_size: usize,
    /// 低速コンシューマタイムアウト（ミリ秒）
    #[serde(default = "default_slow_consumer_timeout_ms")]
    pub slow_consumer_timeout_ms: u64,
}

impl Default for StreamRecvConfig {
    fn default() -> Self {
        Self {
            recv_buffer_size: default_recv_buffer_size(),
            slow_consumer_timeout_ms: default_slow_consumer_timeout_ms(),
        }
    }
}

/// フロー制御付き受信バッファ
///
/// セマフォを使用して受信バッファの容量を管理し、
/// コンシューマの処理遅延時にバックプレッシャーを実現する。
pub struct FlowControlledReceiver {
    semaphore: Arc<Semaphore>,
    capacity: usize,
    metrics: RecvStreamMetrics,
}

impl FlowControlledReceiver {
    /// 指定したバッファサイズで作成
    pub fn new(buffer_size: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(buffer_size)),
            capacity: buffer_size,
            metrics: RecvStreamMetrics::new(),
        }
    }

    /// 設定から作成
    pub fn from_config(config: &StreamRecvConfig) -> Self {
        Self::new(config.recv_buffer_size)
    }

    /// バッファに空きがあるまで待機してから受信許可を取得
    pub async fn acquire(&self) -> Result<RecvPermit, StreamRecvError> {
        if self.semaphore.available_permits() == 0 {
            self.metrics.increment_backpressure();
        }

        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => {
                let used = self.capacity - self.semaphore.available_permits();
                self.metrics
                    .set_buffer_usage(used as f64 / self.capacity as f64);
                Ok(RecvPermit { _permit: permit })
            }
            Err(_) => Err(StreamRecvError::Closed),
        }
    }

    /// タイムアウト付きで受信許可を取得
    pub async fn acquire_timeout(
        &self,
        timeout_ms: u64,
    ) -> Result<RecvPermit, StreamRecvError> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        match tokio::time::timeout(timeout, self.acquire()).await {
            Ok(result) => result,
            Err(_) => Err(StreamRecvError::Timeout { timeout_ms }),
        }
    }

    /// 非ブロッキングで受信許可を試行
    pub fn try_acquire(&self) -> Result<RecvPermit, StreamRecvError> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let used = self.capacity - self.semaphore.available_permits();
                self.metrics
                    .set_buffer_usage(used as f64 / self.capacity as f64);
                Ok(RecvPermit { _permit: permit })
            }
            Err(_) => {
                self.metrics.increment_backpressure();
                Err(StreamRecvError::BufferFull)
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
    pub fn metrics(&self) -> &RecvStreamMetrics {
        &self.metrics
    }
}

/// 受信許可
///
/// ドロップ時にセマフォのスロットが解放される。
pub struct RecvPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_recv_config_default() {
        let config = StreamRecvConfig::default();
        assert_eq!(config.recv_buffer_size, 32);
        assert_eq!(config.slow_consumer_timeout_ms, 30_000);
    }

    #[test]
    fn test_stream_recv_config_serde() {
        let json = r#"{"recv_buffer_size":64,"slow_consumer_timeout_ms":15000}"#;
        let config: StreamRecvConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.recv_buffer_size, 64);
        assert_eq!(config.slow_consumer_timeout_ms, 15_000);
    }

    #[test]
    fn test_recv_stream_metrics_default() {
        let metrics = RecvStreamMetrics::new();
        assert!((metrics.buffer_usage() - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.backpressure_count(), 0);
    }

    #[test]
    fn test_flow_controlled_receiver_new() {
        let receiver = FlowControlledReceiver::new(16);
        assert_eq!(receiver.capacity(), 16);
        assert_eq!(receiver.available_permits(), 16);
    }

    #[test]
    fn test_flow_controlled_receiver_from_config() {
        let config = StreamRecvConfig {
            recv_buffer_size: 64,
            slow_consumer_timeout_ms: 5_000,
        };
        let receiver = FlowControlledReceiver::from_config(&config);
        assert_eq!(receiver.capacity(), 64);
    }

    #[tokio::test]
    async fn test_acquire_and_release() {
        let receiver = FlowControlledReceiver::new(2);
        assert_eq!(receiver.available_permits(), 2);

        let permit1 = receiver.acquire().await.unwrap();
        assert_eq!(receiver.available_permits(), 1);

        let permit2 = receiver.acquire().await.unwrap();
        assert_eq!(receiver.available_permits(), 0);

        drop(permit1);
        assert_eq!(receiver.available_permits(), 1);

        drop(permit2);
        assert_eq!(receiver.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_try_acquire_buffer_full() {
        let receiver = FlowControlledReceiver::new(1);

        let _permit = receiver.try_acquire().unwrap();
        assert_eq!(receiver.available_permits(), 0);

        let result = receiver.try_acquire();
        assert!(result.is_err());
        assert_eq!(receiver.metrics().backpressure_count(), 1);
    }

    #[tokio::test]
    async fn test_acquire_timeout() {
        let receiver = FlowControlledReceiver::new(1);
        let _permit = receiver.acquire().await.unwrap();

        let result = receiver.acquire_timeout(50).await;
        assert!(matches!(
            result,
            Err(StreamRecvError::Timeout { timeout_ms: 50 })
        ));
    }

    #[tokio::test]
    async fn test_backpressure_metrics() {
        let receiver = FlowControlledReceiver::new(1);
        let _permit = receiver.acquire().await.unwrap();

        let _ = receiver.try_acquire();
        let _ = receiver.try_acquire();
        assert_eq!(receiver.metrics().backpressure_count(), 2);
    }

    #[tokio::test]
    async fn test_buffer_usage_tracking() {
        let receiver = FlowControlledReceiver::new(4);

        let _p1 = receiver.acquire().await.unwrap();
        assert!((receiver.metrics().buffer_usage() - 0.25).abs() < f64::EPSILON);

        let _p2 = receiver.acquire().await.unwrap();
        assert!((receiver.metrics().buffer_usage() - 0.5).abs() < f64::EPSILON);
    }
}
