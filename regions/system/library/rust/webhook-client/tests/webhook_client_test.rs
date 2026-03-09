use k1s0_webhook_client::{
    generate_signature, verify_signature, WebhookConfig, WebhookError, WebhookPayload,
    IDEMPOTENCY_KEY_HEADER, SIGNATURE_HEADER,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_payload() -> WebhookPayload {
    WebhookPayload {
        event_type: "order.completed".to_string(),
        timestamp: "2026-03-10T12:00:00Z".to_string(),
        data: json!({"order_id": "ORD-001", "amount": 9999}),
    }
}

// ===========================================================================
// Signature generation & verification roundtrip
// ===========================================================================

#[test]
fn signature_roundtrip_succeeds() {
    let secret = "webhook-secret-key";
    let body = b"some payload body";
    let sig = generate_signature(secret, body);
    assert!(verify_signature(secret, body, &sig));
}

#[test]
fn signature_roundtrip_with_json_payload() {
    let secret = "my-secret";
    let payload = test_payload();
    let body = serde_json::to_vec(&payload).unwrap();
    let sig = generate_signature(secret, &body);
    assert!(verify_signature(secret, &body, &sig));
}

#[test]
fn signature_is_hex_encoded_64_chars() {
    let sig = generate_signature("secret", b"data");
    assert_eq!(sig.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn signature_deterministic_same_input() {
    let secret = "stable-secret";
    let body = b"same body";
    let sig1 = generate_signature(secret, body);
    let sig2 = generate_signature(secret, body);
    assert_eq!(sig1, sig2);
}

#[test]
fn signature_differs_for_different_secrets() {
    let body = b"payload";
    let sig1 = generate_signature("secret-a", body);
    let sig2 = generate_signature("secret-b", body);
    assert_ne!(sig1, sig2);
}

#[test]
fn signature_differs_for_different_bodies() {
    let secret = "same-secret";
    let sig1 = generate_signature(secret, b"body-a");
    let sig2 = generate_signature(secret, b"body-b");
    assert_ne!(sig1, sig2);
}

// ===========================================================================
// Invalid signature verification
// ===========================================================================

#[test]
fn verify_with_wrong_secret_fails() {
    let body = b"payload";
    let sig = generate_signature("correct-secret", body);
    assert!(!verify_signature("wrong-secret", body, &sig));
}

#[test]
fn verify_with_tampered_body_fails() {
    let secret = "my-secret";
    let sig = generate_signature(secret, b"original body");
    assert!(!verify_signature(secret, b"tampered body", &sig));
}

#[test]
fn verify_with_empty_signature_fails() {
    assert!(!verify_signature("secret", b"body", ""));
}

#[test]
fn verify_with_garbage_signature_fails() {
    assert!(!verify_signature("secret", b"body", "not-a-valid-signature"));
}

#[test]
fn verify_with_truncated_signature_fails() {
    let secret = "my-secret";
    let body = b"payload";
    let sig = generate_signature(secret, body);
    let truncated = &sig[..32]; // half of the 64-char hex
    assert!(!verify_signature(secret, body, truncated));
}

// ===========================================================================
// Signature with edge-case inputs
// ===========================================================================

#[test]
fn signature_with_empty_body() {
    let secret = "my-secret";
    let sig = generate_signature(secret, b"");
    assert!(verify_signature(secret, b"", &sig));
    assert_eq!(sig.len(), 64);
}

#[test]
fn signature_with_empty_secret() {
    let sig = generate_signature("", b"body");
    assert!(verify_signature("", b"body", &sig));
    assert_eq!(sig.len(), 64);
}

#[test]
fn signature_with_unicode_secret() {
    let secret = "secret-with-unicode-\u{1F600}";
    let body = b"payload";
    let sig = generate_signature(secret, body);
    assert!(verify_signature(secret, body, &sig));
}

#[test]
fn signature_with_large_body() {
    let secret = "my-secret";
    let body = vec![0xABu8; 10_000];
    let sig = generate_signature(secret, &body);
    assert!(verify_signature(secret, &body, &sig));
}

// ===========================================================================
// WebhookPayload construction & serde
// ===========================================================================

#[test]
fn payload_serialize_deserialize_roundtrip() {
    let payload = test_payload();
    let json_str = serde_json::to_string(&payload).unwrap();
    let deserialized: WebhookPayload = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.event_type, "order.completed");
    assert_eq!(deserialized.timestamp, "2026-03-10T12:00:00Z");
    assert_eq!(deserialized.data["order_id"], "ORD-001");
    assert_eq!(deserialized.data["amount"], 9999);
}

#[test]
fn payload_with_nested_data() {
    let payload = WebhookPayload {
        event_type: "user.updated".to_string(),
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        data: json!({
            "user": {
                "id": "U-123",
                "profile": {
                    "name": "Alice",
                    "roles": ["admin", "user"]
                }
            }
        }),
    };
    let json_str = serde_json::to_string(&payload).unwrap();
    let deserialized: WebhookPayload = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.data["user"]["profile"]["name"], "Alice");
    assert_eq!(deserialized.data["user"]["profile"]["roles"][0], "admin");
}

#[test]
fn payload_with_null_data() {
    let payload = WebhookPayload {
        event_type: "ping".to_string(),
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        data: json!(null),
    };
    let json_str = serde_json::to_string(&payload).unwrap();
    let deserialized: WebhookPayload = serde_json::from_str(&json_str).unwrap();
    assert!(deserialized.data.is_null());
}

#[test]
fn payload_with_empty_object_data() {
    let payload = WebhookPayload {
        event_type: "empty".to_string(),
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        data: json!({}),
    };
    let json_str = serde_json::to_string(&payload).unwrap();
    let deserialized: WebhookPayload = serde_json::from_str(&json_str).unwrap();
    assert!(deserialized.data.is_object());
    assert_eq!(deserialized.data.as_object().unwrap().len(), 0);
}

// ===========================================================================
// WebhookConfig
// ===========================================================================

#[test]
fn webhook_config_default_values() {
    let config = WebhookConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_ms, 100);
    assert_eq!(config.max_backoff_ms, 10000);
}

