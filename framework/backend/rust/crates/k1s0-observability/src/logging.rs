//! ロギング
//!
//! tracing-subscriber を使用した構造化ログの初期化と出力を提供する。
//!
//! # 機能
//!
//! - tracing-subscriber による統一的なログ初期化
//! - JSON フォーマットでの構造化ログ出力
//! - 環境変数によるフィルタリング（RUST_LOG）
//! - サービス情報の自動付与
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_observability::{ObservabilityConfig, logging::init_logging};
//!
//! let config = ObservabilityConfig::builder()
//!     .service_name("my-service")
//!     .env("dev")
//!     .build()
//!     .unwrap();
//!
//! // ロギング初期化
//! let _guard = init_logging(&config).expect("failed to init logging");
//!
//! // tracing マクロでログ出力
//! tracing::info!(user_id = 123, "user logged in");
//! ```

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;
use crate::log_fields::{LogEntry, LogLevel};
use std::io;
use tracing_subscriber::{
    fmt::{self, format::JsonFields, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// ロガー
///
/// サービス情報を保持し、ログエントリを生成する。
#[derive(Debug, Clone)]
pub struct Logger {
    service_name: String,
    service_env: String,
    service_version: Option<String>,
    min_level: LogLevel,
}

impl Logger {
    /// ObservabilityConfig からロガーを作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        let min_level = match config.log_level().to_uppercase().as_str() {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        };

        Self {
            service_name: config.service_name().to_string(),
            service_env: config.env().to_string(),
            service_version: config.version().map(|s| s.to_string()),
            min_level,
        }
    }

    /// ログエントリを作成（サービス情報付き）
    pub fn entry(&self, level: LogLevel, message: impl Into<String>) -> LogEntry {
        let mut entry = LogEntry::new(level, message);
        entry.service_name = Some(self.service_name.clone());
        entry.service_env = Some(self.service_env.clone());
        entry.service_version = self.service_version.clone();
        entry
    }

    /// コンテキスト付きログエントリを作成
    pub fn entry_with_context(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        ctx: &RequestContext,
    ) -> LogEntry {
        self.entry(level, message).with_context(ctx)
    }

    /// TRACE ログ
    pub fn trace(&self, message: impl Into<String>) -> LogEntry {
        self.entry(LogLevel::Trace, message)
    }

    /// DEBUG ログ
    pub fn debug(&self, message: impl Into<String>) -> LogEntry {
        self.entry(LogLevel::Debug, message)
    }

    /// INFO ログ
    pub fn info(&self, message: impl Into<String>) -> LogEntry {
        self.entry(LogLevel::Info, message)
    }

    /// WARN ログ
    pub fn warn(&self, message: impl Into<String>) -> LogEntry {
        self.entry(LogLevel::Warn, message)
    }

    /// ERROR ログ
    pub fn error(&self, message: impl Into<String>) -> LogEntry {
        self.entry(LogLevel::Error, message)
    }

    /// 最小ログレベルを取得
    pub fn min_level(&self) -> LogLevel {
        self.min_level
    }

    /// 指定レベルが出力対象かどうか
    pub fn is_enabled(&self, level: LogLevel) -> bool {
        level as u8 >= self.min_level as u8
    }
}

/// リクエストログ
///
/// HTTP/gRPC リクエストのログを構造化する。
#[derive(Debug, Clone)]
pub struct RequestLog {
    /// 基本のログエントリ
    pub entry: LogEntry,
    /// HTTP メソッド
    pub http_method: Option<String>,
    /// HTTP パス
    pub http_path: Option<String>,
    /// HTTP ステータスコード
    pub http_status_code: Option<u16>,
    /// gRPC サービス
    pub grpc_service: Option<String>,
    /// gRPC メソッド
    pub grpc_method: Option<String>,
    /// gRPC ステータスコード
    pub grpc_status_code: Option<i32>,
    /// レイテンシ（ミリ秒）
    pub latency_ms: Option<f64>,
}

impl RequestLog {
    /// ログエントリから作成
    pub fn new(entry: LogEntry) -> Self {
        Self {
            entry,
            http_method: None,
            http_path: None,
            http_status_code: None,
            grpc_service: None,
            grpc_method: None,
            grpc_status_code: None,
            latency_ms: None,
        }
    }

    /// HTTP リクエスト情報を設定
    pub fn with_http(
        mut self,
        method: impl Into<String>,
        path: impl Into<String>,
        status_code: u16,
    ) -> Self {
        self.http_method = Some(method.into());
        self.http_path = Some(path.into());
        self.http_status_code = Some(status_code);
        self
    }

    /// gRPC リクエスト情報を設定
    pub fn with_grpc(
        mut self,
        service: impl Into<String>,
        method: impl Into<String>,
        status_code: i32,
    ) -> Self {
        self.grpc_service = Some(service.into());
        self.grpc_method = Some(method.into());
        self.grpc_status_code = Some(status_code);
        self
    }

    /// レイテンシを設定
    pub fn with_latency(mut self, latency_ms: f64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let mut extra = serde_json::Map::new();

        if let Some(ref method) = self.http_method {
            extra.insert("http.method".to_string(), serde_json::json!(method));
        }
        if let Some(ref path) = self.http_path {
            extra.insert("http.path".to_string(), serde_json::json!(path));
        }
        if let Some(status) = self.http_status_code {
            extra.insert("http.status_code".to_string(), serde_json::json!(status));
        }
        if let Some(ref service) = self.grpc_service {
            extra.insert("grpc.service".to_string(), serde_json::json!(service));
        }
        if let Some(ref method) = self.grpc_method {
            extra.insert("grpc.method".to_string(), serde_json::json!(method));
        }
        if let Some(status) = self.grpc_status_code {
            extra.insert("grpc.status_code".to_string(), serde_json::json!(status));
        }
        if let Some(latency) = self.latency_ms {
            extra.insert("latency_ms".to_string(), serde_json::json!(latency));
        }

        let mut entry = self.entry.clone();
        entry.extra = Some(serde_json::Value::Object(extra));
        entry.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObservabilityConfig;

    #[test]
    fn test_logger_from_config() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .log_level("DEBUG")
            .build()
            .unwrap();

        let logger = Logger::from_config(&config);
        assert_eq!(logger.min_level(), LogLevel::Debug);
    }

    #[test]
    fn test_logger_entry() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let logger = Logger::from_config(&config);
        let entry = logger.info("テスト");

        assert_eq!(entry.service_name, Some("test-service".to_string()));
        assert_eq!(entry.service_env, Some("dev".to_string()));
    }

    #[test]
    fn test_logger_entry_with_context() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let logger = Logger::from_config(&config);
        let ctx = RequestContext::new();
        let entry = logger.entry_with_context(LogLevel::Info, "テスト", &ctx);

        assert_eq!(entry.trace_id, Some(ctx.trace_id().to_string()));
    }

    #[test]
    fn test_logger_is_enabled() {
        let config = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .log_level("WARN")
            .build()
            .unwrap();

        let logger = Logger::from_config(&config);

        assert!(!logger.is_enabled(LogLevel::Debug));
        assert!(!logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Warn));
        assert!(logger.is_enabled(LogLevel::Error));
    }

    #[test]
    fn test_request_log_http() {
        let entry = LogEntry::info("リクエスト完了");
        let log = RequestLog::new(entry)
            .with_http("GET", "/api/users", 200)
            .with_latency(42.5);

        let json = log.to_json().unwrap();
        assert!(json.contains("\"http.method\":\"GET\""));
        assert!(json.contains("\"http.status_code\":200"));
        assert!(json.contains("\"latency_ms\":42.5"));
    }

    #[test]
    fn test_request_log_grpc() {
        let entry = LogEntry::info("gRPC 呼び出し完了");
        let log = RequestLog::new(entry)
            .with_grpc("UserService", "GetUser", 0)
            .with_latency(15.0);

        let json = log.to_json().unwrap();
        assert!(json.contains("\"grpc.service\":\"UserService\""));
        assert!(json.contains("\"grpc.method\":\"GetUser\""));
        assert!(json.contains("\"grpc.status_code\":0"));
    }
}

