//! gRPC サーバインターセプタ
//!
//! サーバ側の共通インターセプタを提供する:
//! - トレースコンテキスト伝播
//! - error_code/status 統一
//! - request_id 相関
//! - テナント情報読み取り
//! - デッドライン検知

use crate::error::{GrpcStatusCode, LogLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// gRPC メタデータキー
pub struct MetadataKeys;

impl MetadataKeys {
    /// W3C Trace Context traceparent
    pub const TRACEPARENT: &'static str = "traceparent";
    /// W3C Trace Context tracestate
    pub const TRACESTATE: &'static str = "tracestate";
    /// k1s0 リクエスト ID
    pub const X_REQUEST_ID: &'static str = "x-request-id";
    /// k1s0 トレース ID
    pub const X_TRACE_ID: &'static str = "x-trace-id";
    /// テナント ID
    pub const X_TENANT_ID: &'static str = "x-tenant-id";
    /// ユーザー ID
    pub const X_USER_ID: &'static str = "x-user-id";
    /// エラーコード
    pub const X_ERROR_CODE: &'static str = "x-error-code";
    /// エラー詳細（バイナリ）
    pub const X_ERROR_DETAILS_BIN: &'static str = "x-error-details-bin";
    /// デッドライン（ミリ秒）
    pub const GRPC_TIMEOUT: &'static str = "grpc-timeout";
}

/// リクエストコンテキスト
///
/// リクエストごとの相関情報を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// トレース ID
    pub trace_id: String,
    /// スパン ID
    pub span_id: String,
    /// 親スパン ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    /// リクエスト ID
    pub request_id: String,
    /// テナント ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// ユーザー ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// デッドライン（Unix epoch ミリ秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<u64>,
    /// デッドラインが指定されているかどうか
    pub has_deadline: bool,
}

impl RequestContext {
    /// 新しいコンテキストを作成
    pub fn new() -> Self {
        Self {
            trace_id: Self::generate_trace_id(),
            span_id: Self::generate_span_id(),
            parent_span_id: None,
            request_id: Self::generate_request_id(),
            tenant_id: None,
            user_id: None,
            deadline_ms: None,
            has_deadline: false,
        }
    }

    /// メタデータから作成
    pub fn from_metadata(metadata: &HashMap<String, String>) -> Self {
        let mut ctx = Self::new();

        // traceparent から trace_id と parent_span_id を抽出
        if let Some(traceparent) = metadata.get(MetadataKeys::TRACEPARENT) {
            if let Some((trace_id, parent_span_id)) = Self::parse_traceparent(traceparent) {
                ctx.trace_id = trace_id;
                ctx.parent_span_id = Some(parent_span_id);
            }
        }

        // 個別のフィールド
        if let Some(trace_id) = metadata.get(MetadataKeys::X_TRACE_ID) {
            ctx.trace_id = trace_id.clone();
        }

        if let Some(request_id) = metadata.get(MetadataKeys::X_REQUEST_ID) {
            ctx.request_id = request_id.clone();
        }

        if let Some(tenant_id) = metadata.get(MetadataKeys::X_TENANT_ID) {
            ctx.tenant_id = Some(tenant_id.clone());
        }

        if let Some(user_id) = metadata.get(MetadataKeys::X_USER_ID) {
            ctx.user_id = Some(user_id.clone());
        }

        // gRPC タイムアウト（grpc-timeout ヘッダ）
        if let Some(timeout) = metadata.get(MetadataKeys::GRPC_TIMEOUT) {
            if let Some(duration_ms) = Self::parse_grpc_timeout(timeout) {
                let now_ms = Self::now_ms();
                ctx.deadline_ms = Some(now_ms + duration_ms);
                ctx.has_deadline = true;
            }
        }

        ctx
    }

    /// traceparent をパース
    fn parse_traceparent(traceparent: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = traceparent.split('-').collect();
        if parts.len() >= 3 {
            Some((parts[1].to_string(), parts[2].to_string()))
        } else {
            None
        }
    }

