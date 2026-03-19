//! Integration tests for k1s0-test-helper.
#![allow(clippy::unwrap_used)]

use k1s0_test_helper::{
    AssertionHelper, FixtureBuilder, JwtTestHelper, MockServerBuilder, TestClaims,
};
use serde_json::json;

// ============================================================
// JwtTestHelper tests
// ============================================================

// HS256 を用いて JwtTestHelper を生成し、管理者トークンが空でないことを確認する。
#[test]
fn jwt_helper_new_hs256_creates_instance() {
    let helper = JwtTestHelper::new_hs256("my-secret");
    let token = helper.create_admin_token();
    assert!(!token.is_empty());
}

// `new` エイリアスが `new_hs256` と同等に動作することを確認する。
#[test]
fn jwt_helper_new_alias_works() {
    let helper = JwtTestHelper::new("my-secret");
    let token = helper.create_admin_token();
    assert!(!token.is_empty());
}

// 管理者トークンが header.payload.signature の 3 パーツで構成されることを確認する。
#[test]
fn jwt_admin_token_has_three_parts() {
    let helper = JwtTestHelper::new_hs256("secret");
    let token = helper.create_admin_token();
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "JWT should have 3 parts: header.payload.signature"
    );
}

// 管理者トークンの sub が "admin" であり roles に "admin" が含まれることを確認する。
#[test]
fn jwt_admin_token_has_admin_role() {
    let helper = JwtTestHelper::new_hs256("secret");
    let token = helper.create_admin_token();
    let claims = helper
        .decode_claims(&token)
        .expect("admin token should decode");
    assert_eq!(claims.sub, "admin");
    assert!(claims.roles.contains(&"admin".to_string()));
}

// ユーザートークンの sub と roles が指定した値と一致することを確認する。
#[test]
fn jwt_user_token_has_correct_sub_and_roles() {
    let helper = JwtTestHelper::new_hs256("test-secret");
    let roles = vec!["user".to_string(), "reader".to_string()];
    let token = helper.create_user_token("user-42", roles.clone());
    let claims = helper
        .decode_claims(&token)
        .expect("user token should decode");
    assert_eq!(claims.sub, "user-42");
    assert_eq!(claims.roles, roles);
}

// カスタムクレームにテナント ID を含むトークンが正しくデコードされることを確認する。
#[test]
fn jwt_custom_token_with_tenant_id() {
    let helper = JwtTestHelper::new_hs256("secret");
    let claims = TestClaims {
        sub: "svc-account".to_string(),
        roles: vec!["service".to_string()],
        tenant_id: Some("tenant-xyz".to_string()),
        ..Default::default()
    };
    let token = helper.create_token(&claims);
    let decoded = helper
        .decode_claims(&token)
        .expect("custom token with tenant should decode");
    assert_eq!(decoded.sub, "svc-account");
    assert_eq!(decoded.tenant_id, Some("tenant-xyz".to_string()));
    assert_eq!(decoded.roles, vec!["service"]);
}

// テナント ID なしのクレームでトークンを生成した場合に tenant_id が None になることを確認する。
#[test]
fn jwt_custom_token_without_tenant_id() {
    let helper = JwtTestHelper::new_hs256("secret");
    let claims = TestClaims {
        sub: "user-1".to_string(),
        roles: vec![],
        tenant_id: None,
        ..Default::default()
    };
    let token = helper.create_token(&claims);
    let decoded = helper
        .decode_claims(&token)
        .expect("custom token without tenant should decode");
    assert!(decoded.tenant_id.is_none());
}

// 不正な形式の JWT をデコードした場合に None が返されることを確認する。
#[test]
fn jwt_decode_invalid_token_returns_none() {
    let helper = JwtTestHelper::new_hs256("secret");
    assert!(helper.decode_claims("not-a-jwt").is_none());
    assert!(helper.decode_claims("a.b").is_none());
    assert!(helper.decode_claims("").is_none());
}

// デフォルトクレームの exp が iat より大きく約 1 時間後であることを確認する。
#[test]
fn jwt_default_claims_has_valid_expiry() {
    let claims = TestClaims::default();
    assert!(claims.exp > claims.iat);
    // Should have ~1 hour expiry
    let diff = claims.exp - claims.iat;
    assert!((3599..=3601).contains(&diff));
}