// ============================================================================
// tracing-subscriber 統合
// ============================================================================

/// ロギング初期化の結果を保持するガード
///
/// このガードがドロップされると、ロギングシステムがシャットダウンされる。
/// アプリケーションのライフタイム中は保持し続ける必要がある。
pub struct LoggingGuard {
    _private: (),
}

impl LoggingGuard {
    fn new() -> Self {
        Self { _private: () }
    }
}

/// ロギングを初期化
///
/// tracing-subscriber を使用して、JSON 形式の構造化ログを設定する。
///
/// # 引数
///
/// * `config` - ObservabilityConfig
///
/// # 戻り値
///
/// * `Ok(LoggingGuard)` - 初期化成功
/// * `Err` - 初期化失敗
///
/// # 例
///
/// ```ignore
/// let config = ObservabilityConfig::builder()
///     .service_name("my-service")
///     .env("dev")
///     .build()
///     .unwrap();
///
/// let _guard = init_logging(&config)?;
/// tracing::info!("application started");
/// ```
pub fn init_logging(config: &ObservabilityConfig) -> Result<LoggingGuard, LoggingError> {
    let filter = create_env_filter(config)?;
    let json_layer = create_json_layer(config);

    tracing_subscriber::registry()
        .with(filter)
        .with(json_layer)
        .try_init()
        .map_err(|e| LoggingError::InitFailed(e.to_string()))?;

    Ok(LoggingGuard::new())
}

