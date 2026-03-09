//! External integration tests for k1s0-auth.
//!
//! Complements the inline tests in src/tests.rs by testing the public API
//! without relying on internal test helpers.

use k1s0_auth::{
    actor_from_claims, check_permission, has_permission, has_resource_role, has_role,
    has_tier_access, Claims, RealmAccess, RoleSet,
};


// ============================================================
// Test helpers
// ============================================================

/// Build Claims with the given realm roles, resource access, and tier access.
fn build_claims(
    sub: &str,
    realm_roles: Vec<&str>,
    resource_access: Vec<(&str, Vec<&str>)>,
    tier_access: Vec<&str>,
) -> Claims {
    Claims {
        sub: sub.into(),
        iss: "https://auth.example.com/realms/k1s0".into(),
        aud: k1s0_auth::Audience(vec!["k1s0-api".into()]),
        exp: 9999999999,
        iat: 1000000000,
        jti: None,
        typ: Some("Bearer".into()),
        azp: None,
        scope: Some("openid profile email".into()),
        preferred_username: None,
        email: None,
        realm_access: Some(RealmAccess {
            roles: realm_roles.into_iter().map(String::from).collect(),
        }),
        resource_access: Some(
            resource_access
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        RoleSet {
                            roles: v.into_iter().map(String::from).collect(),
                        },
                    )
                })
                .collect(),
        ),
        tier_access: Some(tier_access.into_iter().map(String::from).collect()),
    }
}

/// Build minimal Claims with no roles or access.
fn build_minimal_claims(sub: &str) -> Claims {
    Claims {
        sub: sub.into(),
        iss: "https://auth.example.com/realms/k1s0".into(),
        aud: k1s0_auth::Audience(vec![]),
        exp: 9999999999,
        iat: 1000000000,
        jti: None,
        typ: None,
        azp: None,
        scope: None,
        preferred_username: None,
        email: None,
        realm_access: None,
        resource_access: None,
        tier_access: None,
    }
}

// ============================================================
// has_role tests
// ============================================================

#[test]
fn has_role_returns_true_for_existing_role() {
    let claims = build_claims("user-1", vec!["user", "editor"], vec![], vec![]);
    assert!(has_role(&claims, "user"));
    assert!(has_role(&claims, "editor"));
}

#[test]
fn has_role_returns_false_for_missing_role() {
    let claims = build_claims("user-1", vec!["user"], vec![], vec![]);
    assert!(!has_role(&claims, "admin"));
    assert!(!has_role(&claims, "sys_admin"));
}

#[test]
fn has_role_returns_false_when_realm_access_is_none() {
    let claims = build_minimal_claims("user-1");
    assert!(!has_role(&claims, "user"));
}

#[test]
fn has_role_with_empty_roles() {
    let claims = build_claims("user-1", vec![], vec![], vec![]);
    assert!(!has_role(&claims, "user"));
}

// ============================================================
// has_resource_role tests
// ============================================================

#[test]
fn has_resource_role_returns_true_for_matching_resource_and_role() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["read", "write"])],
        vec![],
    );
    assert!(has_resource_role(&claims, "order-service", "read"));
    assert!(has_resource_role(&claims, "order-service", "write"));
}

#[test]
fn has_resource_role_returns_false_for_wrong_role() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["read"])],
        vec![],
    );
    assert!(!has_resource_role(&claims, "order-service", "delete"));
}

#[test]
fn has_resource_role_returns_false_for_wrong_resource() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["read"])],
        vec![],
    );
    assert!(!has_resource_role(&claims, "user-service", "read"));
}

#[test]
fn has_resource_role_returns_false_when_resource_access_is_none() {
    let claims = build_minimal_claims("user-1");
    assert!(!has_resource_role(&claims, "order-service", "read"));
}

// ============================================================
// check_permission tests
// ============================================================

#[test]
fn check_permission_grants_via_resource_role() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["read", "write"])],
        vec![],
    );
    assert!(check_permission(&claims, "order-service", "read"));
    assert!(check_permission(&claims, "order-service", "write"));
    assert!(!check_permission(&claims, "order-service", "delete"));
}

#[test]
fn check_permission_sys_admin_grants_all() {
    let claims = build_claims("admin-1", vec!["sys_admin"], vec![], vec![]);
    assert!(check_permission(&claims, "any-resource", "read"));
    assert!(check_permission(&claims, "any-resource", "write"));
    assert!(check_permission(&claims, "any-resource", "delete"));
    assert!(check_permission(&claims, "another-resource", "admin"));
}

#[test]
fn check_permission_realm_admin_grants_all() {
    let claims = build_claims("admin-2", vec!["admin"], vec![], vec![]);
    assert!(check_permission(&claims, "order-service", "read"));
    assert!(check_permission(&claims, "user-service", "write"));
}

