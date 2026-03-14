//! Integration tests for k1s0-correlation.
//!
//! Tests the public API for CorrelationId, TraceId, CorrelationContext,
//! and CorrelationHeaders from the consumer perspective (external crate).

use k1s0_correlation::{CorrelationContext, CorrelationHeaders, CorrelationId, TraceId};

// ===========================================================================
// CorrelationId generation and parsing
// ===========================================================================

// 新しく生成した CorrelationId が空文字でないことを確認する。
#[test]
fn correlation_id_new_is_non_empty() {
    let id = CorrelationId::new();
    assert!(!id.as_str().is_empty());
}

// 複数回生成した CorrelationId が全て異なる値であることを確認する。
#[test]
fn correlation_id_new_generates_unique_ids() {
    let ids: Vec<CorrelationId> = (0..10).map(|_| CorrelationId::new()).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "IDs at positions {i} and {j} should differ");
        }
    }
}

// 文字列から生成した CorrelationId がその値を保持することを確認する。
#[test]
fn correlation_id_from_string_preserves_value() {
    let id = CorrelationId::from_string("req-abc-123");
    assert_eq!(id.as_str(), "req-abc-123");
    assert_eq!(format!("{}", id), "req-abc-123");
}

// Default 実装で生成した CorrelationId が空でないことを確認する。
#[test]
fn correlation_id_default_is_non_empty() {
    let id = CorrelationId::default();
    assert!(!id.as_str().is_empty());
}

// 同じ文字列から生成した CorrelationId 同士が等しいことを確認する。
#[test]
fn correlation_id_equality() {
    let a = CorrelationId::from_string("same-id");
    let b = CorrelationId::from_string("same-id");
    assert_eq!(a, b);
}

// 異なる文字列から生成した CorrelationId が等しくないことを確認する。
#[test]
fn correlation_id_inequality() {
    let a = CorrelationId::from_string("id-a");
    let b = CorrelationId::from_string("id-b");
    assert_ne!(a, b);
}

// CorrelationId をクローンした場合に元の値と同じになることを確認する。
#[test]
fn correlation_id_clone() {
    let original = CorrelationId::from_string("clone-test");
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(original.as_str(), cloned.as_str());
}

// CorrelationId のシリアライズ・デシリアライズが元の値を保持することを確認する。
#[test]
fn correlation_id_serialization_roundtrip() {
    let id = CorrelationId::from_string("serde-test-001");
    let json = serde_json::to_string(&id).unwrap();
    let restored: CorrelationId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, restored);
}

// CorrelationId が HashMap のキーとして使えることを確認する。
#[test]
fn correlation_id_usable_as_hash_key() {
    use std::collections::HashMap;
    let id = CorrelationId::from_string("hash-key");
    let mut map = HashMap::new();
    map.insert(id.clone(), "value");
    assert_eq!(map.get(&id), Some(&"value"));
}

// ===========================================================================
// TraceId generation and parsing
// ===========================================================================

// 新しく生成した TraceId が 32 文字の16進数文字列であることを確認する。
#[test]
fn trace_id_new_is_32_char_hex() {
    let id = TraceId::new();
    assert_eq!(id.as_str().len(), 32);
    assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
}

// 複数回生成した TraceId が異なる値であることを確認する。
#[test]
fn trace_id_new_generates_unique_ids() {
    let id1 = TraceId::new();
    let id2 = TraceId::new();
    assert_ne!(id1, id2);
}

// 有効な32文字16進数文字列から TraceId を生成できることを確認する。
#[test]
fn trace_id_from_valid_32_char_hex() {
    let raw = "4bf92f3577b34da6a3ce929d0e0e4736";
    let id = TraceId::from_string(raw);
    assert!(id.is_some());
    assert_eq!(id.unwrap().as_str(), raw);
}

// 大文字の16進数文字列からも TraceId を生成できることを確認する。
#[test]
fn trace_id_from_uppercase_hex_accepted() {
    let upper = "4BF92F3577B34DA6A3CE929D0E0E4736";
    assert!(TraceId::from_string(upper).is_some());
}

// 短すぎる文字列から TraceId を生成しようとすると None が返されることを確認する。
#[test]
fn trace_id_from_short_string_rejected() {
    assert!(TraceId::from_string("abc123").is_none());
}

