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

// 存在するロールに対して has_role が true を返すことを確認する。
#[test]
fn has_role_returns_true_for_existing_role() {
    let claims = build_claims("user-1", vec!["user", "editor"], vec![], vec![]);
    assert!(has_role(&claims, "user"));
    assert!(has_role(&claims, "editor"));
}

// 存在しないロールに対して has_role が false を返すことを確認する。
#[test]
fn has_role_returns_false_for_missing_role() {
    let claims = build_claims("user-1", vec!["user"], vec![], vec![]);
    assert!(!has_role(&claims, "admin"));
    assert!(!has_role(&claims, "sys_admin"));
}

// realm_access が None の場合に has_role が false を返すことを確認する。
#[test]
fn has_role_returns_false_when_realm_access_is_none() {
    let claims = build_minimal_claims("user-1");
    assert!(!has_role(&claims, "user"));
}

// ロールリストが空の場合に has_role が false を返すことを確認する。
#[test]
fn has_role_with_empty_roles() {
    let claims = build_claims("user-1", vec![], vec![], vec![]);
    assert!(!has_role(&claims, "user"));
}

// ============================================================
// has_resource_role tests
// ============================================================

// 一致するリソースとロールに対して has_resource_role が true を返すことを確認する。
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

// 存在しないロールに対して has_resource_role が false を返すことを確認する。
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

// 存在しないリソースに対して has_resource_role が false を返すことを確認する。
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

// resource_access が None の場合に has_resource_role が false を返すことを確認する。
#[test]
fn has_resource_role_returns_false_when_resource_access_is_none() {
    let claims = build_minimal_claims("user-1");
    assert!(!has_resource_role(&claims, "order-service", "read"));
}

// ============================================================
// check_permission tests
// ============================================================

// リソースロールに基づいて check_permission がアクセスを許可することを確認する。
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

// sys_admin ロールを持つユーザーはすべてのリソースへのアクセスが許可されることを確認する。
#[test]
fn check_permission_sys_admin_grants_all() {
    let claims = build_claims("admin-1", vec!["sys_admin"], vec![], vec![]);
    assert!(check_permission(&claims, "any-resource", "read"));
    assert!(check_permission(&claims, "any-resource", "write"));
    assert!(check_permission(&claims, "any-resource", "delete"));
    assert!(check_permission(&claims, "another-resource", "admin"));
}

// realm の admin ロールを持つユーザーはすべてのリソースへのアクセスが許可されることを確認する。
#[test]
fn check_permission_realm_admin_grants_all() {
    let claims = build_claims("admin-2", vec!["admin"], vec![], vec![]);
    assert!(check_permission(&claims, "order-service", "read"));
    assert!(check_permission(&claims, "user-service", "write"));
}

// リソース admin ロールを持つ場合はそのリソースへのすべての操作が許可されることを確認する。
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

// 一致するロールがない場合に check_permission がアクセスを拒否することを確認する。
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

// ロールが一切ない場合に check_permission がアクセスを拒否することを確認する。
#[test]
fn check_permission_denies_with_no_roles_at_all() {
    let claims = build_minimal_claims("user-1");
    assert!(!check_permission(&claims, "order-service", "read"));
}

// ============================================================
// has_permission tests (alias)
// ============================================================

// has_permission が check_permission の別名として同じ結果を返すことを確認する。
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

// 許可された Tier に対して has_tier_access が true を返すことを確認する。
#[test]
fn has_tier_access_returns_true_for_allowed_tier() {
    let claims = build_claims("user-1", vec![], vec![], vec!["system", "business"]);
    assert!(has_tier_access(&claims, "system"));
    assert!(has_tier_access(&claims, "business"));
}

// 階層モデル: system tier は全 tier にアクセス可能であることを確認する。
// service tier のみ持つユーザーは上位 tier（system, business）にアクセスできない。
#[test]
fn has_tier_access_respects_hierarchy() {
    // system は最上位 → 全 tier にアクセス可能
    let system_claims = build_claims("user-1", vec![], vec![], vec!["system"]);
    assert!(has_tier_access(&system_claims, "system"));
    assert!(has_tier_access(&system_claims, "business"));
    assert!(has_tier_access(&system_claims, "service"));

    // service のみ → 上位 tier にはアクセス不可
    let service_claims = build_claims("user-2", vec![], vec![], vec!["service"]);
    assert!(!has_tier_access(&service_claims, "system"));
    assert!(!has_tier_access(&service_claims, "business"));
    assert!(has_tier_access(&service_claims, "service"));
}

// has_tier_access が大文字小文字を区別せずに Tier を比較することを確認する。
#[test]
fn has_tier_access_is_case_insensitive() {
    let claims = build_claims("user-1", vec![], vec![], vec!["system", "Business"]);
    assert!(has_tier_access(&claims, "System"));
    assert!(has_tier_access(&claims, "SYSTEM"));
    assert!(has_tier_access(&claims, "business"));
    assert!(has_tier_access(&claims, "BUSINESS"));
}

// tier_access が None の場合に has_tier_access が false を返すことを確認する。
#[test]
fn has_tier_access_returns_false_when_tier_access_is_none() {
    let claims = build_minimal_claims("user-1");
    assert!(!has_tier_access(&claims, "system"));
}

// tier_access リストが空の場合に has_tier_access が false を返すことを確認する。
#[test]
fn has_tier_access_returns_false_for_empty_tier_list() {
    let claims = build_claims("user-1", vec![], vec![], vec![]);
    assert!(!has_tier_access(&claims, "system"));
}

// ============================================================
// Claims construction and accessor tests
// ============================================================

