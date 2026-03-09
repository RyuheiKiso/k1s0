//! Integration tests for k1s0-test-helper.

use k1s0_test_helper::{
    AssertionHelper, FixtureBuilder, JwtTestHelper, MockServerBuilder, TestClaims,
};
use serde_json::json;

// ============================================================
// JwtTestHelper tests
// ============================================================

#[test]
fn jwt_helper_new_hs256_creates_instance() {
    let helper = JwtTestHelper::new_hs256("my-secret");
    let token = helper.create_admin_token();
    assert!(!token.is_empty());
}

#[test]
fn jwt_helper_new_alias_works() {
    let helper = JwtTestHelper::new("my-secret");
    let token = helper.create_admin_token();
    assert!(!token.is_empty());
}

#[test]
fn jwt_admin_token_has_three_parts() {
    let helper = JwtTestHelper::new_hs256("secret");
    let token = helper.create_admin_token();
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts: header.payload.signature");
}

#[test]
fn jwt_admin_token_has_admin_role() {
    let helper = JwtTestHelper::new_hs256("secret");
    let token = helper.create_admin_token();
    let claims = helper.decode_claims(&token).unwrap();
    assert_eq!(claims.sub, "admin");
    assert!(claims.roles.contains(&"admin".to_string()));
}

#[test]
fn jwt_user_token_has_correct_sub_and_roles() {
    let helper = JwtTestHelper::new_hs256("test-secret");
    let roles = vec!["user".to_string(), "reader".to_string()];
    let token = helper.create_user_token("user-42", roles.clone());
    let claims = helper.decode_claims(&token).unwrap();
    assert_eq!(claims.sub, "user-42");
    assert_eq!(claims.roles, roles);
}

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
    let decoded = helper.decode_claims(&token).unwrap();
    assert_eq!(decoded.sub, "svc-account");
    assert_eq!(decoded.tenant_id, Some("tenant-xyz".to_string()));
    assert_eq!(decoded.roles, vec!["service"]);
}

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
    let decoded = helper.decode_claims(&token).unwrap();
    assert!(decoded.tenant_id.is_none());
}

#[test]
fn jwt_decode_invalid_token_returns_none() {
    let helper = JwtTestHelper::new_hs256("secret");
    assert!(helper.decode_claims("not-a-jwt").is_none());
    assert!(helper.decode_claims("a.b").is_none());
    assert!(helper.decode_claims("").is_none());
}

#[test]
fn jwt_default_claims_has_valid_expiry() {
    let claims = TestClaims::default();
    assert!(claims.exp > claims.iat);
    // Should have ~1 hour expiry
    let diff = claims.exp - claims.iat;
    assert!(diff >= 3599 && diff <= 3601);
}

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

    // Signatures should differ
    let sig1 = token1.rsplit('.').next().unwrap();
    let sig2 = token2.rsplit('.').next().unwrap();
    assert_ne!(sig1, sig2);
}

// ============================================================
// FixtureBuilder tests
// ============================================================

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

#[test]
fn fixture_uuid_generates_unique_values() {
    let ids: Vec<String> = (0..10).map(|_| FixtureBuilder::uuid()).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "UUIDs should be unique");
        }
    }
}

#[test]
fn fixture_email_has_valid_format() {
    let email = FixtureBuilder::email();
    assert!(email.starts_with("test-"));
    assert!(email.ends_with("@example.com"));
    assert!(email.contains('@'));
}

#[test]
fn fixture_email_generates_unique_values() {
    let a = FixtureBuilder::email();
    let b = FixtureBuilder::email();
    assert_ne!(a, b);
}

#[test]
fn fixture_name_has_prefix() {
    let name = FixtureBuilder::name();
    assert!(name.starts_with("user-"));
    assert!(name.len() > 5);
}

#[test]
fn fixture_int_within_range() {
    for _ in 0..100 {
        let val = FixtureBuilder::int(1, 10);
        assert!(val >= 1 && val < 10, "val={} should be in [1, 10)", val);
    }
}

#[test]
fn fixture_int_min_equals_max_returns_min() {
    assert_eq!(FixtureBuilder::int(42, 42), 42);
}

#[test]
fn fixture_int_min_greater_than_max_returns_min() {
    assert_eq!(FixtureBuilder::int(10, 5), 10);
}

#[test]
fn fixture_int_large_range() {
    for _ in 0..50 {
        let val = FixtureBuilder::int(0, 1_000_000);
        assert!(val >= 0 && val < 1_000_000);
    }
}

#[test]
fn fixture_tenant_id_has_prefix() {
    let tid = FixtureBuilder::tenant_id();
    assert!(tid.starts_with("tenant-"));
    assert!(tid.len() > 7);
}

#[test]
fn fixture_tenant_id_unique() {
    let a = FixtureBuilder::tenant_id();
    let b = FixtureBuilder::tenant_id();
    assert_ne!(a, b);
}

// ============================================================
// MockServer / MockServerBuilder tests
// ============================================================

