//! 構造化ログユーティリティ。
//! tracing クレートを使用し、JSON またはテキスト形式の構造化ログを出力する。
//!
//! # 使用例
//!
//! ```ignore
//! use tracing::{info, warn, error};
//!
//! info!(service = "order-server", tier = "service", "Request completed");
//! warn!(method = "POST", path = "/api/v1/orders", "Slow request detected");
//! error!(error = %e, "Failed to process request");
//! ```

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// init_logger は tracing-subscriber を初期化する。
/// 環境に応じてログレベルを設定し、format に応じて出力形式を切り替える。
///
/// - dev: debug
/// - staging: info
/// - prod: warn
///
/// format が "text" の場合はプレーンテキスト出力、それ以外は JSON 出力。
pub fn init_logger(env: &str, format: &str) {
    let filter = match env {
        "dev" => "debug",
        "staging" => "info",
        _ => "warn",
    };

    let registry = tracing_subscriber::registry().with(EnvFilter::new(filter));

    if format == "text" {
        registry
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_span_events(fmt::format::FmtSpan::CLOSE),
            )
            .init();
    } else {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_span_events(fmt::format::FmtSpan::CLOSE),
            )
            .init();
    }
}

/// parse_log_level はログレベル文字列を tracing の Level に変換する。
pub fn parse_log_level(level: &str) -> tracing::Level {
    match level {
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    }
}