#[test]
fn check_permission_resource_admin_grants_all_on_that_resource() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["admin"])],
        vec![],
    );
    assert!(check_permission(&claims, "order-service", "read"));
    assert!(check_permission(&claims, "order-service", "write"));
    assert!(check_permission(&claims, "order-service", "delete"));
    // But not on other resources
    assert!(!check_permission(&claims, "user-service", "read"));
}

#[test]
fn check_permission_denies_when_no_matching_role() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["read"])],
        vec![],
    );
    assert!(!check_permission(&claims, "order-service", "delete"));
    assert!(!check_permission(&claims, "payment-service", "read"));
}

#[test]
fn check_permission_denies_with_no_roles_at_all() {
    let claims = build_minimal_claims("user-1");
    assert!(!check_permission(&claims, "order-service", "read"));
}

// ============================================================
// has_permission tests (alias)
// ============================================================

#[test]
fn has_permission_is_alias_for_check_permission() {
    let claims = build_claims(
        "user-1",
        vec!["user"],
        vec![("order-service", vec!["read"])],
        vec![],
    );
    assert_eq!(
        has_permission(&claims, "order-service", "read"),
        check_permission(&claims, "order-service", "read")
    );
    assert_eq!(
        has_permission(&claims, "order-service", "write"),
        check_permission(&claims, "order-service", "write")
    );
}

// ============================================================
// has_tier_access tests
// ============================================================

#[test]
fn has_tier_access_returns_true_for_allowed_tier() {
    let claims = build_claims("user-1", vec![], vec![], vec!["system", "business"]);
    assert!(has_tier_access(&claims, "system"));
    assert!(has_tier_access(&claims, "business"));
}

#[test]
fn has_tier_access_returns_false_for_disallowed_tier() {
    let claims = build_claims("user-1", vec![], vec![], vec!["system"]);
    assert!(!has_tier_access(&claims, "business"));
    assert!(!has_tier_access(&claims, "service"));
}

#[test]
fn has_tier_access_is_case_insensitive() {
    let claims = build_claims("user-1", vec![], vec![], vec!["system", "Business"]);
    assert!(has_tier_access(&claims, "System"));
    assert!(has_tier_access(&claims, "SYSTEM"));
    assert!(has_tier_access(&claims, "business"));
    assert!(has_tier_access(&claims, "BUSINESS"));
}

#[test]
fn has_tier_access_returns_false_when_tier_access_is_none() {
    let claims = build_minimal_claims("user-1");
    assert!(!has_tier_access(&claims, "system"));
}

#[test]
fn has_tier_access_returns_false_for_empty_tier_list() {
    let claims = build_claims("user-1", vec![], vec![], vec![]);
    assert!(!has_tier_access(&claims, "system"));
}

// ============================================================
// Claims construction and accessor tests
// ============================================================

#[test]
fn claims_audience_returns_first_audience() {
    let claims = Claims {
        sub: "u".into(),
        iss: "i".into(),
        aud: k1s0_auth::Audience(vec!["aud1".into(), "aud2".into()]),
        exp: 0,
        iat: 0,
        jti: None,
        typ: None,
        azp: None,
        scope: None,
        preferred_username: None,
        email: None,
        realm_access: None,
        resource_access: None,
        tier_access: None,
    };
    assert_eq!(claims.audience(), Some("aud1"));
}

#[test]
fn claims_audience_returns_none_when_empty() {
    let claims = build_minimal_claims("u");
    assert!(claims.audience().is_none());
}

#[test]
fn claims_realm_roles_returns_empty_slice_when_none() {
    let claims = build_minimal_claims("u");
    assert!(claims.realm_roles().is_empty());
}

#[test]
fn claims_realm_roles_returns_roles() {
    let claims = build_claims("u", vec!["a", "b", "c"], vec![], vec![]);
    assert_eq!(claims.realm_roles(), &["a", "b", "c"]);
}

#[test]
fn claims_resource_roles_returns_empty_for_unknown_resource() {
    let claims = build_claims(
        "u",
        vec![],
        vec![("order-service", vec!["read"])],
        vec![],
    );
    assert!(claims.resource_roles("unknown-service").is_empty());
}

#[test]
fn claims_resource_roles_returns_roles_for_known_resource() {
    let claims = build_claims(
        "u",
        vec![],
        vec![("order-service", vec!["read", "write"])],
        vec![],
    );
    assert_eq!(claims.resource_roles("order-service"), &["read", "write"]);
}

#[test]
fn claims_tier_access_list_returns_empty_when_none() {
    let claims = build_minimal_claims("u");
    assert!(claims.tier_access_list().is_empty());
}

#[test]
fn claims_tier_access_list_returns_tiers() {
    let claims = build_claims("u", vec![], vec![], vec!["system", "business"]);
    assert_eq!(claims.tier_access_list(), &["system", "business"]);
}

#[test]
fn claims_display_contains_sub() {
    let claims = build_claims("user-xyz", vec![], vec![], vec![]);
    let display = format!("{}", claims);
    assert!(display.contains("user-xyz"));
}

// ============================================================
// actor / actor_from_claims tests
// ============================================================

