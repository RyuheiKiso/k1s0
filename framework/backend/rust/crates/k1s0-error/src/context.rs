//! エラーコンテキスト
//!
//! trace_id、request_id 等の相関情報を保持する。

use serde::{Deserialize, Serialize};

/// エラーコンテキスト
///
/// エラーに付随する相関情報を保持する。
/// 調査の入口（ログ/トレース）へ確実につながる状態を実現する。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorContext {
    /// トレース ID（分散トレーシング用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// リクエスト ID（リクエスト相関用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// スパン ID（分散トレーシング用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,

    /// テナント ID（マルチテナント用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    /// ユーザー ID（監査用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// サービス名（発生元の特定用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
}

impl ErrorContext {
    /// 新しい空のコンテキストを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// トレース ID を設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// リクエスト ID を設定
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// スパン ID を設定
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
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

    /// サービス名を設定
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    /// コンテキストが空かどうか
    pub fn is_empty(&self) -> bool {
        self.trace_id.is_none()
            && self.request_id.is_none()
            && self.span_id.is_none()
            && self.tenant_id.is_none()
            && self.user_id.is_none()
            && self.service_name.is_none()
    }

    /// 別のコンテキストとマージ（self が優先）
    pub fn merge(self, other: &ErrorContext) -> Self {
        Self {
            trace_id: self.trace_id.or_else(|| other.trace_id.clone()),
            request_id: self.request_id.or_else(|| other.request_id.clone()),
            span_id: self.span_id.or_else(|| other.span_id.clone()),
            tenant_id: self.tenant_id.or_else(|| other.tenant_id.clone()),
            user_id: self.user_id.or_else(|| other.user_id.clone()),
            service_name: self.service_name.or_else(|| other.service_name.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ctx = ErrorContext::new();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_with_trace_id() {
        let ctx = ErrorContext::new().with_trace_id("trace-123");
        assert_eq!(ctx.trace_id, Some("trace-123".to_string()));
        assert!(!ctx.is_empty());
    }

    #[test]
    fn test_with_request_id() {
        let ctx = ErrorContext::new().with_request_id("req-456");
        assert_eq!(ctx.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_builder_chain() {
        let ctx = ErrorContext::new()
            .with_trace_id("trace-123")
            .with_request_id("req-456")
            .with_tenant_id("tenant-789")
            .with_service_name("user-service");

        assert_eq!(ctx.trace_id, Some("trace-123".to_string()));
        assert_eq!(ctx.request_id, Some("req-456".to_string()));
        assert_eq!(ctx.tenant_id, Some("tenant-789".to_string()));
        assert_eq!(ctx.service_name, Some("user-service".to_string()));
    }

    #[test]
    fn test_merge() {
        let ctx1 = ErrorContext::new()
            .with_trace_id("trace-1")
            .with_request_id("req-1");

        let ctx2 = ErrorContext::new()
            .with_trace_id("trace-2")
            .with_tenant_id("tenant-2");

        let merged = ctx1.merge(&ctx2);

        // ctx1 が優先される
        assert_eq!(merged.trace_id, Some("trace-1".to_string()));
        assert_eq!(merged.request_id, Some("req-1".to_string()));
        // ctx2 から補完される
        assert_eq!(merged.tenant_id, Some("tenant-2".to_string()));
    }

    #[test]
    fn test_serialize() {
        let ctx = ErrorContext::new()
            .with_trace_id("trace-123")
            .with_request_id("req-456");

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("trace-123"));
        assert!(json.contains("req-456"));
        // None のフィールドは含まれない
        assert!(!json.contains("tenant_id"));
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{"trace_id":"trace-123","request_id":"req-456"}"#;
        let ctx: ErrorContext = serde_json::from_str(json).unwrap();
        assert_eq!(ctx.trace_id, Some("trace-123".to_string()));
        assert_eq!(ctx.request_id, Some("req-456".to_string()));
        assert_eq!(ctx.tenant_id, None);
    }
}