#[test]
fn webhook_config_custom_values() {
    let config = WebhookConfig {
        max_retries: 5,
        initial_backoff_ms: 200,
        max_backoff_ms: 30000,
    };
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.initial_backoff_ms, 200);
    assert_eq!(config.max_backoff_ms, 30000);
}

// ===========================================================================
// Constants
// ===========================================================================

#[test]
fn signature_header_constant() {
    assert_eq!(SIGNATURE_HEADER, "X-K1s0-Signature");
}

#[test]
fn idempotency_key_header_constant() {
    assert_eq!(IDEMPOTENCY_KEY_HEADER, "Idempotency-Key");
}

// ===========================================================================
// WebhookError display
// ===========================================================================

#[test]
fn error_display_request_failed() {
    let err = WebhookError::RequestFailed("connection reset".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("connection reset"));
}

#[test]
fn error_display_serialization_error() {
    // Create a real serde_json::Error
    let bad_json: Result<WebhookPayload, _> = serde_json::from_str("not json");
    let serde_err = bad_json.unwrap_err();
    let err = WebhookError::SerializationError(serde_err);
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn error_display_signature_error() {
    let err = WebhookError::SignatureError("invalid key length".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("invalid key length"));
}

#[test]
fn error_display_internal() {
    let err = WebhookError::Internal("unexpected state".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("unexpected state"));
}

#[test]
fn error_display_max_retries_exceeded() {
    let err = WebhookError::MaxRetriesExceeded {
        attempts: 4,
        last_status_code: 503,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("4"));
    assert!(msg.contains("503"));
}

// ===========================================================================
// Integration-style: sign payload and verify
// ===========================================================================

#[test]
fn sign_and_verify_full_payload() {
    let secret = "production-secret-key-2026";
    let payload = WebhookPayload {
        event_type: "invoice.paid".to_string(),
        timestamp: "2026-03-10T15:30:00Z".to_string(),
        data: json!({
            "invoice_id": "INV-12345",
            "amount_cents": 50000,
            "currency": "JPY",
            "customer": {
                "id": "CUST-789",
                "email": "billing@example.com"
            }
        }),
    };

    let body = serde_json::to_vec(&payload).unwrap();
    let signature = generate_signature(secret, &body);

    // Receiver side: verify the signature
    assert!(verify_signature(secret, &body, &signature));

    // Tamper detection: modifying one byte invalidates the signature
    let mut tampered = body.clone();
    if let Some(byte) = tampered.last_mut() {
        *byte ^= 0xFF;
    }
    assert!(!verify_signature(secret, &tampered, &signature));
}
