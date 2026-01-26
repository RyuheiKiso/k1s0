//! リクエストコンテキスト
//!
//! リクエストごとの相関情報（trace_id, request_id 等）を管理する。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// リクエストコンテキスト
///
/// リクエストごとの相関情報を保持する。
/// ログ、トレース、メトリクスで共通して使用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// トレース ID（分散トレーシング用、W3C Trace Context 互換）
    trace_id: String,
    /// スパン ID
    span_id: String,
    /// リクエスト ID（k1s0 独自、ログ相関用）
    request_id: String,
    /// 親スパン ID（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_span_id: Option<String>,
    /// テナント ID（マルチテナント用）
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<String>,
    /// ユーザー ID（認証済みの場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

impl RequestContext {
    /// 新しいリクエストコンテキストを作成
    ///
    /// trace_id, span_id, request_id を自動生成。
    pub fn new() -> Self {
        Self {
            trace_id: Self::generate_trace_id(),
            span_id: Self::generate_span_id(),
            request_id: Self::generate_request_id(),
            parent_span_id: None,
            tenant_id: None,
            user_id: None,
        }
    }

    /// 指定したトレース ID でコンテキストを作成
    ///
    /// 上流サービスからトレース ID を引き継ぐ場合に使用。
    pub fn with_trace_id(trace_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: Self::generate_span_id(),
            request_id: Self::generate_request_id(),
            parent_span_id: None,
            tenant_id: None,
            user_id: None,
        }
    }

    /// 親コンテキストから子コンテキストを作成
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Self::generate_span_id(),
            request_id: self.request_id.clone(),
            parent_span_id: Some(self.span_id.clone()),
            tenant_id: self.tenant_id.clone(),
            user_id: self.user_id.clone(),
        }
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

    /// トレース ID を取得
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// スパン ID を取得
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// リクエスト ID を取得
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// 親スパン ID を取得
    pub fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }

    /// テナント ID を取得
    pub fn tenant_id(&self) -> Option<&str> {
        self.tenant_id.as_deref()
    }

    /// ユーザー ID を取得
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// W3C Trace Context の traceparent ヘッダ値を生成
    ///
    /// 形式: `00-{trace_id}-{span_id}-01`
    pub fn to_traceparent(&self) -> String {
        format!("00-{}-{}-01", self.trace_id, self.span_id)
    }

    /// traceparent ヘッダからコンテキストを作成
    pub fn from_traceparent(traceparent: &str) -> Option<Self> {
        let parts: Vec<&str> = traceparent.split('-').collect();
        if parts.len() >= 3 {
            let trace_id = parts[1].to_string();
            let parent_span_id = parts[2].to_string();

            Some(Self {
                trace_id,
                span_id: Self::generate_span_id(),
                request_id: Self::generate_request_id(),
                parent_span_id: Some(parent_span_id),
                tenant_id: None,
                user_id: None,
            })
        } else {
            None
        }
    }

    /// トレース ID を生成（32 文字の 16 進数）
    fn generate_trace_id() -> String {
        format!("{:032x}", Uuid::new_v4().as_u128())
    }

    /// スパン ID を生成（16 文字の 16 進数）
    fn generate_span_id() -> String {
        format!("{:016x}", Uuid::new_v4().as_u128() >> 64)
    }

    /// リクエスト ID を生成
    fn generate_request_id() -> String {
        Uuid::new_v4().to_string()
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP ヘッダ名
pub struct HeaderNames;

impl HeaderNames {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ctx = RequestContext::new();
        assert_eq!(ctx.trace_id().len(), 32);
        assert_eq!(ctx.span_id().len(), 16);
        assert!(!ctx.request_id().is_empty());
    }

    #[test]
    fn test_with_trace_id() {
        let ctx = RequestContext::with_trace_id("abc123");
        assert_eq!(ctx.trace_id(), "abc123");
    }

    #[test]
    fn test_child() {
        let parent = RequestContext::new();
        let child = parent.child();

        // trace_id と request_id は引き継ぐ
        assert_eq!(child.trace_id(), parent.trace_id());
        assert_eq!(child.request_id(), parent.request_id());

        // span_id は新しく生成
        assert_ne!(child.span_id(), parent.span_id());

        // parent_span_id は親の span_id
        assert_eq!(child.parent_span_id(), Some(parent.span_id()));
    }

    #[test]
    fn test_with_tenant_id() {
        let ctx = RequestContext::new().with_tenant_id("tenant-123");
        assert_eq!(ctx.tenant_id(), Some("tenant-123"));
    }

    #[test]
    fn test_with_user_id() {
        let ctx = RequestContext::new().with_user_id("user-456");
        assert_eq!(ctx.user_id(), Some("user-456"));
    }

    #[test]
    fn test_to_traceparent() {
        let ctx = RequestContext::new();
        let traceparent = ctx.to_traceparent();

        assert!(traceparent.starts_with("00-"));
        assert!(traceparent.ends_with("-01"));
        assert!(traceparent.contains(ctx.trace_id()));
        assert!(traceparent.contains(ctx.span_id()));
    }

    #[test]
    fn test_from_traceparent() {
        let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx = RequestContext::from_traceparent(traceparent).unwrap();

        assert_eq!(ctx.trace_id(), "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.parent_span_id(), Some("b7ad6b7169203331"));
    }

    #[test]
    fn test_from_traceparent_invalid() {
        let ctx = RequestContext::from_traceparent("invalid");
        assert!(ctx.is_none());
    }

    #[test]
    fn test_serialize() {
        let ctx = RequestContext::new().with_tenant_id("tenant-1");
        let json = serde_json::to_string(&ctx).unwrap();

        assert!(json.contains("trace_id"));
        assert!(json.contains("request_id"));
        assert!(json.contains("tenant-1"));
    }

    #[test]
    fn test_header_names() {
        assert_eq!(HeaderNames::TRACEPARENT, "traceparent");
        assert_eq!(HeaderNames::X_REQUEST_ID, "x-request-id");
    }
}