    /// grpc-timeout をパース
    ///
    /// 形式: `{value}{unit}` (例: "1000m" = 1000 milliseconds, "1S" = 1 second)
    fn parse_grpc_timeout(timeout: &str) -> Option<u64> {
        if timeout.is_empty() {
            return None;
        }

        let (value_str, unit) = timeout.split_at(timeout.len() - 1);
        let value: u64 = value_str.parse().ok()?;

        let ms = match unit {
            "n" => value / 1_000_000, // nanoseconds
            "u" => value / 1_000,     // microseconds
            "m" => value,             // milliseconds
            "S" => value * 1_000,     // seconds
            "M" => value * 60_000,    // minutes
            "H" => value * 3_600_000, // hours
            _ => return None,
        };

        Some(ms)
    }

    /// traceparent ヘッダ値を生成
    pub fn to_traceparent(&self) -> String {
        format!("00-{}-{}-01", self.trace_id, self.span_id)
    }

    /// 残りデッドライン時間を取得
    pub fn remaining_deadline(&self) -> Option<Duration> {
        self.deadline_ms.map(|deadline| {
            let now = Self::now_ms();
            if deadline > now {
                Duration::from_millis(deadline - now)
            } else {
                Duration::ZERO
            }
        })
    }

    /// デッドラインを超過しているかどうか
    pub fn is_deadline_exceeded(&self) -> bool {
        if let Some(deadline) = self.deadline_ms {
            Self::now_ms() >= deadline
        } else {
            false
        }
    }

    fn generate_trace_id() -> String {
        format!("{:032x}", uuid::Uuid::new_v4().as_u128())
    }

    fn generate_span_id() -> String {
        format!("{:016x}", uuid::Uuid::new_v4().as_u128() >> 64)
    }

    fn generate_request_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn now_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

// UUID モジュール（簡易実装）
mod uuid {
    pub struct Uuid(u128);

    impl Uuid {
        pub fn new_v4() -> Self {
            use std::time::{SystemTime, UNIX_EPOCH};

            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u128;

            // 簡易的な UUID v4 風の生成（実運用では uuid crate を使用）
            let random = (time ^ (time >> 64)) | 0x4000_0000_0000_0000_8000_0000_0000_0000u128;
            Self(random)
        }

        pub fn as_u128(&self) -> u128 {
            self.0
        }

        pub fn to_string(&self) -> String {
            let bytes = self.0.to_be_bytes();
            format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5],
                bytes[6], bytes[7],
                bytes[8], bytes[9],
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            )
        }
    }
}

/// レスポンスメタデータ
///
/// レスポンスに付与するメタデータを管理する。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// エラーコード
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// トレース ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// リクエスト ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// カスタムメタデータ
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, String>,
}

impl ResponseMetadata {
    /// 新しいレスポンスメタデータを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// コンテキストから作成
    pub fn from_context(ctx: &RequestContext) -> Self {
        Self {
            error_code: None,
            trace_id: Some(ctx.trace_id.clone()),
            request_id: Some(ctx.request_id.clone()),
            custom: HashMap::new(),
        }
    }

    /// エラーコードを設定
    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    /// gRPC メタデータに変換
    pub fn to_metadata_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if let Some(ref error_code) = self.error_code {
            map.insert(MetadataKeys::X_ERROR_CODE.to_string(), error_code.clone());
        }

        if let Some(ref trace_id) = self.trace_id {
            map.insert(MetadataKeys::X_TRACE_ID.to_string(), trace_id.clone());
        }

        if let Some(ref request_id) = self.request_id {
            map.insert(MetadataKeys::X_REQUEST_ID.to_string(), request_id.clone());
        }

        for (key, value) in &self.custom {
            map.insert(key.clone(), value.clone());
        }

        map
    }
}

/// インターセプタ処理結果
#[derive(Debug, Clone)]
pub struct InterceptorResult {
    /// 処理を続行するかどうか
    pub continue_processing: bool,
    /// エラー（処理を中断する場合）
    pub error: Option<InterceptorError>,
}

impl InterceptorResult {
    /// 続行結果を作成
    pub fn continue_() -> Self {
        Self {
            continue_processing: true,
            error: None,
        }
    }

    /// 中断結果を作成
    pub fn abort(error: InterceptorError) -> Self {
        Self {
            continue_processing: false,
            error: Some(error),
        }
    }
}