// 31 文字の文字列から TraceId を生成しようとすると None が返されることを確認する。
#[test]
fn trace_id_from_31_char_rejected() {
    let short = "4bf92f3577b34da6a3ce929d0e0e473";
    assert!(TraceId::from_string(short).is_none());
}

// 33 文字の文字列から TraceId を生成しようとすると None が返されることを確認する。
#[test]
fn trace_id_from_33_char_rejected() {
    let long = "4bf92f3577b34da6a3ce929d0e0e47360";
    assert!(TraceId::from_string(long).is_none());
}

// 16進数以外の文字を含む文字列から TraceId を生成しようとすると None が返されることを確認する。
#[test]
fn trace_id_from_non_hex_rejected() {
    let bad = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
    assert!(TraceId::from_string(bad).is_none());
}

// Default 実装で生成した TraceId が有効な32文字16進数であることを確認する。
#[test]
fn trace_id_default_is_valid() {
    let id = TraceId::default();
    assert_eq!(id.as_str().len(), 32);
    assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
}

// TraceId の Display 実装が元の文字列を返すことを確認する。
#[test]
fn trace_id_display() {
    let raw = "4bf92f3577b34da6a3ce929d0e0e4736";
    let id = TraceId::from_string(raw).unwrap();
    assert_eq!(format!("{}", id), raw);
}

// TraceId のシリアライズ・デシリアライズが元の値を保持することを確認する。
#[test]
fn trace_id_serialization_roundtrip() {
    let id = TraceId::from_string("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
    let json = serde_json::to_string(&id).unwrap();
    let restored: TraceId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, restored);
}

// ===========================================================================
// CorrelationContext
// ===========================================================================

// 新しく生成したコンテキストが相関 ID を持ちトレース ID が None であることを確認する。
#[test]
fn context_new_has_correlation_id_and_no_trace() {
    let ctx = CorrelationContext::new();
    assert!(!ctx.correlation_id.as_str().is_empty());
    assert!(ctx.trace_id.is_none());
}

// Default 実装が new と同じ挙動を持つことを確認する。
#[test]
fn context_default_same_as_new() {
    let ctx = CorrelationContext::default();
    assert!(!ctx.correlation_id.as_str().is_empty());
    assert!(ctx.trace_id.is_none());
}

// with_trace_id でトレース ID をコンテキストに設定できることを確認する。
#[test]
fn context_with_trace_id() {
    let trace = TraceId::new();
    let expected = trace.as_str().to_string();
    let ctx = CorrelationContext::new().with_trace_id(trace);
    assert!(ctx.trace_id.is_some());
    assert_eq!(ctx.trace_id.unwrap().as_str(), expected);
}

// 既存の相関 ID からコンテキストを生成できることを確認する。
#[test]
fn context_from_correlation_id() {
    let cid = CorrelationId::from_string("custom-corr-id");
    let ctx = CorrelationContext::from_correlation_id(cid);
    assert_eq!(ctx.correlation_id.as_str(), "custom-corr-id");
    assert!(ctx.trace_id.is_none());
}

// new で生成した複数のコンテキストが異なる相関 ID を持つことを確認する。
#[test]
fn context_new_generates_unique_correlation_ids() {
    let ctx1 = CorrelationContext::new();
    let ctx2 = CorrelationContext::new();
    assert_ne!(ctx1.correlation_id.as_str(), ctx2.correlation_id.as_str());
}

// ===========================================================================
// CorrelationHeaders: to_headers / from_headers
// ===========================================================================

// ヘッダー名定数が正しい値を持つことを確認する。
#[test]
fn header_constants() {
    assert_eq!(CorrelationHeaders::CORRELATION_ID, "x-correlation-id");
    assert_eq!(CorrelationHeaders::TRACE_ID, "x-trace-id");
}

// トレース ID なしのコンテキストからヘッダーを生成すると 1 件のみ返されることを確認する。
#[test]
fn to_headers_without_trace_returns_one_header() {
    let ctx = CorrelationContext::from_correlation_id(CorrelationId::from_string("corr-001"));
    let headers = CorrelationHeaders::to_headers(&ctx);
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].0, "x-correlation-id");
    assert_eq!(headers[0].1, "corr-001");
}