#[test]
fn mock_server_builder_notification_with_health() {
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .build();

    let (status, body) = server.handle("GET", "/health").unwrap();
    assert_eq!(status, 200);
    assert!(body.contains("ok"));
}

#[test]
fn mock_server_builder_ratelimit_type() {
    let builder = MockServerBuilder::ratelimit_server();
    assert_eq!(builder.server_type(), "ratelimit");
}

#[test]
fn mock_server_builder_tenant_type() {
    let builder = MockServerBuilder::tenant_server();
    assert_eq!(builder.server_type(), "tenant");
}

#[test]
fn mock_server_builder_notification_type() {
    let builder = MockServerBuilder::notification_server();
    assert_eq!(builder.server_type(), "notification");
}

#[test]
fn mock_server_success_response() {
    let server = MockServerBuilder::notification_server()
        .with_success_response("/send", r#"{"id":"1","status":"sent"}"#)
        .build();

    let (status, body) = server.handle("POST", "/send").unwrap();
    assert_eq!(status, 200);
    assert!(body.contains("sent"));
}

#[test]
fn mock_server_error_response() {
    let server = MockServerBuilder::tenant_server()
        .with_error_response("/create", 500)
        .build();

    let (status, body) = server.handle("POST", "/create").unwrap();
    assert_eq!(status, 500);
    assert!(body.contains("error"));
}

#[test]
fn mock_server_unregistered_route_returns_none() {
    let server = MockServerBuilder::ratelimit_server()
        .with_health_ok()
        .build();
    assert!(server.handle("GET", "/unknown").is_none());
    assert!(server.handle("POST", "/health").is_none()); // wrong method
}

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

#[test]
fn mock_server_base_url() {
    let server = MockServerBuilder::notification_server().build();
    let url = server.base_url();
    assert!(url.starts_with("http://"));
}

#[test]
fn mock_server_multiple_routes() {
    let server = MockServerBuilder::notification_server()
        .with_health_ok()
        .with_success_response("/send", r#"{"sent":true}"#)
        .with_success_response("/batch", r#"{"count":5}"#)
        .with_error_response("/fail", 503)
        .build();

    assert_eq!(server.handle("GET", "/health").unwrap().0, 200);
    assert_eq!(server.handle("POST", "/send").unwrap().0, 200);
    assert_eq!(server.handle("POST", "/batch").unwrap().0, 200);
    assert_eq!(server.handle("POST", "/fail").unwrap().0, 503);
}

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

#[test]
fn assertion_json_contains_simple_match() {
    AssertionHelper::assert_json_contains(
        r#"{"id":"1","name":"test","extra":"data"}"#,
        r#"{"id":"1","name":"test"}"#,
    );
}

#[test]
fn assertion_json_contains_nested_match() {
    AssertionHelper::assert_json_contains(
        r#"{"user":{"id":"1","name":"test","age":30},"status":"active"}"#,
        r#"{"user":{"id":"1"},"status":"active"}"#,
    );
}

#[test]
fn assertion_json_contains_array_match() {
    AssertionHelper::assert_json_contains(
        r#"{"items":[{"id":"1"},{"id":"2"},{"id":"3"}]}"#,
        r#"{"items":[{"id":"2"}]}"#,
    );
}

#[test]
#[should_panic(expected = "JSON partial match failed")]
fn assertion_json_contains_mismatch_panics() {
    AssertionHelper::assert_json_contains(r#"{"id":"1"}"#, r#"{"id":"2"}"#);
}

#[test]
#[should_panic(expected = "JSON partial match failed")]
fn assertion_json_contains_missing_key_panics() {
    AssertionHelper::assert_json_contains(r#"{"id":"1"}"#, r#"{"id":"1","name":"test"}"#);
}

#[test]
fn assertion_event_emitted_finds_event() {
    let events = vec![
        json!({"type": "order_created", "id": "1"}),
        json!({"type": "order_shipped", "id": "2"}),
    ];
    AssertionHelper::assert_event_emitted(&events, "order_created");
    AssertionHelper::assert_event_emitted(&events, "order_shipped");
}

#[test]
#[should_panic(expected = "not found")]
fn assertion_event_emitted_missing_panics() {
    let events = vec![json!({"type": "created"})];
    AssertionHelper::assert_event_emitted(&events, "deleted");
}

#[test]
fn assertion_not_null_with_value() {
    AssertionHelper::assert_not_null(r#"{"data":{"id":"abc","name":"test"}}"#, "data.id");
    AssertionHelper::assert_not_null(r#"{"data":{"id":"abc","name":"test"}}"#, "data.name");
}

#[test]
#[should_panic(expected = "non-null")]
fn assertion_not_null_with_missing_path_panics() {
    AssertionHelper::assert_not_null(r#"{"data":{}}"#, "data.missing_field");
}

#[test]
#[should_panic(expected = "non-null")]
fn assertion_not_null_with_null_value_panics() {
    AssertionHelper::assert_not_null(r#"{"data":{"id":null}}"#, "data.id");
}