/// インターセプタエラー
#[derive(Debug, Clone)]
pub struct InterceptorError {
    /// gRPC ステータスコード
    pub status_code: GrpcStatusCode,
    /// エラーメッセージ
    pub message: String,
    /// error_code
    pub error_code: Option<String>,
}

impl InterceptorError {
    /// 新しいエラーを作成
    pub fn new(status_code: GrpcStatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
            error_code: None,
        }
    }

    /// error_code 付きのエラーを作成
    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    /// デッドライン未指定エラー
    pub fn deadline_not_specified() -> Self {
        Self::new(
            GrpcStatusCode::InvalidArgument,
            "deadline is required but not specified",
        )
        .with_error_code("DEADLINE_NOT_SPECIFIED")
    }

    /// デッドライン超過エラー
    pub fn deadline_exceeded() -> Self {
        Self::new(GrpcStatusCode::DeadlineExceeded, "deadline exceeded")
            .with_error_code("DEADLINE_EXCEEDED")
    }
}

/// リクエストログ
#[derive(Debug, Clone, Serialize)]
pub struct RequestLog {
    /// タイムスタンプ
    pub timestamp: String,
    /// ログレベル
    pub level: String,
    /// メッセージ
    pub message: String,
    /// サービス名
    pub service_name: String,
    /// 環境
    pub env: String,
    /// トレース ID
    pub trace_id: String,
    /// スパン ID
    pub span_id: String,
    /// リクエスト ID
    pub request_id: String,
    /// gRPC サービス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_service: Option<String>,
    /// gRPC メソッド
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_method: Option<String>,
    /// gRPC ステータスコード
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_status: Option<i32>,
    /// gRPC ステータス名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_status_name: Option<String>,
    /// レイテンシ（ミリ秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<f64>,
    /// error_code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// テナント ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl RequestLog {
    /// 新しいリクエストログを作成
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        service_name: impl Into<String>,
        env: impl Into<String>,
        ctx: &RequestContext,
    ) -> Self {
        Self {
            timestamp: Self::now_iso8601(),
            level: level.as_str().to_string(),
            message: message.into(),
            service_name: service_name.into(),
            env: env.into(),
            trace_id: ctx.trace_id.clone(),
            span_id: ctx.span_id.clone(),
            request_id: ctx.request_id.clone(),
            grpc_service: None,
            grpc_method: None,
            grpc_status: None,
            grpc_status_name: None,
            latency_ms: None,
            error_code: None,
            tenant_id: ctx.tenant_id.clone(),
        }
    }

    /// gRPC 情報を設定
    pub fn with_grpc(
        mut self,
        service: impl Into<String>,
        method: impl Into<String>,
        status: GrpcStatusCode,
    ) -> Self {
        self.grpc_service = Some(service.into());
        self.grpc_method = Some(method.into());
        self.grpc_status = Some(status as i32);
        self.grpc_status_name = Some(status.name().to_string());
        self
    }

    /// レイテンシを設定
    pub fn with_latency(mut self, latency_ms: f64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// error_code を設定
    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    /// JSON に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn now_iso8601() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        format!(
            "{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            1970 + secs / 31536000,
            (secs % 31536000) / 2592000 + 1,
            (secs % 2592000) / 86400 + 1,
            (secs % 86400) / 3600,
            (secs % 3600) / 60,
            secs % 60,
            millis
        )
    }
}

/// メトリクスラベル
#[derive(Debug, Clone)]
pub struct ServerMetricLabels {
    /// サービス名
    pub service: String,
    /// 環境
    pub env: String,
    /// gRPC サービス名
    pub grpc_service: String,
    /// gRPC メソッド名
    pub grpc_method: String,
    /// gRPC ステータス名
    pub grpc_status: String,
}

