//! gRPC インターセプタ
//!
//! 共通のインターセプタを提供する:
//! - トレースコンテキスト伝播
//! - error_code 受け渡し
//! - request_id 相関
//! - テナント情報付与

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// エラーコード
    pub const X_ERROR_CODE: &'static str = "x-error-code";
    /// エラー詳細（バイナリ）
    pub const X_ERROR_DETAILS_BIN: &'static str = "x-error-details-bin";
    /// ユーザー ID
    pub const X_USER_ID: &'static str = "x-user-id";
    /// デッドライン（ミリ秒、Unix epoch）
    pub const X_DEADLINE_MS: &'static str = "x-deadline-ms";
}

/// リクエストメタデータ
///
/// gRPC リクエストに付与するメタデータを管理する。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// トレース ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// スパン ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    /// 親スパン ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    /// リクエスト ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// テナント ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// ユーザー ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// デッドライン（Unix epoch ミリ秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<u64>,
    /// カスタムメタデータ
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, String>,
}

impl RequestMetadata {
    /// 新しいメタデータを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// トレース ID を設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// スパン ID を設定
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// 親スパン ID を設定
    pub fn with_parent_span_id(mut self, parent_span_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_span_id.into());
        self
    }

    /// リクエスト ID を設定
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// テナント ID を設定
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// ユーザー ID を設定
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// デッドラインを設定
    pub fn with_deadline_ms(mut self, deadline_ms: u64) -> Self {
        self.deadline_ms = Some(deadline_ms);
        self
    }

    /// カスタムメタデータを追加
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }

    /// traceparent ヘッダ値を生成
    ///
    /// 形式: `00-{trace_id}-{span_id}-01`
    pub fn to_traceparent(&self) -> Option<String> {
        match (&self.trace_id, &self.span_id) {
            (Some(trace_id), Some(span_id)) => {
                Some(format!("00-{}-{}-01", trace_id, span_id))
            }
            _ => None,
        }
    }

    /// gRPC メタデータに変換
    pub fn to_metadata_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        // traceparent
        if let Some(traceparent) = self.to_traceparent() {
            map.insert(MetadataKeys::TRACEPARENT.to_string(), traceparent);
        }

        // trace_id（別途）
        if let Some(ref trace_id) = self.trace_id {
            map.insert(MetadataKeys::X_TRACE_ID.to_string(), trace_id.clone());
        }

        // request_id
        if let Some(ref request_id) = self.request_id {
            map.insert(MetadataKeys::X_REQUEST_ID.to_string(), request_id.clone());
        }

        // tenant_id
        if let Some(ref tenant_id) = self.tenant_id {
            map.insert(MetadataKeys::X_TENANT_ID.to_string(), tenant_id.clone());
        }

        // user_id
        if let Some(ref user_id) = self.user_id {
            map.insert(MetadataKeys::X_USER_ID.to_string(), user_id.clone());
        }

        // deadline
        if let Some(deadline_ms) = self.deadline_ms {
            map.insert(
                MetadataKeys::X_DEADLINE_MS.to_string(),
                deadline_ms.to_string(),
            );
        }

        // カスタムメタデータ
        for (key, value) in &self.custom {
            map.insert(key.clone(), value.clone());
        }

        map
    }

    /// gRPC メタデータから作成
    pub fn from_metadata_map(map: &HashMap<String, String>) -> Self {
        let mut metadata = Self::new();

        // traceparent から trace_id と parent_span_id を抽出
        if let Some(traceparent) = map.get(MetadataKeys::TRACEPARENT) {
            if let Some((trace_id, parent_span_id)) = Self::parse_traceparent(traceparent) {
                metadata.trace_id = Some(trace_id);
                metadata.parent_span_id = Some(parent_span_id);
            }
        }

        // 個別のフィールド
        if let Some(trace_id) = map.get(MetadataKeys::X_TRACE_ID) {
            metadata.trace_id = Some(trace_id.clone());
        }

        if let Some(request_id) = map.get(MetadataKeys::X_REQUEST_ID) {
            metadata.request_id = Some(request_id.clone());
        }

        if let Some(tenant_id) = map.get(MetadataKeys::X_TENANT_ID) {
            metadata.tenant_id = Some(tenant_id.clone());
        }

        if let Some(user_id) = map.get(MetadataKeys::X_USER_ID) {
            metadata.user_id = Some(user_id.clone());
        }

        if let Some(deadline_ms) = map.get(MetadataKeys::X_DEADLINE_MS) {
            metadata.deadline_ms = deadline_ms.parse().ok();
        }

        metadata
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
}