// Claims の audience() が複数オーディエンスの最初の要素を返すことを確認する。
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

// オーディエンスリストが空の場合に Claims の audience() が None を返すことを確認する。
#[test]
fn claims_audience_returns_none_when_empty() {
    let claims = build_minimal_claims("u");
    assert!(claims.audience().is_none());
}

// realm_access が None の場合に realm_roles() が空スライスを返すことを確認する。
#[test]
fn claims_realm_roles_returns_empty_slice_when_none() {
    let claims = build_minimal_claims("u");
    assert!(claims.realm_roles().is_empty());
}

// Claims の realm_roles() が設定されたロール一覧を返すことを確認する。
#[test]
fn claims_realm_roles_returns_roles() {
    let claims = build_claims("u", vec!["a", "b", "c"], vec![], vec![]);
    assert_eq!(claims.realm_roles(), &["a", "b", "c"]);
}

// 未登録リソースに対して resource_roles() が空スライスを返すことを確認する。
#[test]
fn claims_resource_roles_returns_empty_for_unknown_resource() {
    let claims = build_claims("u", vec![], vec![("order-service", vec!["read"])], vec![]);
    assert!(claims.resource_roles("unknown-service").is_empty());
}

// 登録済みリソースに対して resource_roles() がそのロール一覧を返すことを確認する。
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

// tier_access が None の場合に tier_access_list() が空スライスを返すことを確認する。
#[test]
fn claims_tier_access_list_returns_empty_when_none() {
    let claims = build_minimal_claims("u");
    assert!(claims.tier_access_list().is_empty());
}

// Claims の tier_access_list() が設定された Tier 一覧を返すことを確認する。
#[test]
fn claims_tier_access_list_returns_tiers() {
    let claims = build_claims("u", vec![], vec![], vec!["system", "business"]);
    assert_eq!(claims.tier_access_list(), &["system", "business"]);
}

// Claims の Display 出力に sub フィールドの値が含まれることを確認する。
#[test]
fn claims_display_contains_sub() {
    let claims = build_claims("user-xyz", vec![], vec![], vec![]);
    let display = format!("{}", claims);
    assert!(display.contains("user-xyz"));
}

// ============================================================
// actor / actor_from_claims tests
// ============================================================

// preferred_username が設定されている場合に actor() がそれを返すことを確認する。
#[test]
fn claims_actor_prefers_preferred_username() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = Some("taro".into());
    claims.email = Some("taro@example.com".into());
    assert_eq!(claims.actor(), Some("taro"));
}

// preferred_username が None の場合に actor() がメールアドレスにフォールバックすることを確認する。
#[test]
fn claims_actor_falls_back_to_email() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = None;
    claims.email = Some("taro@example.com".into());
    assert_eq!(claims.actor(), Some("taro@example.com"));
}

// preferred_username と email が共に None の場合に actor() が sub にフォールバックすることを確認する。
#[test]
fn claims_actor_falls_back_to_sub() {
    let claims = build_minimal_claims("sub-id");
    assert_eq!(claims.actor(), Some("sub-id"));
}

// preferred_username が空文字列の場合に actor() がメールアドレスにフォールバックすることを確認する。
#[test]
fn claims_actor_skips_empty_preferred_username() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = Some("".into());
    claims.email = Some("user@test.com".into());
    assert_eq!(claims.actor(), Some("user@test.com"));
}

// username と email が共に空文字列の場合に actor() が sub を返すことを確認する。
#[test]
fn claims_actor_skips_empty_email_and_username() {
    let mut claims = build_minimal_claims("sub-id");
    claims.preferred_username = Some("".into());
    claims.email = Some("".into());
    assert_eq!(claims.actor(), Some("sub-id"));
}

// すべてのフィールドが空の場合に actor() が None を返すことを確認する。
#[test]
fn claims_actor_returns_none_when_all_empty() {
    let mut claims = build_minimal_claims("");
    claims.preferred_username = Some("".into());
    claims.email = Some("".into());
    assert!(claims.actor().is_none());
}

// actor_from_claims が Claims のアクター名を返すことを確認する。
#[test]
fn actor_from_claims_returns_actor_name() {
    let mut claims = build_minimal_claims("user-1");
    claims.preferred_username = Some("taro".into());
    assert_eq!(actor_from_claims(Some(&claims)), "taro");
}

// Claims が None の場合に actor_from_claims が "system" を返すことを確認する。
#[test]
fn actor_from_claims_returns_system_when_none() {
    assert_eq!(actor_from_claims(None), "system");
}

// すべてのフィールドが空の Claims では actor_from_claims が "system" を返すことを確認する。
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

// 一般ユーザーが特定のロールとリソースアクセス権を持つ場合の総合シナリオを確認する。
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

    // Tier checks: business tier は business + service にアクセス可能（階層モデル）
    assert!(has_tier_access(&claims, "system"));
    assert!(has_tier_access(&claims, "business"));
    assert!(has_tier_access(&claims, "service"));
}

// sys_admin ユーザーがすべてのリソースと Tier に完全アクセスできることを確認する。
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

// ロールを一切持たない最小構成ユーザーがすべてのアクセスを拒否されることを確認する。
#[test]
fn combined_scenario_minimal_user_has_no_access() {
    let claims = build_minimal_claims("user-minimal");

    assert!(!has_role(&claims, "user"));
    assert!(!has_resource_role(&claims, "order-service", "read"));
    assert!(!check_permission(&claims, "order-service", "read"));
    assert!(!has_tier_access(&claims, "system"));
}

// 複数のリソースアクセスエントリが個別に正しく評価されることを確認する。
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
