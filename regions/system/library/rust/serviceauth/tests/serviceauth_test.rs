#![allow(clippy::unwrap_used)]
// serviceauth の外部結合テスト。
// SpiffeId のパース、ServiceToken の動作、設定バリデーションを検証する。

use k1s0_serviceauth::{ServiceAuthConfig, ServiceAuthError, ServiceToken, SpiffeId};

// --- SpiffeId パーステスト ---

// 正しい SPIFFE URI を解析して各フィールドが正しく取得できることを確認する。
#[test]
fn test_spiffe_id_parse_valid() {
    let spiffe = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
    assert_eq!(spiffe.trust_domain, "k1s0.internal");
    assert_eq!(spiffe.namespace, "system");
    assert_eq!(spiffe.service_account, "auth-service");
}

// business ネームスペースの SPIFFE URI が正しく解析されることを確認する。
#[test]
fn test_spiffe_id_parse_business() {
    let spiffe = SpiffeId::parse("spiffe://k1s0.internal/ns/business/sa/task-server").unwrap();
    assert_eq!(spiffe.namespace, "business");
    assert_eq!(spiffe.service_account, "task-server");
}

// service ネームスペースの SPIFFE URI が正しく解析されることを確認する。
#[test]
fn test_spiffe_id_parse_service() {
    let spiffe = SpiffeId::parse("spiffe://example.io/ns/service/sa/notification-svc").unwrap();
    assert_eq!(spiffe.trust_domain, "example.io");
    assert_eq!(spiffe.namespace, "service");
    assert_eq!(spiffe.service_account, "notification-svc");
}

// "spiffe://" プレフィックスがない URI の解析がエラーになることを確認する。
#[test]
fn test_spiffe_id_parse_invalid_prefix() {
    let result = SpiffeId::parse("https://k1s0.internal/ns/system/sa/svc");
    assert!(result.is_err());
}

// 空文字列の解析がエラーになることを確認する。
#[test]
fn test_spiffe_id_parse_empty() {
    let result = SpiffeId::parse("");
    assert!(result.is_err());
}

// パスが不正な SPIFFE URI の解析がエラーになることを確認する。
#[test]
fn test_spiffe_id_parse_wrong_format() {
    let result = SpiffeId::parse("spiffe://k1s0.internal/wrong/path");
    assert!(result.is_err());
}

// ネームスペースが空の SPIFFE URI の解析がエラーになることを確認する。
#[test]
fn test_spiffe_id_parse_empty_namespace() {
    let result = SpiffeId::parse("spiffe://k1s0.internal/ns//sa/svc");
    assert!(result.is_err());
}

// to_uri で元の URI 文字列に戻せることを確認する。
#[test]
fn test_spiffe_id_roundtrip() {
    let original = "spiffe://k1s0.internal/ns/system/sa/auth-service";
    let spiffe = SpiffeId::parse(original).unwrap();
    assert_eq!(spiffe.to_uri(), original);
}

// allows_tier_access が k1s0 の Region 階層に基づいて正しくアクセス制御することを確認する。
#[test]
fn test_spiffe_id_tier_access() {
    let system_svc = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
    assert!(system_svc.allows_tier_access("system"));
    assert!(system_svc.allows_tier_access("business"));
    assert!(system_svc.allows_tier_access("service"));

    let business_svc =
        SpiffeId::parse("spiffe://k1s0.internal/ns/business/sa/task-server").unwrap();
    assert!(!business_svc.allows_tier_access("system"));
    assert!(business_svc.allows_tier_access("business"));
    assert!(business_svc.allows_tier_access("service"));

    let service_svc = SpiffeId::parse("spiffe://k1s0.internal/ns/service/sa/leaf-service").unwrap();
    assert!(!service_svc.allows_tier_access("system"));
    assert!(!service_svc.allows_tier_access("business"));
    assert!(service_svc.allows_tier_access("service"));
}