/// レスポンスメタデータ
///
/// gRPC レスポンスから抽出するメタデータを管理する。
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

    /// gRPC メタデータから作成
    pub fn from_metadata_map(map: &HashMap<String, String>) -> Self {
        let mut metadata = Self::new();

        if let Some(error_code) = map.get(MetadataKeys::X_ERROR_CODE) {
            metadata.error_code = Some(error_code.clone());
        }

        if let Some(trace_id) = map.get(MetadataKeys::X_TRACE_ID) {
            metadata.trace_id = Some(trace_id.clone());
        }

        if let Some(request_id) = map.get(MetadataKeys::X_REQUEST_ID) {
            metadata.request_id = Some(request_id.clone());
        }

        metadata
    }

    /// エラーコードがあるかどうか
    pub fn has_error_code(&self) -> bool {
        self.error_code.is_some()
    }
}

/// インターセプタコンテキスト
///
/// インターセプタ間で共有するコンテキスト情報。
#[derive(Debug, Clone)]
pub struct InterceptorContext {
    /// リクエストメタデータ
    pub request: RequestMetadata,
    /// サービス名
    pub service_name: String,
    /// 呼び出し先サービス名
    pub target_service: String,
    /// メソッド名
    pub method: String,
    /// 開始時刻（Unix epoch ミリ秒）
    pub start_time_ms: u64,
}

impl InterceptorContext {
    /// 新しいコンテキストを作成
    pub fn new(
        request: RequestMetadata,
        service_name: impl Into<String>,
        target_service: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            request,
            service_name: service_name.into(),
            target_service: target_service.into(),
            method: method.into(),
            start_time_ms: Self::now_ms(),
        }
    }

    /// 経過時間（ミリ秒）を取得
    pub fn elapsed_ms(&self) -> u64 {
        Self::now_ms().saturating_sub(self.start_time_ms)
    }

    fn now_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// インターセプタ結果
#[derive(Debug, Clone)]
pub struct InterceptorResult {
    /// 成功かどうか
    pub success: bool,
    /// レスポンスメタデータ
    pub response: ResponseMetadata,
    /// gRPC ステータスコード
    pub status_code: i32,
    /// レイテンシ（ミリ秒）
    pub latency_ms: u64,
}

impl InterceptorResult {
    /// 成功結果を作成
    pub fn success(response: ResponseMetadata, latency_ms: u64) -> Self {
        Self {
            success: true,
            response,
            status_code: 0,
            latency_ms,
        }
    }

    /// 失敗結果を作成
    pub fn failure(response: ResponseMetadata, status_code: i32, latency_ms: u64) -> Self {
        Self {
            success: false,
            response,
            status_code,
            latency_ms,
        }
    }
}

/// メトリクスラベル
///
/// gRPC クライアントメトリクスのラベル。
#[derive(Debug, Clone)]
pub struct ClientMetricLabels {
    /// サービス名（呼び出し元）
    pub service: String,
    /// ターゲットサービス名（呼び出し先）
    pub target_service: String,
    /// gRPC メソッド名
    pub method: String,
    /// gRPC ステータス名
    pub status: String,
}

impl ClientMetricLabels {
    /// コンテキストと結果から作成
    pub fn from_context_and_result(ctx: &InterceptorContext, result: &InterceptorResult) -> Self {
        Self {
            service: ctx.service_name.clone(),
            target_service: ctx.target_service.clone(),
            method: ctx.method.clone(),
            status: Self::status_name(result.status_code),
        }
    }

