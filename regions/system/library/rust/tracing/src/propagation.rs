use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    pub trace_id: String,
    pub parent_id: String,
    pub flags: u8,
}

impl TraceContext {
    pub fn new(trace_id: &str, parent_id: &str, flags: u8) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            parent_id: parent_id.to_string(),
            flags,
        }
    }

    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.parent_id, self.flags
        )
    }

    pub fn from_traceparent(s: &str) -> Option<TraceContext> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 4 {
            return None;
        }
        if parts[0] != "00" {
            return None;
        }
        let trace_id = parts[1];
        let parent_id = parts[2];
        let flags = u8::from_str_radix(parts[3], 16).ok()?;

        if trace_id.len() != 32 || parent_id.len() != 16 {
            return None;
        }
        if !trace_id.chars().all(|c| c.is_ascii_hexdigit())
            || !parent_id.chars().all(|c| c.is_ascii_hexdigit())
        {
            return None;
        }

        Some(TraceContext {
            trace_id: trace_id.to_string(),
            parent_id: parent_id.to_string(),
            flags,
        })
    }
}

pub fn inject_context(ctx: &TraceContext, headers: &mut HashMap<String, String>) {
    headers.insert("traceparent".to_string(), ctx.to_traceparent());
}

pub fn extract_context(headers: &HashMap<String, String>) -> Option<TraceContext> {
    headers
        .get("traceparent")
        .and_then(|v| TraceContext::from_traceparent(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_traceparent() {
        let ctx = TraceContext::new(
            "0af7651916cd43dd8448eb211c80319c",
            "b7ad6b7169203331",
            1,
        );
        assert_eq!(
            ctx.to_traceparent(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
        );
    }

    #[test]
    fn test_from_traceparent() {
        let s = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx = TraceContext::from_traceparent(s).unwrap();
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.parent_id, "b7ad6b7169203331");
        assert_eq!(ctx.flags, 1);
    }

    #[test]
    fn test_roundtrip() {
        let original = TraceContext::new(
            "abcdef1234567890abcdef1234567890",
            "1234567890abcdef",
            0,
        );
        let s = original.to_traceparent();
        let parsed = TraceContext::from_traceparent(&s).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_invalid_version() {
        let s = "01-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        assert!(TraceContext::from_traceparent(s).is_none());
    }

    #[test]
    fn test_invalid_format() {
        assert!(TraceContext::from_traceparent("invalid").is_none());
        assert!(TraceContext::from_traceparent("00-short-id-01").is_none());
        assert!(TraceContext::from_traceparent("").is_none());
    }

    #[test]
    fn test_invalid_hex() {
        let s = "00-zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz-b7ad6b7169203331-01";
        assert!(TraceContext::from_traceparent(s).is_none());
    }

    #[test]
    fn test_inject_context() {
        let ctx = TraceContext::new(
            "0af7651916cd43dd8448eb211c80319c",
            "b7ad6b7169203331",
            1,
        );
        let mut headers = HashMap::new();
        inject_context(&ctx, &mut headers);
        assert!(headers.contains_key("traceparent"));
        assert_eq!(
            headers["traceparent"],
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
        );
    }

    #[test]
    fn test_extract_context() {
        let mut headers = HashMap::new();
        headers.insert(
            "traceparent".to_string(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
        );
        let ctx = extract_context(&headers).unwrap();
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_extract_context_missing() {
        let headers = HashMap::new();
        assert!(extract_context(&headers).is_none());
    }

    #[test]
    fn test_flags_zero() {
        let ctx = TraceContext::new(
            "0af7651916cd43dd8448eb211c80319c",
            "b7ad6b7169203331",
            0,
        );
        assert!(ctx.to_traceparent().ends_with("-00"));
    }
}