impl ServerMetricLabels {
    /// 新しいラベルを作成
    pub fn new(
        service: impl Into<String>,
        env: impl Into<String>,
        grpc_service: impl Into<String>,
        grpc_method: impl Into<String>,
        grpc_status: GrpcStatusCode,
    ) -> Self {
        Self {
            service: service.into(),
            env: env.into(),
            grpc_service: grpc_service.into(),
            grpc_method: grpc_method.into(),
            grpc_status: grpc_status.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new();
        assert_eq!(ctx.trace_id.len(), 32);
        assert_eq!(ctx.span_id.len(), 16);
        assert!(!ctx.request_id.is_empty());
        assert!(!ctx.has_deadline);
    }

    #[test]
    fn test_request_context_from_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert(
            MetadataKeys::TRACEPARENT.to_string(),
            "00-abc123def456-789012-01".to_string(),
        );
        metadata.insert(
            MetadataKeys::X_REQUEST_ID.to_string(),
            "req-001".to_string(),
        );
        metadata.insert(
            MetadataKeys::X_TENANT_ID.to_string(),
            "tenant-1".to_string(),
        );

        let ctx = RequestContext::from_metadata(&metadata);

        assert_eq!(ctx.trace_id, "abc123def456");
        assert_eq!(ctx.parent_span_id, Some("789012".to_string()));
        assert_eq!(ctx.request_id, "req-001");
        assert_eq!(ctx.tenant_id, Some("tenant-1".to_string()));
    }

    #[test]
    fn test_request_context_parse_grpc_timeout() {
        assert_eq!(
            RequestContext::parse_grpc_timeout("1000m"),
            Some(1000)
        );
        assert_eq!(
            RequestContext::parse_grpc_timeout("1S"),
            Some(1000)
        );
        assert_eq!(
            RequestContext::parse_grpc_timeout("1M"),
            Some(60000)
        );
        assert_eq!(
            RequestContext::parse_grpc_timeout("1H"),
            Some(3600000)
        );
    }

    #[test]
    fn test_request_context_to_traceparent() {
        let mut ctx = RequestContext::new();
        ctx.trace_id = "abc123".to_string();
        ctx.span_id = "def456".to_string();

        assert_eq!(ctx.to_traceparent(), "00-abc123-def456-01");
    }

    #[test]
    fn test_response_metadata_from_context() {
        let ctx = RequestContext::new();
        let resp = ResponseMetadata::from_context(&ctx);

        assert_eq!(resp.trace_id, Some(ctx.trace_id.clone()));
        assert_eq!(resp.request_id, Some(ctx.request_id.clone()));
    }

    #[test]
    fn test_response_metadata_with_error_code() {
        let resp = ResponseMetadata::new().with_error_code("USER_NOT_FOUND");
        assert_eq!(resp.error_code, Some("USER_NOT_FOUND".to_string()));
    }

    #[test]
    fn test_interceptor_result_continue() {
        let result = InterceptorResult::continue_();
        assert!(result.continue_processing);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_interceptor_result_abort() {
        let error = InterceptorError::deadline_not_specified();
        let result = InterceptorResult::abort(error);

        assert!(!result.continue_processing);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_interceptor_error_deadline_not_specified() {
        let error = InterceptorError::deadline_not_specified();
        assert_eq!(error.status_code, GrpcStatusCode::InvalidArgument);
        assert_eq!(error.error_code, Some("DEADLINE_NOT_SPECIFIED".to_string()));
    }

    #[test]
    fn test_request_log() {
        let ctx = RequestContext::new();
        let log = RequestLog::new(LogLevel::Info, "test message", "test-service", "dev", &ctx)
            .with_grpc("UserService", "GetUser", GrpcStatusCode::Ok)
            .with_latency(42.5);

        let json = log.to_json().unwrap();
        assert!(json.contains("test-service"));
        assert!(json.contains("UserService"));
        assert!(json.contains("42.5"));
    }

    #[test]
    fn test_server_metric_labels() {
        let labels = ServerMetricLabels::new(
            "test-service",
            "dev",
            "UserService",
            "GetUser",
            GrpcStatusCode::Ok,
        );

        assert_eq!(labels.service, "test-service");
        assert_eq!(labels.grpc_service, "UserService");
        assert_eq!(labels.grpc_status, "OK");
    }

    #[test]
    fn test_metadata_keys() {
        assert_eq!(MetadataKeys::TRACEPARENT, "traceparent");
        assert_eq!(MetadataKeys::X_ERROR_CODE, "x-error-code");
    }
}
