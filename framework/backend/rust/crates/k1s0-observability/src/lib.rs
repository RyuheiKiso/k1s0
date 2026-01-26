//! k1s0-observability
//!
//! 観測性（ログ/トレース/メトリクス）の統一初期化ライブラリ。
//!
//! # 設計方針
//!
//! - **必須フィールドの強制**: `service.name`, `env` 等を初期化時に必須化
//! - **JSON ログの統一**: 構造化ログの必須フィールドを固定
//! - **OTel 統合**: OpenTelemetry によるトレース/メトリクス
//! - **ミドルウェア提供**: HTTP/gRPC 共通の観測性ミドルウェア
//!
//! # 必須フィールド（ログ）
//!
//! | フィールド | 説明 |
//! |-----------|------|
//! | `timestamp` | ISO 8601 形式のタイムスタンプ |
//! | `level` | ログレベル（DEBUG/INFO/WARN/ERROR） |
//! | `message` | ログメッセージ |
//! | `service.name` | サービス名 |
//! | `service.env` | 環境名（dev/stg/prod） |
//! | `trace.id` | トレース ID（リクエスト相関用） |
//! | `request.id` | リクエスト ID |
//!
//! # 使用例
//!
//! ```
//! use k1s0_observability::{ObservabilityConfig, ObservabilityBuilder};
//!
//! // 初期化（必須フィールドを強制）
//! let config = ObservabilityConfig::builder()
//!     .service_name("user-service")
//!     .env("dev")
//!     .build()
//!     .expect("必須フィールドが不足");
//!
//! // リクエストコンテキストの作成
//! let ctx = config.new_request_context();
//! println!("trace_id: {}", ctx.trace_id());
//! println!("request_id: {}", ctx.request_id());
//! ```

mod config;
mod context;
mod log_fields;
pub mod logging;
pub mod metrics;
pub mod middleware;
pub mod tracing;

pub use config::{ObservabilityBuilder, ObservabilityConfig};
pub use context::RequestContext;
pub use log_fields::{LogEntry, LogLevel, LogFields};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_initialization() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .expect("config should be valid");

        assert_eq!(config.service_name(), "test-service");
        assert_eq!(config.env(), "dev");

        let ctx = config.new_request_context();
        assert!(!ctx.trace_id().is_empty());
        assert!(!ctx.request_id().is_empty());
    }

    #[test]
    fn test_log_entry() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let ctx = config.new_request_context();
        let entry = LogEntry::info("テストメッセージ")
            .with_context(&ctx)
            .with_service(&config);

        let json = entry.to_json().unwrap();
        assert!(json.contains("test-service"));
        assert!(json.contains("テストメッセージ"));
    }
}