// トレース ID ありのコンテキストからヘッダーを生成すると 2 件返されることを確認する。
#[test]
fn to_headers_with_trace_returns_two_headers() {
    let trace = TraceId::from_string("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
    let ctx = CorrelationContext::from_correlation_id(CorrelationId::from_string("corr-002"))
        .with_trace_id(trace);
    let headers = CorrelationHeaders::to_headers(&ctx);
    assert_eq!(headers.len(), 2);

    let corr = headers.iter().find(|(k, _)| k == "x-correlation-id");
    assert!(corr.is_some());
    assert_eq!(corr.unwrap().1, "corr-002");

    let trace_h = headers.iter().find(|(k, _)| k == "x-trace-id");
    assert!(trace_h.is_some());
    assert_eq!(trace_h.unwrap().1, "4bf92f3577b34da6a3ce929d0e0e4736");
}

// ヘッダーから相関 ID とトレース ID の両方が正しく取り出されることを確認する。
#[test]
fn from_headers_extracts_both_ids() {
    let headers = vec![
        ("x-correlation-id".to_string(), "from-header".to_string()),
        (
            "x-trace-id".to_string(),
            "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
        ),
    ];
    let ctx = CorrelationHeaders::from_headers(&headers);
    assert_eq!(ctx.correlation_id.as_str(), "from-header");
    assert!(ctx.trace_id.is_some());
    assert_eq!(
        ctx.trace_id.unwrap().as_str(),
        "4bf92f3577b34da6a3ce929d0e0e4736"
    );
}

// 相関 ID ヘッダーがない場合にデフォルトの相関 ID が自動生成されることを確認する。
#[test]
fn from_headers_missing_generates_default_correlation_id() {
    let headers: Vec<(String, String)> = vec![];
    let ctx = CorrelationHeaders::from_headers(&headers);
    assert!(!ctx.correlation_id.as_str().is_empty());
    assert!(ctx.trace_id.is_none());
}

// 不正なトレース ID ヘッダーが None として扱われることを確認する。
#[test]
fn from_headers_invalid_trace_id_becomes_none() {
    let headers = vec![
        ("x-correlation-id".to_string(), "corr-abc".to_string()),
        ("x-trace-id".to_string(), "not-valid-hex".to_string()),
    ];
    let ctx = CorrelationHeaders::from_headers(&headers);
    assert_eq!(ctx.correlation_id.as_str(), "corr-abc");
    assert!(ctx.trace_id.is_none());
}

// ヘッダーキーの大文字小文字を区別せずに相関 ID が取得できることを確認する。
#[test]
fn from_headers_case_insensitive_key_matching() {
    let headers = vec![("X-Correlation-Id".to_string(), "upper-case".to_string())];
    let ctx = CorrelationHeaders::from_headers(&headers);
    assert_eq!(ctx.correlation_id.as_str(), "upper-case");
}

// ===========================================================================
// Roundtrip: to_headers -> from_headers
// ===========================================================================

// to_headers → from_headers のラウンドトリップでトレース ID 付きコンテキストが復元されることを確認する。
#[test]
fn roundtrip_headers_with_trace() {
    let original_ctx =
        CorrelationContext::from_correlation_id(CorrelationId::from_string("roundtrip-id"))
            .with_trace_id(TraceId::from_string("4bf92f3577b34da6a3ce929d0e0e4736").unwrap());

    let headers = CorrelationHeaders::to_headers(&original_ctx);
    let restored_ctx = CorrelationHeaders::from_headers(&headers);

    assert_eq!(
        restored_ctx.correlation_id.as_str(),
        original_ctx.correlation_id.as_str()
    );
    assert_eq!(
        restored_ctx.trace_id.as_ref().unwrap().as_str(),
        original_ctx.trace_id.as_ref().unwrap().as_str()
    );
}

// to_headers → from_headers のラウンドトリップでトレース ID なしのコンテキストが復元されることを確認する。
#[test]
fn roundtrip_headers_without_trace() {
    let original_ctx =
        CorrelationContext::from_correlation_id(CorrelationId::from_string("no-trace-rt"));

    let headers = CorrelationHeaders::to_headers(&original_ctx);
    let restored_ctx = CorrelationHeaders::from_headers(&headers);

    assert_eq!(
        restored_ctx.correlation_id.as_str(),
        original_ctx.correlation_id.as_str()
    );
    assert!(restored_ctx.trace_id.is_none());
}
