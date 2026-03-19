//! External integration tests for k1s0-tracing.
//!
//! Inline tests already cover basic span create/end, event add, baggage set/get,
//! and propagation roundtrip. These external tests exercise the PUBLIC API as a
//! consumer would, focusing on:
//! - Cross-module workflows (span + baggage + propagation together)
//! - Multi-span event accumulation
//! - Baggage overwrite semantics
//! - TraceContext edge cases not covered inline
//! - inject_context / extract_context (lib.rs top-level API) integration
#![allow(clippy::unwrap_used)]

use k1s0_tracing::*;
use std::collections::HashMap;

// ===========================================================================
// Span lifecycle — external consumer perspective
// ===========================================================================

// スパンに複数のイベントを追加し正しく記録されることを確認する。
#[test]
fn span_lifecycle_with_multiple_events() {
    let mut span = start_span("multi-event-op");
    let mut attrs1 = HashMap::new();
    attrs1.insert("step".to_string(), "1".to_string());
    add_event(&mut span, "step-start", attrs1);

    let mut attrs2 = HashMap::new();
    attrs2.insert("step".to_string(), "2".to_string());
    attrs2.insert("detail".to_string(), "processing".to_string());
    add_event(&mut span, "step-processing", attrs2);

    add_event(&mut span, "step-done", HashMap::new());

    assert_eq!(span.events.len(), 3);
    assert_eq!(span.events[0].name, "step-start");
    assert_eq!(
        span.events[1].attributes.get("detail"),
        Some(&"processing".to_string())
    );
    assert_eq!(span.events[2].attributes.len(), 0);

    end_span(span); // should not panic
}

// スパン生成時に指定した名前が保持されることを確認する。
#[test]
fn span_name_preserved() {
    let span = start_span("db.query.find_user");
    assert_eq!(span.name, "db.query.find_user");
    end_span(span);
}

// 新規スパンの trace_id、span_id、attributes、events がすべて空であることを確認する。
#[test]
fn span_handle_fields_default_empty() {
    let span = start_span("op");
    assert!(span.trace_id.is_empty());
    assert!(span.span_id.is_empty());
    assert!(span.attributes.is_empty());
    assert!(span.events.is_empty());
    end_span(span);
}

// ===========================================================================
// Baggage — overwrite and multi-entry
// ===========================================================================

// 同じキーを再度セットした場合に値が上書きされ件数が増えないことを確認する。
#[test]
fn baggage_overwrite_key() {
    let mut b = Baggage::new();
    b.set("tenant", "acme");
    b.set("tenant", "globex");
    assert_eq!(b.get("tenant"), Some("globex"));
    assert_eq!(b.len(), 1);
}

// 複数エントリを持つ Baggage のヘッダーへのシリアライズと復元が正しいことを確認する。
#[test]
fn baggage_multiple_entries_header_roundtrip() {
    let mut b = Baggage::new();
    b.set("userId", "alice");
    b.set("requestId", "req-123");
    b.set("tenant", "acme");

    let header = b.to_header();
    let parsed = Baggage::from_header(&header);

    assert_eq!(parsed.get("userId"), Some("alice"));
    assert_eq!(parsed.get("requestId"), Some("req-123"));
    assert_eq!(parsed.get("tenant"), Some("acme"));
    assert_eq!(parsed.len(), 3);
}

// "=" を含まないエントリはスキップされ有効なエントリのみが解析されることを確認する。
#[test]
fn baggage_from_header_ignores_entries_without_equals() {
    let b = Baggage::from_header("valid=ok,no_equals_here");
    assert_eq!(b.get("valid"), Some("ok"));
    // "no_equals_here" has no '=' so it's skipped
    assert_eq!(b.len(), 1);
}

// ===========================================================================
// TraceContext — boundary cases
// ===========================================================================

// flags が 0xff のときに traceparent の末尾が "-ff" になることを確認する。
#[test]
fn trace_context_flags_all_bits() {
    let ctx = TraceContext::new("0af7651916cd43dd8448eb211c80319c", "b7ad6b7169203331", 0xff);
    let tp = ctx.to_traceparent();
    assert!(tp.ends_with("-ff"));

    let parsed = TraceContext::from_traceparent(&tp).unwrap();
    assert_eq!(parsed.flags, 0xff);
}

// パーツ数が 4 でない traceparent 文字列の解析が None を返すことを確認する。
#[test]
fn trace_context_from_traceparent_wrong_part_count() {
    assert!(TraceContext::from_traceparent("00-abc-def").is_none());
    assert!(TraceContext::from_traceparent("00-a-b-c-d-e").is_none());
}

// trace_id や parent_id が規定の長さでない場合の解析が None を返すことを確認する。
#[test]
fn trace_context_from_traceparent_wrong_lengths() {
    // trace_id too short (31 hex chars instead of 32)
    assert!(TraceContext::from_traceparent(
        "00-0af7651916cd43dd8448eb211c8031-b7ad6b7169203331-01"
    )
    .is_none());
    // parent_id too short (15 hex chars instead of 16)
    assert!(TraceContext::from_traceparent(
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b716920333-01"
    )
    .is_none());
}