// SpiffeId の PartialEq が正しく機能することを確認する。
#[test]
fn test_spiffe_id_equality() {
    let a = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
    let b = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
    let c = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/other-service").unwrap();
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// --- ServiceToken テスト ---

// ServiceToken::new が正しいフィールドで生成されることを確認する。
#[test]
fn test_service_token_new() {
    let token = ServiceToken::new("my-access-token".to_string(), "Bearer".to_string(), 900);
    assert_eq!(token.access_token, "my-access-token");
    assert_eq!(token.token_type, "Bearer");
    assert_eq!(token.expires_in, 900);
}

// bearer_header が "Bearer <token>" 形式の文字列を返すことを確認する。
#[test]
fn test_service_token_bearer_header() {
    let token = ServiceToken::new("abc-def-123".to_string(), "Bearer".to_string(), 3600);
    assert_eq!(token.bearer_header(), "Bearer abc-def-123");
}

// 新規作成直後のトークンが有効期限切れでないことを確認する。
#[test]
fn test_service_token_not_expired() {
    let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 3600);
    assert!(!token.is_expired());
}

// expires_in が 0 のトークンが即座に期限切れになることを確認する。
#[test]
fn test_service_token_zero_expires_in() {
    let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 0);
    assert!(token.is_expired());
}

// 新規作成直後のトークンがリフレッシュ不要であることを確認する。
#[test]
fn test_service_token_should_not_refresh_when_fresh() {
    let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 3600);
    assert!(!token.should_refresh(120));
}

// ServiceToken の Clone が正しく動作することを確認する。
#[test]
fn test_service_token_clone() {
    let token = ServiceToken::new("clone-me".to_string(), "Bearer".to_string(), 1800);
    let cloned = token.clone();
    assert_eq!(cloned.access_token, token.access_token);
    assert_eq!(cloned.token_type, token.token_type);
    assert_eq!(cloned.expires_in, token.expires_in);
    assert_eq!(cloned.acquired_at, token.acquired_at);
}

// --- ServiceAuthConfig テスト ---

// ServiceAuthConfig::new がデフォルト値を正しく設定することを確認する。
#[test]
fn test_config_defaults() {
    let config =
        ServiceAuthConfig::new("https://auth.example.com/token", "my-service", "my-secret");
    assert_eq!(config.token_endpoint, "https://auth.example.com/token");
    assert_eq!(config.client_id, "my-service");
    assert_eq!(config.client_secret, "my-secret");
    assert!(config.jwks_uri.is_none());
    assert_eq!(config.refresh_before_secs, 120);
    assert_eq!(config.timeout_secs, 10);
}

// ビルダーメソッドチェーンで全オプションを設定できることを確認する。
#[test]
fn test_config_builder_chain() {
    let config = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec")
        .with_jwks_uri("https://auth.example.com/certs")
        .with_refresh_before_secs(60)
        .with_timeout_secs(30);

    assert_eq!(
        config.jwks_uri.as_deref(),
        Some("https://auth.example.com/certs")
    );
    assert_eq!(config.refresh_before_secs, 60);
    assert_eq!(config.timeout_secs, 30);
}

// ServiceAuthConfig が serde デシリアライズでデフォルト値を適用することを確認する。
#[test]
fn test_config_serde_defaults() {
    let json = r#"{
        "token_endpoint": "https://auth.example.com/token",
        "client_id": "svc",
        "client_secret": "sec"
    }"#;
    let config: ServiceAuthConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.refresh_before_secs, 120);
    assert_eq!(config.timeout_secs, 10);
    assert!(config.jwks_uri.is_none());
}

// ServiceAuthConfig の JSON ラウンドトリップが全フィールドを保持することを確認する。
#[test]
fn test_config_serde_roundtrip() {
    let original = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec")
        .with_jwks_uri("https://auth.example.com/certs")
        .with_refresh_before_secs(90)
        .with_timeout_secs(15);

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ServiceAuthConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.token_endpoint, original.token_endpoint);
    assert_eq!(deserialized.client_id, original.client_id);
    assert_eq!(deserialized.client_secret, original.client_secret);
    assert_eq!(deserialized.jwks_uri, original.jwks_uri);
    assert_eq!(
        deserialized.refresh_before_secs,
        original.refresh_before_secs
    );
    assert_eq!(deserialized.timeout_secs, original.timeout_secs);
}

// --- エラーバリアントテスト ---

// ServiceAuthError の各バリアントが適切な Display 出力を生成することを確認する。
#[test]
fn test_error_display() {
    let err = ServiceAuthError::TokenExpired;
    assert!(err.to_string().contains("有効期限"));

    let err = ServiceAuthError::InvalidToken("bad token".to_string());
    assert!(err.to_string().contains("bad token"));

    let err = ServiceAuthError::SpiffeValidationFailed("ns mismatch".to_string());
    assert!(err.to_string().contains("ns mismatch"));

    let err = ServiceAuthError::Http("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}