// 異なるシークレットで生成したトークンの署名部分が異なることを確認する。
#[test]
fn jwt_different_secrets_produce_different_tokens() {
    let helper1 = JwtTestHelper::new_hs256("secret-1");
    let helper2 = JwtTestHelper::new_hs256("secret-2");

    let claims = TestClaims {
        sub: "user-1".to_string(),
        roles: vec![],
        tenant_id: None,
        ..Default::default()
    };

    let token1 = helper1.create_token(&claims);
    let token2 = helper2.create_token(&claims);

    // 署名部分が異なることを確認する
    let sig1 = token1
        .rsplit('.')
        .next()
        .expect("token1 should have signature part");
    let sig2 = token2
        .rsplit('.')
        .next()
        .expect("token2 should have signature part");
    assert_ne!(sig1, sig2);
}

// ============================================================
// FixtureBuilder tests
// ============================================================

// 生成された UUID が 36 文字でハイフン区切りの正しい形式であることを確認する。
#[test]
fn fixture_uuid_has_valid_format() {
    let id = FixtureBuilder::uuid();
    assert_eq!(id.len(), 36);
    assert!(id.contains('-'));
    // UUID format: 8-4-4-4-12
    let parts: Vec<&str> = id.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}

// 複数回呼び出しで一意の UUID が生成されることを確認する。
#[test]
fn fixture_uuid_generates_unique_values() {
    let ids: Vec<String> = (0..10).map(|_| FixtureBuilder::uuid()).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "UUIDs should be unique");
        }
    }
}

// 生成されたメールアドレスが "test-" 始まりで "@example.com" 終わりの形式であることを確認する。
#[test]
fn fixture_email_has_valid_format() {
    let email = FixtureBuilder::email();
    assert!(email.starts_with("test-"));
    assert!(email.ends_with("@example.com"));
    assert!(email.contains('@'));
}

// 複数回呼び出しで一意のメールアドレスが生成されることを確認する。
#[test]
fn fixture_email_generates_unique_values() {
    let a = FixtureBuilder::email();
    let b = FixtureBuilder::email();
    assert_ne!(a, b);
}

// 生成されたユーザー名が "user-" プレフィックスを持つことを確認する。
#[test]
fn fixture_name_has_prefix() {
    let name = FixtureBuilder::name();
    assert!(name.starts_with("user-"));
    assert!(name.len() > 5);
}

// 生成された整数が指定した範囲内に収まることを確認する。
#[test]
fn fixture_int_within_range() {
    for _ in 0..100 {
        let val = FixtureBuilder::int(1, 10);
        assert!((1..10).contains(&val), "val={} should be in [1, 10)", val);
    }
}

// min と max が等しい場合は min の値がそのまま返されることを確認する。
#[test]
fn fixture_int_min_equals_max_returns_min() {
    assert_eq!(FixtureBuilder::int(42, 42), 42);
}

// min が max より大きい場合は min の値が返されることを確認する。
#[test]
fn fixture_int_min_greater_than_max_returns_min() {
    assert_eq!(FixtureBuilder::int(10, 5), 10);
}

// 広範囲の整数生成で値が範囲内に収まることを確認する。
#[test]
fn fixture_int_large_range() {
    for _ in 0..50 {
        let val = FixtureBuilder::int(0, 1_000_000);
        assert!((0..1_000_000).contains(&val));
    }
}

// 生成されたテナント ID が "tenant-" プレフィックスを持つことを確認する。
#[test]
fn fixture_tenant_id_has_prefix() {
    let tid = FixtureBuilder::tenant_id();
    assert!(tid.starts_with("tenant-"));
    assert!(tid.len() > 7);
}

// 複数回呼び出しで一意のテナント ID が生成されることを確認する。
#[test]
fn fixture_tenant_id_unique() {
    let a = FixtureBuilder::tenant_id();
    let b = FixtureBuilder::tenant_id();
    assert_ne!(a, b);
}

// ============================================================
// MockServer / MockServerBuilder tests
// ============================================================