// 同じフィールド値を持つ TraceContext が等しく、異なる flags を持つと不等になることを確認する。
#[test]
fn trace_context_equality() {
    let a = TraceContext::new("0af7651916cd43dd8448eb211c80319c", "b7ad6b7169203331", 1);
    let b = TraceContext::new("0af7651916cd43dd8448eb211c80319c", "b7ad6b7169203331", 1);
    assert_eq!(a, b);

    let c = TraceContext::new("0af7651916cd43dd8448eb211c80319c", "b7ad6b7169203331", 0);
    assert_ne!(a, c);
}

// ===========================================================================
// inject_context / extract_context — top-level API
// ===========================================================================

// コンテキストと Baggage をヘッダーに注入し正しく復元できることを確認する。
#[test]
fn inject_extract_roundtrip_with_baggage() {
    let ctx = TraceContext::new("abcdef1234567890abcdef1234567890", "1234567890abcdef", 1);
    let mut baggage = Baggage::new();
    baggage.set("userId", "bob");
    baggage.set("env", "prod");

    let mut headers = HashMap::new();
    inject_context(&mut headers, &ctx, Some(&baggage));

    assert!(headers.contains_key("traceparent"));
    assert!(headers.contains_key("baggage"));

    let (extracted_ctx, extracted_baggage) = extract_context(&headers);
    let extracted_ctx = extracted_ctx.unwrap();
    assert_eq!(extracted_ctx.trace_id, "abcdef1234567890abcdef1234567890");
    assert_eq!(extracted_ctx.parent_id, "1234567890abcdef");
    assert_eq!(extracted_ctx.flags, 1);
    assert_eq!(extracted_baggage.get("userId"), Some("bob"));
    assert_eq!(extracted_baggage.get("env"), Some("prod"));
}

// Baggage なしで注入した場合に baggage ヘッダーが追加されないことを確認する。
#[test]
fn inject_without_baggage_no_baggage_header() {
    let ctx = TraceContext::new("abcdef1234567890abcdef1234567890", "1234567890abcdef", 0);
    let mut headers = HashMap::new();
    inject_context(&mut headers, &ctx, None);

    assert!(headers.contains_key("traceparent"));
    assert!(!headers.contains_key("baggage"));
}

// 空の Baggage を渡した場合に baggage ヘッダーが追加されないことを確認する。
#[test]
fn inject_with_empty_baggage_no_baggage_header() {
    let ctx = TraceContext::new("abcdef1234567890abcdef1234567890", "1234567890abcdef", 0);
    let baggage = Baggage::new();
    let mut headers = HashMap::new();
    inject_context(&mut headers, &ctx, Some(&baggage));

    assert!(headers.contains_key("traceparent"));
    // Empty baggage should not produce a header
    assert!(!headers.contains_key("baggage"));
}

// 空のヘッダーマップからコンテキスト抽出すると ctx が None で Baggage が空であることを確認する。
#[test]
fn extract_from_empty_headers() {
    let headers = HashMap::new();
    let (ctx, baggage) = extract_context(&headers);
    assert!(ctx.is_none());
    assert!(baggage.is_empty());
}

// baggage ヘッダーのみ存在する場合に ctx が None で Baggage が解析されることを確認する。
#[test]
fn extract_with_only_baggage_header() {
    let mut headers = HashMap::new();
    headers.insert("baggage".to_string(), "key=val".to_string());
    let (ctx, baggage) = extract_context(&headers);
    assert!(ctx.is_none());
    assert_eq!(baggage.get("key"), Some("val"));
}

// 無効な traceparent ヘッダーからの抽出で ctx が None になることを確認する。
#[test]
fn extract_with_invalid_traceparent() {
    let mut headers = HashMap::new();
    headers.insert(
        "traceparent".to_string(),
        "not-a-valid-traceparent".to_string(),
    );
    let (ctx, baggage) = extract_context(&headers);
    assert!(ctx.is_none());
    assert!(baggage.is_empty());
}

// ===========================================================================
// Cross-module integration: span + inject/extract
// ===========================================================================

// サービス間でコンテキストを伝播させる分散トレースのシナリオが正しく動作することを確認する。
#[test]
fn simulate_distributed_trace_propagation() {
    // Service A: create span and propagate context
    let span_a = start_span("service-a.handle_request");
    let ctx = TraceContext::new("abcdef1234567890abcdef1234567890", "1234567890abcdef", 1);
    let mut baggage = Baggage::new();
    baggage.set("requestId", "req-42");

    let mut outgoing_headers = HashMap::new();
    inject_context(&mut outgoing_headers, &ctx, Some(&baggage));
    end_span(span_a);

    // Service B: extract context and create child span
    let (extracted_ctx, extracted_baggage) = extract_context(&outgoing_headers);
    assert!(extracted_ctx.is_some());
    assert_eq!(extracted_baggage.get("requestId"), Some("req-42"));

    let mut span_b = start_span("service-b.process");
    add_event(&mut span_b, "received", HashMap::new());
    end_span(span_b);
}
