use std::collections::HashMap;

/// KafkaTracing provides helpers for injecting and extracting
/// OpenTelemetry trace context into/from Kafka message headers.
pub struct KafkaTracing;

impl KafkaTracing {
    /// Injects the current OpenTelemetry context into Kafka headers.
    pub fn inject_context(headers: &mut HashMap<String, Vec<u8>>) {
        use opentelemetry::global;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let context = tracing::Span::current().context();
        let propagator = global::get_text_map_propagator(|p| {
            let mut tmp = HashMap::<String, String>::new();
            p.inject_context(&context, &mut tmp);
            tmp
        });
        for (key, value) in propagator {
            headers.insert(key, value.into_bytes());
        }
    }

    /// Extracts an OpenTelemetry context from Kafka headers.
    pub fn extract_context(headers: &HashMap<String, Vec<u8>>) {
        use opentelemetry::global;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let string_headers: HashMap<String, String> = headers
            .iter()
            .filter_map(|(k, v)| String::from_utf8(v.clone()).ok().map(|s| (k.clone(), s)))
            .collect();

        let context = global::get_text_map_propagator(|p| p.extract(&string_headers));
        tracing::Span::current().set_parent(context);
    }
}