/// カスタムライターでロギングを初期化
///
/// テストやカスタム出力先が必要な場合に使用する。
///
/// # 引数
///
/// * `config` - ObservabilityConfig
/// * `writer` - 出力先ライター
pub fn init_logging_with_writer<W>(
    config: &ObservabilityConfig,
    writer: W,
) -> Result<LoggingGuard, LoggingError>
where
    W: for<'writer> fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    let filter = create_env_filter(config)?;
    let json_layer = create_json_layer_with_writer(config, writer);

    tracing_subscriber::registry()
        .with(filter)
        .with(json_layer)
        .try_init()
        .map_err(|e| LoggingError::InitFailed(e.to_string()))?;

    Ok(LoggingGuard::new())
}

/// 環境フィルタを作成
fn create_env_filter(config: &ObservabilityConfig) -> Result<EnvFilter, LoggingError> {
    // RUST_LOG 環境変数があればそれを使用、なければ config から
    let default_filter = match config.log_level().to_uppercase().as_str() {
        "TRACE" => "trace",
        "DEBUG" => "debug",
        "WARN" => "warn",
        "ERROR" => "error",
        _ => "info",
    };

    EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_filter))
        .map_err(|e| LoggingError::FilterParseFailed(e.to_string()))
}

/// JSON レイヤーを作成（標準出力）
fn create_json_layer<S>(
    config: &ObservabilityConfig,
) -> fmt::Layer<S, JsonFields, fmt::format::Format<fmt::format::Json, UtcTime<&'static str>>>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::layer()
        .json()
        .with_timer(UtcTime::new(time_format_description()))
        .with_current_span(true)
        .with_span_list(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .flatten_event(true)
}

/// JSON レイヤーを作成（カスタムライター）
fn create_json_layer_with_writer<S, W>(
    config: &ObservabilityConfig,
    writer: W,
) -> fmt::Layer<S, JsonFields, fmt::format::Format<fmt::format::Json, UtcTime<&'static str>>, W>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    W: for<'writer> fmt::MakeWriter<'writer> + 'static,
{
    fmt::layer()
        .json()
        .with_writer(writer)
        .with_timer(UtcTime::new(time_format_description()))
        .with_current_span(true)
        .with_span_list(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .flatten_event(true)
}

/// 時刻フォーマット記述子を取得
const fn time_format_description() -> &'static str {
    // ISO 8601 形式
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
}

/// ロギングエラー
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    /// フィルタのパースに失敗
    #[error("failed to parse filter: {0}")]
    FilterParseFailed(String),

    /// 初期化に失敗
    #[error("failed to initialize logging: {0}")]
    InitFailed(String),
}

/// JSON フォーマッター
///
/// カスタム JSON 出力が必要な場合に使用する。
#[derive(Debug, Clone)]
pub struct JsonFormatter {
    service_name: String,
    service_env: String,
    service_version: Option<String>,
}