#[test]
fn claims_actor_prefers_preferred_username() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = Some("taro".into());
    claims.email = Some("taro@example.com".into());
    assert_eq!(claims.actor(), Some("taro"));
}

#[test]
fn claims_actor_falls_back_to_email() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = None;
    claims.email = Some("taro@example.com".into());
    assert_eq!(claims.actor(), Some("taro@example.com"));
}

#[test]
fn claims_actor_falls_back_to_sub() {
    let claims = build_minimal_claims("sub-id");
    assert_eq!(claims.actor(), Some("sub-id"));
}

#[test]
fn claims_actor_skips_empty_preferred_username() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = Some("".into());
    claims.email = Some("user@test.com".into());
    assert_eq!(claims.actor(), Some("user@test.com"));
}

#[test]
fn claims_actor_skips_empty_email_and_username() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = Some("".into());
    claims.email = Some("".into());
    assert_eq!(claims.actor(), Some("sub-id"));
}

#[test]
fn claims_actor_returns_none_when_all_empty() {
    let mut claims = build_minimal_claims("");
    claims.preferred_username = Some("".into());
    claims.email = Some("".into());
    assert!(claims.actor().is_none());
}

#[test]
fn actor_from_claims_returns_actor_name() {
    let mut claims = build_minimal_claims("user-1");
    claims.preferred_username = Some("taro".into());
    assert_eq!(actor_from_claims(Some(&claims)), "taro");
}

#[test]
fn actor_from_claims_returns_system_when_none() {
    assert_eq!(actor_from_claims(None), "system");
}

#[test]
fn actor_from_claims_returns_system_when_all_fields_empty() {
    let mut claims = build_minimal_claims("");
    claims.preferred_username = Some("".into());
    claims.email = Some("".into());
    assert_eq!(actor_from_claims(Some(&claims)), "system");
}

// ============================================================
// Combined RBAC scenario tests
// ============================================================

#[test]
fn combined_scenario_regular_user_with_specific_permissions() {
    let claims = build_claims(
        "user-001",
        vec!["user", "order_manager"],
        vec![
            ("order-service", vec!["read", "write"]),
            ("config-service", vec!["read"]),
        ],
        vec!["system", "business"],
    );

    // Role checks
    assert!(has_role(&claims, "user"));
    assert!(has_role(&claims, "order_manager"));
    assert!(!has_role(&claims, "admin"));

    // Resource role checks
    assert!(has_resource_role(&claims, "order-service", "read"));
    assert!(has_resource_role(&claims, "order-service", "write"));
    assert!(!has_resource_role(&claims, "order-service", "delete"));
    assert!(has_resource_role(&claims, "config-service", "read"));
    assert!(!has_resource_role(&claims, "config-service", "write"));

    // Permission checks
    assert!(check_permission(&claims, "order-service", "read"));
    assert!(check_permission(&claims, "order-service", "write"));
    assert!(!check_permission(&claims, "order-service", "delete"));
    assert!(check_permission(&claims, "config-service", "read"));
    assert!(!check_permission(&claims, "payment-service", "read"));

    // Tier checks
    assert!(has_tier_access(&claims, "system"));
    assert!(has_tier_access(&claims, "business"));
    assert!(!has_tier_access(&claims, "service"));
}

#[test]
fn combined_scenario_sys_admin_has_full_access() {
    let claims = build_claims(
        "admin-001",
        vec!["sys_admin"],
        vec![],
        vec!["system", "business", "service"],
    );

    assert!(check_permission(&claims, "order-service", "read"));
    assert!(check_permission(&claims, "order-service", "delete"));
    assert!(check_permission(&claims, "any-service", "anything"));
    assert!(has_tier_access(&claims, "system"));
    assert!(has_tier_access(&claims, "business"));
    assert!(has_tier_access(&claims, "service"));
}

#[test]
fn combined_scenario_minimal_user_has_no_access() {
    let claims = build_minimal_claims("user-minimal");

    assert!(!has_role(&claims, "user"));
    assert!(!has_resource_role(&claims, "order-service", "read"));
    assert!(!check_permission(&claims, "order-service", "read"));
    assert!(!has_tier_access(&claims, "system"));
}

#[test]
fn multiple_resource_access_entries() {
    let claims = build_claims(
        "user-multi",
        vec!["user"],
        vec![
            ("service-a", vec!["read"]),
            ("service-b", vec!["write"]),
            ("service-c", vec!["admin"]),
        ],
        vec![],
    );

    assert!(check_permission(&claims, "service-a", "read"));
    assert!(!check_permission(&claims, "service-a", "write"));

    assert!(!check_permission(&claims, "service-b", "read"));
    assert!(check_permission(&claims, "service-b", "write"));

    // service-c has admin role, so all permissions granted
    assert!(check_permission(&claims, "service-c", "read"));
    assert!(check_permission(&claims, "service-c", "write"));
    assert!(check_permission(&claims, "service-c", "delete"));
}
