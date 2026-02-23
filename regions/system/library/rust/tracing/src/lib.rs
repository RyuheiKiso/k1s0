pub mod baggage;
pub mod propagation;
pub mod span;

pub use baggage::Baggage;
pub use propagation::TraceContext;
pub use span::{add_event, end_span, start_span, SpanHandle};

use std::collections::HashMap;

pub fn inject_context(
    headers: &mut HashMap<String, String>,
    ctx: &TraceContext,
    baggage: Option<&Baggage>,
) {
    headers.insert("traceparent".to_string(), ctx.to_traceparent());
    if let Some(b) = baggage {
        let h = b.to_header();
        if !h.is_empty() {
            headers.insert("baggage".to_string(), h);
        }
    }
}

pub fn extract_context(headers: &HashMap<String, String>) -> (Option<TraceContext>, Baggage) {
    let ctx = headers
        .get("traceparent")
        .and_then(|v| TraceContext::from_traceparent(v));
    let baggage = headers
        .get("baggage")
        .map(|v| Baggage::from_header(v))
        .unwrap_or_default();
    (ctx, baggage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_extract_roundtrip() {
        let ctx = TraceContext::new(
            "0af7651916cd43dd8448eb211c80319c",
            "b7ad6b7169203331",
            1,
        );
        let mut baggage = Baggage::new();
        baggage.set("userId", "alice");

        let mut headers = HashMap::new();
        inject_context(&mut headers, &ctx, Some(&baggage));

        assert!(headers.contains_key("traceparent"));
        assert!(headers.contains_key("baggage"));

        let (extracted_ctx, extracted_baggage) = extract_context(&headers);
        assert!(extracted_ctx.is_some());
        let extracted_ctx = extracted_ctx.unwrap();
        assert_eq!(extracted_ctx.trace_id, ctx.trace_id);
        assert_eq!(extracted_ctx.parent_id, ctx.parent_id);
        assert_eq!(extracted_ctx.flags, ctx.flags);
        assert_eq!(extracted_baggage.get("userId"), Some("alice"));
    }

    #[test]
    fn test_inject_without_baggage() {
        let ctx = TraceContext::new(
            "0af7651916cd43dd8448eb211c80319c",
            "b7ad6b7169203331",
            0,
        );
        let mut headers = HashMap::new();
        inject_context(&mut headers, &ctx, None);

        assert!(headers.contains_key("traceparent"));
        assert!(!headers.contains_key("baggage"));
    }

    #[test]
    fn test_extract_empty_headers() {
        let headers = HashMap::new();
        let (ctx, baggage) = extract_context(&headers);
        assert!(ctx.is_none());
        assert!(baggage.is_empty());
    }
}