impl JsonFormatter {
    /// ObservabilityConfig から作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        Self {
            service_name: config.service_name().to_string(),
            service_env: config.env().to_string(),
            service_version: config.version().map(|s| s.to_string()),
        }
    }

    /// ログエントリを JSON 文字列に変換
    pub fn format(&self, entry: &LogEntry) -> Result<String, serde_json::Error> {
        let mut map = serde_json::Map::new();

        // タイムスタンプ
        map.insert(
            "timestamp".to_string(),
            serde_json::Value::String(entry.timestamp.clone()),
        );

        // レベル
        map.insert(
            "level".to_string(),
            serde_json::Value::String(entry.level.as_str().to_string()),
        );

        // メッセージ
        map.insert(
            "message".to_string(),
            serde_json::Value::String(entry.message.clone()),
        );

        // サービス情報（エントリから、なければ formatter から）
        let service_name = entry
            .service_name
            .clone()
            .unwrap_or_else(|| self.service_name.clone());
        let service_env = entry
            .service_env
            .clone()
            .unwrap_or_else(|| self.service_env.clone());

        map.insert(
            "service.name".to_string(),
            serde_json::Value::String(service_name),
        );
        map.insert(
            "service.env".to_string(),
            serde_json::Value::String(service_env),
        );

        if let Some(ref version) = entry.service_version.clone().or(self.service_version.clone()) {
            map.insert(
                "service.version".to_string(),
                serde_json::Value::String(version.clone()),
            );
        }

        // トレース情報
        if let Some(ref trace_id) = entry.trace_id {
            map.insert(
                "trace.id".to_string(),
                serde_json::Value::String(trace_id.clone()),
            );
        }
        if let Some(ref span_id) = entry.span_id {
            map.insert(
                "span.id".to_string(),
                serde_json::Value::String(span_id.clone()),
            );
        }
        if let Some(ref request_id) = entry.request_id {
            map.insert(
                "request.id".to_string(),
                serde_json::Value::String(request_id.clone()),
            );
        }

        // エラー情報
        if let Some(ref error_kind) = entry.error_kind {
            map.insert(
                "error.kind".to_string(),
                serde_json::Value::String(error_kind.clone()),
            );
        }
        if let Some(ref error_code) = entry.error_code {
            map.insert(
                "error.code".to_string(),
                serde_json::Value::String(error_code.clone()),
            );
        }
        if let Some(ref error_message) = entry.error_message {
            map.insert(
                "error.message".to_string(),
                serde_json::Value::String(error_message.clone()),
            );
        }

        // 追加フィールド
        if let Some(ref extra) = entry.extra {
            if let serde_json::Value::Object(extra_map) = extra {
                for (k, v) in extra_map {
                    map.insert(k.clone(), v.clone());
                }
            }
        }

        serde_json::to_string(&serde_json::Value::Object(map))
    }

    /// コンテキスト付きでフォーマット
    pub fn format_with_context(
        &self,
        entry: &LogEntry,
        ctx: &RequestContext,
    ) -> Result<String, serde_json::Error> {
        let mut entry = entry.clone();
        entry.trace_id = Some(ctx.trace_id().to_string());
        entry.span_id = Some(ctx.span_id().to_string());
        entry.request_id = Some(ctx.request_id().to_string());
        self.format(&entry)
    }
}

#[cfg(test)]
mod tracing_tests {
    use super::*;

    #[test]
    fn test_json_formatter() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let formatter = JsonFormatter::from_config(&config);
        let entry = LogEntry::info("test message");
        let json = formatter.format(&entry).unwrap();

        assert!(json.contains("\"service.name\":\"test-service\""));
        assert!(json.contains("\"service.env\":\"dev\""));
        assert!(json.contains("\"level\":\"INFO\""));
        assert!(json.contains("\"message\":\"test message\""));
    }

    #[test]
    fn test_json_formatter_with_context() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let formatter = JsonFormatter::from_config(&config);
        let entry = LogEntry::info("test message");
        let ctx = RequestContext::new();
        let json = formatter.format_with_context(&entry, &ctx).unwrap();

        assert!(json.contains("trace.id"));
        assert!(json.contains("span.id"));
        assert!(json.contains("request.id"));
    }
}
