//! ロギング
//!
//! 構造化ログの初期化と出力を提供する。

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;
use crate::log_fields::{LogEntry, LogLevel};

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
