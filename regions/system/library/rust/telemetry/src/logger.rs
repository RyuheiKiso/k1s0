//! 構造化ログユーティリティ。
//! tracing クレートを使用し、JSON 形式の構造化ログを出力する。
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

/// init_logger は tracing-subscriber を JSON フォーマットで初期化する。
/// 環境に応じてログレベルを設定する。
///
/// - dev: debug
/// - staging: info
/// - prod: warn
pub fn init_logger(env: &str) {
    let filter = match env {
        "dev" => "debug",
        "staging" => "info",
        _ => "warn",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(filter))
        .with(
            fmt::layer()
                .json()
                .with_target(true)
                .with_span_events(fmt::format::FmtSpan::CLOSE),
        )
        .init();
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
