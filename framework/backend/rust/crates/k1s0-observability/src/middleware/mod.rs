//! ミドルウェア
//!
//! HTTP/gRPC 共通の観測性ミドルウェアを提供する。

mod grpc;
mod http;

pub use grpc::{GrpcObservability, GrpcRequestInfo, GrpcResponseInfo};
pub use http::{HttpObservability, HttpRequestInfo, HttpResponseInfo};

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;
use crate::log_fields::LogLevel;
use crate::logging::RequestLog;
use crate::metrics::MetricLabels;

/// 観測性ミドルウェアの共通トレイト
pub trait ObservabilityMiddleware {
    /// リクエスト開始時の処理
    fn on_request_start(&self, ctx: &RequestContext);

    /// リクエスト終了時の処理
    fn on_request_end(&self, ctx: &RequestContext, latency_ms: f64, success: bool);

    /// エラー発生時の処理
    fn on_error(&self, ctx: &RequestContext, error_kind: &str, error_code: &str);
}

/// リクエスト計測
///
/// リクエストの開始から終了までの計測を行う。
#[derive(Debug)]
pub struct RequestMeasurement {
    /// リクエストコンテキスト
    ctx: RequestContext,
    /// 開始時刻（ミリ秒）
    start_ms: u64,
}

impl RequestMeasurement {
    /// 計測を開始
    pub fn start(ctx: RequestContext) -> Self {
        Self {
            ctx,
            start_ms: Self::now_ms(),
        }
    }

    /// 計測を終了し、経過時間を取得
    pub fn finish(self) -> (RequestContext, f64) {
        let elapsed = Self::now_ms() - self.start_ms;
        (self.ctx, elapsed as f64)
    }

    /// コンテキストへの参照を取得
    pub fn context(&self) -> &RequestContext {
        &self.ctx
    }

    fn now_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// 観測性出力
///
/// ログ、メトリクス、トレースの出力を統合する。
#[derive(Debug)]
pub struct ObservabilityOutput {
    /// ログエントリ
    pub log: RequestLog,
    /// メトリクスラベル
    pub labels: MetricLabels,
    /// 推奨ログレベル
    pub log_level: LogLevel,
    /// レイテンシ（ミリ秒）
    pub latency_ms: f64,
    /// 成功かどうか
    pub success: bool,
}

impl ObservabilityOutput {
    /// JSON ログを出力
    pub fn log_json(&self) -> Result<String, serde_json::Error> {
        self.log.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_measurement() {
        let ctx = RequestContext::new();
        let measurement = RequestMeasurement::start(ctx.clone());

        // 少し待つ
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (returned_ctx, latency) = measurement.finish();
        assert_eq!(returned_ctx.trace_id(), ctx.trace_id());
        assert!(latency >= 10.0);
    }
}