// 通知サーバーモックにヘルスチェックルートを追加し、正しいステータスを返すことを確認する。
#[test]
fn mock_server_builder_notification_with_health() {
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .build();

    let (status, body) = server
        .handle("GET", "/health")
        .expect("health route should match");
    assert_eq!(status, 200);
    assert!(body.contains("ok"));
}

// レートリミットサーバービルダーのサーバータイプが "ratelimit" であることを確認する。
#[test]
fn mock_server_builder_ratelimit_type() {
    let builder = MockServerBuilder::ratelimit_server();
    assert_eq!(builder.server_type(), "ratelimit");
}

// テナントサーバービルダーのサーバータイプが "tenant" であることを確認する。
#[test]
fn mock_server_builder_tenant_type() {
    let builder = MockServerBuilder::tenant_server();
    assert_eq!(builder.server_type(), "tenant");
}

// 通知サーバービルダーのサーバータイプが "notification" であることを確認する。
#[test]
fn mock_server_builder_notification_type() {
    let builder = MockServerBuilder::notification_server();
    assert_eq!(builder.server_type(), "notification");
}

// 成功レスポンスルートが登録され、200 ステータスと期待したボディを返すことを確認する。
#[test]
fn mock_server_success_response() {
    let server = MockServerBuilder::notification_server()
        .with_success_response("/send", r#"{"id":"1","status":"sent"}"#)
        .build();

    let (status, body) = server
        .handle("POST", "/send")
        .expect("send route should match");
    assert_eq!(status, 200);
    assert!(body.contains("sent"));
}

// エラーレスポンスルートが登録され、指定したエラーステータスを返すことを確認する。
#[test]
fn mock_server_error_response() {
    let server = MockServerBuilder::tenant_server()
        .with_error_response("/create", 500)
        .build();

    let (status, body) = server
        .handle("POST", "/create")
        .expect("create route should match");
    assert_eq!(status, 500);
    assert!(body.contains("error"));
}

// 未登録ルートへのリクエストで None が返されることを確認する。
#[test]
fn mock_server_unregistered_route_returns_none() {
    let server = MockServerBuilder::ratelimit_server()
        .with_health_ok()
        .build();
    assert!(server.handle("GET", "/unknown").is_none());
    assert!(server.handle("POST", "/health").is_none()); // wrong method
}

// モックサーバーがリクエストを記録し、カウントが正しく増加することを確認する。
#[test]
fn mock_server_records_requests() {
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .with_success_response("/send", r#"{"ok":true}"#)
        .build();

    assert_eq!(server.request_count(), 0);

    server.handle("GET", "/health");
    assert_eq!(server.request_count(), 1);

    server.handle("POST", "/send");
    assert_eq!(server.request_count(), 2);

    // Even unmatched routes get recorded
    server.handle("DELETE", "/nothing");
    assert_eq!(server.request_count(), 3);
}

// 記録されたリクエストのメソッドとパスが正しい順序で保存されていることを確認する。
#[test]
fn mock_server_recorded_requests_content() {
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .build();

    server.handle("GET", "/health");
    server.handle("POST", "/api/send");

    let reqs = server.recorded_requests();
    assert_eq!(reqs.len(), 2);
    assert_eq!(reqs[0], ("GET".to_string(), "/health".to_string()));
    assert_eq!(reqs[1], ("POST".to_string(), "/api/send".to_string()));
}

// モックサーバーのベース URL が "http://" で始まることを確認する。
#[test]
fn mock_server_base_url() {
    let server = MockServerBuilder::notification_server().build();
    let url = server.base_url();
    assert!(url.starts_with("http://"));
}

// 複数のルートを登録したモックサーバーがそれぞれ正しいステータスを返すことを確認する。
#[test]
fn mock_server_multiple_routes() {
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .with_success_response("/send", r#"{"sent":true}"#)
        .with_success_response("/batch", r#"{"count":5}"#)
        .with_error_response("/fail", 503)
        .build();

    assert_eq!(
        server.handle("GET", "/health").expect("health route").0,
        200
    );
    assert_eq!(server.handle("POST", "/send").expect("send route").0, 200);
    assert_eq!(server.handle("POST", "/batch").expect("batch route").0, 200);
    assert_eq!(server.handle("POST", "/fail").expect("fail route").0, 503);
}

// メソッドチェーンでビルダーを構築した後のリクエストカウントが 0 であることを確認する。
#[test]
fn mock_server_builder_chaining() {
    // Verify fluent API works
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .with_success_response("/a", "{}")
        .with_success_response("/b", "{}")
        .with_error_response("/c", 400)
        .build();

    assert_eq!(server.request_count(), 0);
}

// デフォルトモックルートにヘルスチェックルートが含まれ正しい値を持つことを確認する。
#[test]
fn default_mock_routes_contains_health() {
    let routes = k1s0_test_helper::mock_server::default_mock_routes("my-service");
    assert!(routes.contains_key("health"));
    let health = &routes["health"];
    assert_eq!(health.method, "GET");
    assert_eq!(health.path, "/health");
    assert_eq!(health.status, 200);
    assert!(health.body.contains("my-service"));
}

// ============================================================
// AssertionHelper tests
// ============================================================

// 余分なキーを含む JSON が期待値の全キーを包含していることを確認する。
#[test]
fn assertion_json_contains_simple_match() {
    AssertionHelper::assert_json_contains(
        r#"{"id":"1","name":"test","extra":"data"}"#,
        r#"{"id":"1","name":"test"}"#,
    );
}

// ネストされた JSON オブジェクトの部分一致が正しく機能することを確認する。
#[test]
fn assertion_json_contains_nested_match() {
    AssertionHelper::assert_json_contains(
        r#"{"user":{"id":"1","name":"test","age":30},"status":"active"}"#,
        r#"{"user":{"id":"1"},"status":"active"}"#,
    );
}

// JSON 配列の部分一致アサーションが正しく機能することを確認する。
#[test]
fn assertion_json_contains_array_match() {
    AssertionHelper::assert_json_contains(
        r#"{"items":[{"id":"1"},{"id":"2"},{"id":"3"}]}"#,
        r#"{"items":[{"id":"2"}]}"#,
    );
}

// 値が一致しない場合に "JSON partial match failed" でパニックすることを確認する。
#[test]
#[should_panic(expected = "JSON partial match failed")]
fn assertion_json_contains_mismatch_panics() {
    AssertionHelper::assert_json_contains(r#"{"id":"1"}"#, r#"{"id":"2"}"#);
}

// 期待値のキーが実際の JSON に存在しない場合にパニックすることを確認する。
#[test]
#[should_panic(expected = "JSON partial match failed")]
fn assertion_json_contains_missing_key_panics() {
    AssertionHelper::assert_json_contains(r#"{"id":"1"}"#, r#"{"id":"1","name":"test"}"#);
}

// イベント一覧に指定タイプのイベントが含まれていることを検証できることを確認する。
#[test]
fn assertion_event_emitted_finds_event() {
    let events = vec![
        json!({"type": "order_created", "id": "1"}),
        json!({"type": "order_shipped", "id": "2"}),
    ];
    AssertionHelper::assert_event_emitted(&events, "order_created");
    AssertionHelper::assert_event_emitted(&events, "order_shipped");
}

// 存在しないイベントタイプを検証すると "not found" でパニックすることを確認する。
#[test]
#[should_panic(expected = "not found")]
fn assertion_event_emitted_missing_panics() {
    let events = vec![json!({"type": "created"})];
    AssertionHelper::assert_event_emitted(&events, "deleted");
}

// 指定パスに非 null 値が存在する場合にアサーションが成功することを確認する。
#[test]
fn assertion_not_null_with_value() {
    AssertionHelper::assert_not_null(r#"{"data":{"id":"abc","name":"test"}}"#, "data.id");
    AssertionHelper::assert_not_null(r#"{"data":{"id":"abc","name":"test"}}"#, "data.name");
}

// 存在しないパスを指定した場合に "non-null" でパニックすることを確認する。
#[test]
#[should_panic(expected = "non-null")]
fn assertion_not_null_with_missing_path_panics() {
    AssertionHelper::assert_not_null(r#"{"data":{}}"#, "data.missing_field");
}

// 値が null の場合に "non-null" でパニックすることを確認する。
#[test]
#[should_panic(expected = "non-null")]
fn assertion_not_null_with_null_value_panics() {
    AssertionHelper::assert_not_null(r#"{"data":{"id":null}}"#, "data.id");
}
