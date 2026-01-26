//! ログの必須フィールド
//!
//! 構造化ログ（JSON）の必須フィールドを定義する。

use serde::Serialize;

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    /// トレース
    Trace,
    /// デバッグ
    Debug,
    /// 情報
    Info,
    /// 警告
    Warn,
    /// エラー
    Error,
}

impl LogLevel {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// ログの必須フィールド定義
///
/// すべてのログエントリに含めるべきフィールドを定義。
pub struct LogFields;

impl LogFields {
    // === 基本フィールド ===

    /// タイムスタンプ（ISO 8601）
    pub const TIMESTAMP: &'static str = "timestamp";
    /// ログレベル
    pub const LEVEL: &'static str = "level";
    /// ログメッセージ
    pub const MESSAGE: &'static str = "message";

    // === サービス情報 ===

    /// サービス名
    pub const SERVICE_NAME: &'static str = "service.name";
    /// 環境名
    pub const SERVICE_ENV: &'static str = "service.env";
    /// サービスバージョン
    pub const SERVICE_VERSION: &'static str = "service.version";
    /// インスタンス ID
    pub const SERVICE_INSTANCE: &'static str = "service.instance";

    // === トレース情報 ===

    /// トレース ID
    pub const TRACE_ID: &'static str = "trace.id";
    /// スパン ID
    pub const SPAN_ID: &'static str = "span.id";
    /// 親スパン ID
    pub const PARENT_SPAN_ID: &'static str = "span.parent_id";
    /// リクエスト ID
    pub const REQUEST_ID: &'static str = "request.id";

    // === HTTP 情報 ===

    /// HTTP メソッド
    pub const HTTP_METHOD: &'static str = "http.method";
    /// HTTP パス
    pub const HTTP_PATH: &'static str = "http.path";
    /// HTTP ステータスコード
    pub const HTTP_STATUS_CODE: &'static str = "http.status_code";
    /// HTTP リクエストサイズ
    pub const HTTP_REQUEST_SIZE: &'static str = "http.request.size";
    /// HTTP レスポンスサイズ
    pub const HTTP_RESPONSE_SIZE: &'static str = "http.response.size";

    // === gRPC 情報 ===

    /// gRPC サービス
    pub const GRPC_SERVICE: &'static str = "grpc.service";
    /// gRPC メソッド
    pub const GRPC_METHOD: &'static str = "grpc.method";
    /// gRPC ステータスコード
    pub const GRPC_STATUS_CODE: &'static str = "grpc.status_code";

    // === エラー情報 ===

    /// エラーの種類
    pub const ERROR_KIND: &'static str = "error.kind";
    /// エラーコード
    pub const ERROR_CODE: &'static str = "error.code";
    /// エラーメッセージ
    pub const ERROR_MESSAGE: &'static str = "error.message";

    // === パフォーマンス ===

    /// レイテンシ（ミリ秒）
    pub const LATENCY_MS: &'static str = "latency_ms";
}

/// ログエントリ
///
/// 構造化ログの 1 エントリを表現する。
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// タイムスタンプ（ISO 8601）
    pub timestamp: String,
    /// ログレベル
    pub level: LogLevel,
    /// メッセージ
    pub message: String,

    // === サービス情報 ===
    #[serde(rename = "service.name", skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(rename = "service.env", skip_serializing_if = "Option::is_none")]
    pub service_env: Option<String>,
    #[serde(rename = "service.version", skip_serializing_if = "Option::is_none")]
    pub service_version: Option<String>,

    // === トレース情報 ===
    #[serde(rename = "trace.id", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(rename = "span.id", skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(rename = "request.id", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    // === 追加フィールド ===
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl LogEntry {
    /// 新しいログエントリを作成
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Self::now_iso8601(),
            level,
            message: message.into(),
            service_name: None,
            service_env: None,
            service_version: None,
            trace_id: None,
            span_id: None,
            request_id: None,
            extra: None,
        }
    }

    /// TRACE レベルのログエントリを作成
    pub fn trace(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Trace, message)
    }

    /// DEBUG レベルのログエントリを作成
    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Debug, message)
    }

    /// INFO レベルのログエントリを作成
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, message)
    }

    /// WARN レベルのログエントリを作成
    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Warn, message)
    }

    /// ERROR レベルのログエントリを作成
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, message)
    }

    /// リクエストコンテキストを設定
    pub fn with_context(mut self, ctx: &RequestContext) -> Self {
        self.trace_id = Some(ctx.trace_id().to_string());
        self.span_id = Some(ctx.span_id().to_string());
        self.request_id = Some(ctx.request_id().to_string());
        self
    }

    /// サービス情報を設定
    pub fn with_service(mut self, config: &ObservabilityConfig) -> Self {
        self.service_name = Some(config.service_name().to_string());
        self.service_env = Some(config.env().to_string());
        self.service_version = config.version().map(|s| s.to_string());
        self
    }

    /// 追加フィールドを設定
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }

    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 現在時刻を ISO 8601 形式で取得
    fn now_iso8601() -> String {
        // 簡易実装（実際は chrono 等を使用）
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        format!("{}T{:03}Z", secs, millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, "テストメッセージ");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "テストメッセージ");
        assert!(!entry.timestamp.is_empty());
    }

    #[test]
    fn test_log_entry_levels() {
        assert_eq!(LogEntry::trace("msg").level, LogLevel::Trace);
        assert_eq!(LogEntry::debug("msg").level, LogLevel::Debug);
        assert_eq!(LogEntry::info("msg").level, LogLevel::Info);
        assert_eq!(LogEntry::warn("msg").level, LogLevel::Warn);
        assert_eq!(LogEntry::error("msg").level, LogLevel::Error);
    }

    #[test]
    fn test_log_entry_with_context() {
        let ctx = RequestContext::new();
        let entry = LogEntry::info("test").with_context(&ctx);

        assert_eq!(entry.trace_id, Some(ctx.trace_id().to_string()));
        assert_eq!(entry.span_id, Some(ctx.span_id().to_string()));
        assert_eq!(entry.request_id, Some(ctx.request_id().to_string()));
    }

    #[test]
    fn test_log_entry_with_service() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .version("1.0.0")
            .build()
            .unwrap();

        let entry = LogEntry::info("test").with_service(&config);

        assert_eq!(entry.service_name, Some("test-service".to_string()));
        assert_eq!(entry.service_env, Some("dev".to_string()));
        assert_eq!(entry.service_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_log_entry_to_json() {
        let entry = LogEntry::info("テスト")
            .with_extra(serde_json::json!({"user_id": "123"}));

        let json = entry.to_json().unwrap();
        assert!(json.contains("INFO"));
        assert!(json.contains("テスト"));
        assert!(json.contains("user_id"));
    }

    #[test]
    fn test_log_fields_constants() {
        assert_eq!(LogFields::TIMESTAMP, "timestamp");
        assert_eq!(LogFields::SERVICE_NAME, "service.name");
        assert_eq!(LogFields::TRACE_ID, "trace.id");
        assert_eq!(LogFields::HTTP_STATUS_CODE, "http.status_code");
    }
}
