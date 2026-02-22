use crate::id::{CorrelationId, TraceId};

/// CorrelationContext はリクエスト伝播コンテキストを表す。
/// HTTP ヘッダー・gRPC メタデータ・Kafka ヘッダー経由で伝播する。
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// 業務相関 ID
    pub correlation_id: CorrelationId,
    /// OpenTelemetry トレース ID
    pub trace_id: Option<TraceId>,
}

impl CorrelationContext {
    /// 新しいコンテキストを生成する（相関 ID は自動生成）。
    pub fn new() -> Self {
        Self {
            correlation_id: CorrelationId::new(),
            trace_id: None,
        }
    }

    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    /// 既存の相関 ID からコンテキストを生成する。
    pub fn from_correlation_id(correlation_id: CorrelationId) -> Self {
        Self {
            correlation_id,
            trace_id: None,
        }
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// CorrelationHeaders は HTTP/gRPC ヘッダーでの伝播用のヘッダー名定数。
pub struct CorrelationHeaders;

impl CorrelationHeaders {
    /// 相関 ID のヘッダー名
    pub const CORRELATION_ID: &'static str = "x-correlation-id";
    /// トレース ID のヘッダー名（OpenTelemetry の traceparent に対応）
    pub const TRACE_ID: &'static str = "x-trace-id";

    /// CorrelationContext からヘッダーマップを生成する。
    pub fn to_headers(ctx: &CorrelationContext) -> Vec<(String, String)> {
        let mut headers = vec![(
            Self::CORRELATION_ID.to_string(),
            ctx.correlation_id.to_string(),
        )];
        if let Some(ref trace_id) = ctx.trace_id {
            headers.push((Self::TRACE_ID.to_string(), trace_id.to_string()));
        }
        headers
    }

    /// HTTP ヘッダーから CorrelationContext を復元する。
    pub fn from_headers(headers: &[(String, String)]) -> CorrelationContext {
        let mut correlation_id = None;
        let mut trace_id = None;

        for (key, value) in headers {
            match key.to_lowercase().as_str() {
                Self::CORRELATION_ID => {
                    correlation_id = Some(CorrelationId::from_string(value));
                }
                Self::TRACE_ID => {
                    trace_id = TraceId::from_string(value);
                }
                _ => {}
            }
        }

        CorrelationContext {
            correlation_id: correlation_id.unwrap_or_default(),
            trace_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::CorrelationId;

    #[test]
    fn test_context_new() {
        let ctx = CorrelationContext::new();
        assert!(!ctx.correlation_id.as_str().is_empty());
        assert!(ctx.trace_id.is_none());
    }

    #[test]
    fn test_context_with_trace_id() {
        let trace = TraceId::new();
        let ctx = CorrelationContext::new().with_trace_id(trace.clone());
        assert_eq!(ctx.trace_id.as_ref().unwrap().as_str(), trace.as_str());
    }

    #[test]
    fn test_to_headers_without_trace() {
        let ctx = CorrelationContext::from_correlation_id(
            CorrelationId::from_string("corr-001"),
        );
        let headers = CorrelationHeaders::to_headers(&ctx);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "x-correlation-id");
        assert_eq!(headers[0].1, "corr-001");
    }

    #[test]
    fn test_to_headers_with_trace() {
        let trace = TraceId::from_string("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let ctx = CorrelationContext::from_correlation_id(
            CorrelationId::from_string("corr-001"),
        )
        .with_trace_id(trace);
        let headers = CorrelationHeaders::to_headers(&ctx);
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_from_headers() {
        let headers = vec![
            ("x-correlation-id".to_string(), "corr-123".to_string()),
            (
                "x-trace-id".to_string(),
                "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            ),
        ];
        let ctx = CorrelationHeaders::from_headers(&headers);
        assert_eq!(ctx.correlation_id.as_str(), "corr-123");
        assert!(ctx.trace_id.is_some());
    }

    #[test]
    fn test_from_headers_missing_correlation() {
        let headers = vec![];
        let ctx = CorrelationHeaders::from_headers(&headers);
        assert!(!ctx.correlation_id.as_str().is_empty()); // 自動生成
    }

    #[test]
    fn test_context_default() {
        let ctx = CorrelationContext::default();
        assert!(!ctx.correlation_id.as_str().is_empty());
        assert!(ctx.trace_id.is_none());
    }

    #[test]
    fn test_context_from_correlation_id() {
        let cid = CorrelationId::from_string("my-corr-id");
        let ctx = CorrelationContext::from_correlation_id(cid.clone());
        assert_eq!(ctx.correlation_id.as_str(), "my-corr-id");
        assert!(ctx.trace_id.is_none());
    }

    #[test]
    fn test_to_headers_with_trace_content() {
        let trace = TraceId::from_string("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        let ctx = CorrelationContext::from_correlation_id(
            CorrelationId::from_string("corr-xyz"),
        )
        .with_trace_id(trace);
        let headers = CorrelationHeaders::to_headers(&ctx);
        assert_eq!(headers.len(), 2);
        let corr_header = headers.iter().find(|(k, _)| k == "x-correlation-id");
        assert!(corr_header.is_some());
        assert_eq!(corr_header.unwrap().1, "corr-xyz");
        let trace_header = headers.iter().find(|(k, _)| k == "x-trace-id");
        assert!(trace_header.is_some());
        assert_eq!(
            trace_header.unwrap().1,
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
    }

    #[test]
    fn test_from_headers_case_insensitive() {
        // ヘッダーキーは小文字に正規化されて比較される
        let headers = vec![
            ("X-Correlation-Id".to_string(), "corr-upper".to_string()),
        ];
        let ctx = CorrelationHeaders::from_headers(&headers);
        assert_eq!(ctx.correlation_id.as_str(), "corr-upper");
    }

    #[test]
    fn test_from_headers_invalid_trace_id_ignored() {
        // 不正なトレース ID は None になる
        let headers = vec![
            ("x-correlation-id".to_string(), "corr-001".to_string()),
            ("x-trace-id".to_string(), "invalid-trace".to_string()),
        ];
        let ctx = CorrelationHeaders::from_headers(&headers);
        assert_eq!(ctx.correlation_id.as_str(), "corr-001");
        assert!(ctx.trace_id.is_none());
    }

    #[test]
    fn test_header_name_constants() {
        assert_eq!(CorrelationHeaders::CORRELATION_ID, "x-correlation-id");
        assert_eq!(CorrelationHeaders::TRACE_ID, "x-trace-id");
    }

    #[test]
    fn test_context_unique_per_new() {
        let ctx1 = CorrelationContext::new();
        let ctx2 = CorrelationContext::new();
        assert_ne!(
            ctx1.correlation_id.as_str(),
            ctx2.correlation_id.as_str()
        );
    }
}
