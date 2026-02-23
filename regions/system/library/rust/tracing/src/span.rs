use std::collections::HashMap;

pub struct SpanEvent {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

pub struct SpanHandle {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

impl SpanHandle {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            trace_id: String::new(),
            span_id: String::new(),
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }
}

pub fn add_event(handle: &mut SpanHandle, name: &str, attributes: HashMap<String, String>) {
    handle.events.push(SpanEvent {
        name: name.to_string(),
        attributes,
    });
}

pub fn start_span(name: &str) -> SpanHandle {
    SpanHandle::new(name)
}

pub fn end_span(_handle: SpanHandle) {
    // 本番では span を閉じる
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_and_end_span() {
        let span = start_span("test-operation");
        assert_eq!(span.name, "test-operation");
        assert!(span.events.is_empty());
        end_span(span);
    }

    #[test]
    fn test_add_event() {
        let mut span = start_span("my-span");
        let mut attrs = HashMap::new();
        attrs.insert("key".to_string(), "value".to_string());
        add_event(&mut span, "my-event", attrs);

        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "my-event");
        assert_eq!(
            span.events[0].attributes.get("key"),
            Some(&"value".to_string())
        );
        end_span(span);
    }

    #[test]
    fn test_add_event_empty_attrs() {
        let mut span = start_span("test-span");
        add_event(&mut span, "empty-event", HashMap::new());
        assert_eq!(span.events.len(), 1);
        assert!(span.events[0].attributes.is_empty());
        end_span(span);
    }

    #[test]
    fn test_span_handle_new() {
        let span = SpanHandle::new("op");
        assert_eq!(span.name, "op");
        assert!(span.trace_id.is_empty());
        assert!(span.span_id.is_empty());
        assert!(span.attributes.is_empty());
        assert!(span.events.is_empty());
    }
}