    fn status_name(code: i32) -> String {
        match code {
            0 => "OK",
            1 => "CANCELLED",
            2 => "UNKNOWN",
            3 => "INVALID_ARGUMENT",
            4 => "DEADLINE_EXCEEDED",
            5 => "NOT_FOUND",
            6 => "ALREADY_EXISTS",
            7 => "PERMISSION_DENIED",
            8 => "RESOURCE_EXHAUSTED",
            9 => "FAILED_PRECONDITION",
            10 => "ABORTED",
            11 => "OUT_OF_RANGE",
            12 => "UNIMPLEMENTED",
            13 => "INTERNAL",
            14 => "UNAVAILABLE",
            15 => "DATA_LOSS",
            16 => "UNAUTHENTICATED",
            _ => "UNKNOWN",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metadata_new() {
        let metadata = RequestMetadata::new();
        assert!(metadata.trace_id.is_none());
        assert!(metadata.request_id.is_none());
    }

    #[test]
    fn test_request_metadata_builder() {
        let metadata = RequestMetadata::new()
            .with_trace_id("abc123")
            .with_span_id("def456")
            .with_request_id("req-001")
            .with_tenant_id("tenant-1")
            .with_user_id("user-1")
            .with_deadline_ms(1000);

        assert_eq!(metadata.trace_id, Some("abc123".to_string()));
        assert_eq!(metadata.span_id, Some("def456".to_string()));
        assert_eq!(metadata.request_id, Some("req-001".to_string()));
        assert_eq!(metadata.tenant_id, Some("tenant-1".to_string()));
        assert_eq!(metadata.user_id, Some("user-1".to_string()));
        assert_eq!(metadata.deadline_ms, Some(1000));
    }

    #[test]
    fn test_request_metadata_to_traceparent() {
        let metadata = RequestMetadata::new()
            .with_trace_id("0af7651916cd43dd8448eb211c80319c")
            .with_span_id("b7ad6b7169203331");

        let traceparent = metadata.to_traceparent().unwrap();
        assert_eq!(
            traceparent,
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
        );
    }

    #[test]
    fn test_request_metadata_to_metadata_map() {
        let metadata = RequestMetadata::new()
            .with_trace_id("abc123")
            .with_span_id("def456")
            .with_request_id("req-001")
            .with_tenant_id("tenant-1");

        let map = metadata.to_metadata_map();

        assert!(map.contains_key(MetadataKeys::TRACEPARENT));
        assert_eq!(map.get(MetadataKeys::X_REQUEST_ID), Some(&"req-001".to_string()));
        assert_eq!(map.get(MetadataKeys::X_TENANT_ID), Some(&"tenant-1".to_string()));
    }

    #[test]
    fn test_request_metadata_from_metadata_map() {
        let mut map = HashMap::new();
        map.insert(
            MetadataKeys::TRACEPARENT.to_string(),
            "00-abc123-def456-01".to_string(),
        );
        map.insert(
            MetadataKeys::X_REQUEST_ID.to_string(),
            "req-001".to_string(),
        );
        map.insert(
            MetadataKeys::X_TENANT_ID.to_string(),
            "tenant-1".to_string(),
        );

        let metadata = RequestMetadata::from_metadata_map(&map);

        assert_eq!(metadata.trace_id, Some("abc123".to_string()));
        assert_eq!(metadata.parent_span_id, Some("def456".to_string()));
        assert_eq!(metadata.request_id, Some("req-001".to_string()));
        assert_eq!(metadata.tenant_id, Some("tenant-1".to_string()));
    }

    #[test]
    fn test_request_metadata_custom() {
        let metadata = RequestMetadata::new()
            .with_custom("x-custom-header", "custom-value");

        let map = metadata.to_metadata_map();
        assert_eq!(map.get("x-custom-header"), Some(&"custom-value".to_string()));
    }

    #[test]
    fn test_response_metadata_from_map() {
        let mut map = HashMap::new();
        map.insert(MetadataKeys::X_ERROR_CODE.to_string(), "USER_NOT_FOUND".to_string());
        map.insert(MetadataKeys::X_TRACE_ID.to_string(), "abc123".to_string());

        let metadata = ResponseMetadata::from_metadata_map(&map);

        assert_eq!(metadata.error_code, Some("USER_NOT_FOUND".to_string()));
        assert_eq!(metadata.trace_id, Some("abc123".to_string()));
        assert!(metadata.has_error_code());
    }

    #[test]
    fn test_interceptor_context() {
        let request = RequestMetadata::new().with_trace_id("abc123");
        let ctx = InterceptorContext::new(
            request,
            "my-service",
            "auth-service",
            "GetUser",
        );

        assert_eq!(ctx.service_name, "my-service");
        assert_eq!(ctx.target_service, "auth-service");
        assert_eq!(ctx.method, "GetUser");
        assert!(ctx.start_time_ms > 0);
    }

    #[test]
    fn test_interceptor_result_success() {
        let response = ResponseMetadata::new();
        let result = InterceptorResult::success(response, 100);

        assert!(result.success);
        assert_eq!(result.status_code, 0);
        assert_eq!(result.latency_ms, 100);
    }

    #[test]
    fn test_interceptor_result_failure() {
        let mut response = ResponseMetadata::new();
        response.error_code = Some("USER_NOT_FOUND".to_string());
        let result = InterceptorResult::failure(response, 5, 50);

        assert!(!result.success);
        assert_eq!(result.status_code, 5);
        assert!(result.response.has_error_code());
    }

    #[test]
    fn test_client_metric_labels() {
        let request = RequestMetadata::new();
        let ctx = InterceptorContext::new(request, "my-service", "auth-service", "GetUser");
        let result = InterceptorResult::success(ResponseMetadata::new(), 100);

        let labels = ClientMetricLabels::from_context_and_result(&ctx, &result);

        assert_eq!(labels.service, "my-service");
        assert_eq!(labels.target_service, "auth-service");
        assert_eq!(labels.method, "GetUser");
        assert_eq!(labels.status, "OK");
    }

    #[test]
    fn test_metadata_keys() {
        assert_eq!(MetadataKeys::TRACEPARENT, "traceparent");
        assert_eq!(MetadataKeys::X_REQUEST_ID, "x-request-id");
        assert_eq!(MetadataKeys::X_ERROR_CODE, "x-error-code");
    }
}
