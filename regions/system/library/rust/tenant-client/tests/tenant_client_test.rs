use std::collections::HashMap;

use chrono::Utc;
use k1s0_tenant_client::{
    CreateTenantRequest, InMemoryTenantClient, ProvisioningStatus, Tenant, TenantClient,
    TenantClientConfig, TenantError, TenantFilter, TenantSettings, TenantStatus,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_tenant(id: &str, status: TenantStatus, plan: &str) -> Tenant {
    let mut settings = HashMap::new();
    settings.insert("max_users".to_string(), "100".to_string());
    settings.insert("feature_x".to_string(), "enabled".to_string());
    Tenant {
        id: id.to_string(),
        name: format!("Tenant {}", id),
        status,
        plan: plan.to_string(),
        settings,
        created_at: Utc::now(),
    }
}

fn sample_tenants() -> Vec<Tenant> {
    vec![
        make_tenant("T-001", TenantStatus::Active, "enterprise"),
        make_tenant("T-002", TenantStatus::Suspended, "basic"),
        make_tenant("T-003", TenantStatus::Active, "basic"),
        make_tenant("T-004", TenantStatus::Deleted, "enterprise"),
        make_tenant("T-005", TenantStatus::Active, "pro"),
    ]
}

// ===========================================================================
// CRUD: get_tenant
// ===========================================================================

#[tokio::test]
async fn get_tenant_returns_existing_tenant() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let tenant = client.get_tenant("T-001").await.unwrap();
    assert_eq!(tenant.id, "T-001");
    assert_eq!(tenant.name, "Tenant T-001");
    assert_eq!(tenant.status, TenantStatus::Active);
    assert_eq!(tenant.plan, "enterprise");
}

#[tokio::test]
async fn get_tenant_not_found_returns_error() {
    let client = InMemoryTenantClient::new();
    let result = client.get_tenant("nonexistent").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        TenantError::NotFound(id) => assert_eq!(id, "nonexistent"),
        other => panic!("expected NotFound, got: {:?}", other),
    }
}

#[tokio::test]
async fn get_tenant_after_add_tenant() {
    let client = InMemoryTenantClient::new();
    client.add_tenant(make_tenant("T-100", TenantStatus::Active, "pro"));
    let tenant = client.get_tenant("T-100").await.unwrap();
    assert_eq!(tenant.id, "T-100");
    assert_eq!(tenant.plan, "pro");
}

// ===========================================================================
// CRUD: create_tenant
// ===========================================================================

#[tokio::test]
async fn create_tenant_assigns_uuid_and_active_status() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "Acme Corp".to_string(),
            plan: "enterprise".to_string(),
            admin_user_id: Some("admin-1".to_string()),
        })
        .await
        .unwrap();

    assert!(!tenant.id.is_empty());
    assert_eq!(tenant.name, "Acme Corp");
    assert_eq!(tenant.plan, "enterprise");
    assert_eq!(tenant.status, TenantStatus::Active);
}

#[tokio::test]
async fn create_tenant_is_retrievable() {
    let client = InMemoryTenantClient::new();
    let created = client
        .create_tenant(CreateTenantRequest {
            name: "Test Org".to_string(),
            plan: "basic".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    let fetched = client.get_tenant(&created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Test Org");
}

#[tokio::test]
async fn create_tenant_sets_provisioning_pending() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "Prov Tenant".to_string(),
            plan: "pro".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    let status = client.get_provisioning_status(&tenant.id).await.unwrap();
    assert_eq!(status, ProvisioningStatus::Pending);
}

#[tokio::test]
async fn create_multiple_tenants_have_unique_ids() {
    let client = InMemoryTenantClient::new();
    let t1 = client
        .create_tenant(CreateTenantRequest {
            name: "Org A".to_string(),
            plan: "basic".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();
    let t2 = client
        .create_tenant(CreateTenantRequest {
            name: "Org B".to_string(),
            plan: "basic".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    assert_ne!(t1.id, t2.id);
}

// ===========================================================================
// Filtering: list_tenants
// ===========================================================================

#[tokio::test]
async fn list_tenants_no_filter_returns_all() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let tenants = client.list_tenants(TenantFilter::new()).await.unwrap();
    assert_eq!(tenants.len(), 5);
}

#[tokio::test]
async fn list_tenants_filter_by_status_active() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let filter = TenantFilter::new().status(TenantStatus::Active);
    let tenants = client.list_tenants(filter).await.unwrap();
    assert_eq!(tenants.len(), 3);
    assert!(tenants.iter().all(|t| t.status == TenantStatus::Active));
}

#[tokio::test]
async fn list_tenants_filter_by_status_suspended() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let filter = TenantFilter::new().status(TenantStatus::Suspended);
    let tenants = client.list_tenants(filter).await.unwrap();
    assert_eq!(tenants.len(), 1);
    assert_eq!(tenants[0].id, "T-002");
}

#[tokio::test]
async fn list_tenants_filter_by_status_deleted() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let filter = TenantFilter::new().status(TenantStatus::Deleted);
    let tenants = client.list_tenants(filter).await.unwrap();
    assert_eq!(tenants.len(), 1);
    assert_eq!(tenants[0].id, "T-004");
}

#[tokio::test]
async fn list_tenants_filter_by_plan() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let filter = TenantFilter::new().plan("enterprise");
    let tenants = client.list_tenants(filter).await.unwrap();
    assert_eq!(tenants.len(), 2);
}

#[tokio::test]
async fn list_tenants_filter_by_status_and_plan() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let filter = TenantFilter::new()
        .status(TenantStatus::Active)
        .plan("basic");
    let tenants = client.list_tenants(filter).await.unwrap();
    assert_eq!(tenants.len(), 1);
    assert_eq!(tenants[0].id, "T-003");
}

#[tokio::test]
async fn list_tenants_filter_no_match_returns_empty() {
    let client = InMemoryTenantClient::with_tenants(sample_tenants());
    let filter = TenantFilter::new().plan("nonexistent-plan");
    let tenants = client.list_tenants(filter).await.unwrap();
    assert!(tenants.is_empty());
}

#[tokio::test]
async fn list_tenants_empty_store_returns_empty() {
    let client = InMemoryTenantClient::new();
    let tenants = client.list_tenants(TenantFilter::new()).await.unwrap();
    assert!(tenants.is_empty());
}

// ===========================================================================
// is_active
// ===========================================================================

#[tokio::test]
async fn is_active_returns_true_for_active_tenant() {
    let client = InMemoryTenantClient::new();
    client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
    assert!(client.is_active("T-001").await.unwrap());
}

#[tokio::test]
async fn is_active_returns_false_for_suspended_tenant() {
    let client = InMemoryTenantClient::new();
    client.add_tenant(make_tenant("T-001", TenantStatus::Suspended, "basic"));
    assert!(!client.is_active("T-001").await.unwrap());
}

#[tokio::test]
async fn is_active_returns_false_for_deleted_tenant() {
    let client = InMemoryTenantClient::new();
    client.add_tenant(make_tenant("T-001", TenantStatus::Deleted, "basic"));
    assert!(!client.is_active("T-001").await.unwrap());
}

#[tokio::test]
async fn is_active_not_found_returns_error() {
    let client = InMemoryTenantClient::new();
    let result = client.is_active("missing").await;
    assert!(matches!(result, Err(TenantError::NotFound(_))));
}

// ===========================================================================
// Settings
// ===========================================================================

#[tokio::test]
async fn get_settings_returns_tenant_settings() {
    let client = InMemoryTenantClient::new();
    client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
    let settings = client.get_settings("T-001").await.unwrap();
    assert_eq!(settings.get("max_users"), Some("100"));
    assert_eq!(settings.get("feature_x"), Some("enabled"));
}

#[tokio::test]
async fn get_settings_missing_key_returns_none() {
    let client = InMemoryTenantClient::new();
    client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
    let settings = client.get_settings("T-001").await.unwrap();
    assert_eq!(settings.get("nonexistent_key"), None);
}

#[tokio::test]
async fn get_settings_not_found_tenant_returns_error() {
    let client = InMemoryTenantClient::new();
    let result = client.get_settings("missing").await;
    assert!(matches!(result, Err(TenantError::NotFound(_))));
}

// ===========================================================================
// Member management
// ===========================================================================

#[tokio::test]
async fn add_member_to_existing_tenant() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "Members Test".to_string(),
            plan: "pro".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    let member = client
        .add_member(&tenant.id, "user-1", "admin")
        .await
        .unwrap();
    assert_eq!(member.user_id, "user-1");
    assert_eq!(member.role, "admin");
}

#[tokio::test]
async fn add_member_to_nonexistent_tenant_returns_error() {
    let client = InMemoryTenantClient::new();
    let result = client.add_member("missing-tenant", "user-1", "admin").await;
    assert!(matches!(result, Err(TenantError::NotFound(_))));
}

#[tokio::test]
async fn list_members_returns_added_members() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "Multi Members".to_string(),
            plan: "enterprise".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    client.add_member(&tenant.id, "u1", "admin").await.unwrap();
    client.add_member(&tenant.id, "u2", "member").await.unwrap();
    client.add_member(&tenant.id, "u3", "viewer").await.unwrap();

    let members = client.list_members(&tenant.id).await.unwrap();
    assert_eq!(members.len(), 3);

    let user_ids: Vec<&str> = members.iter().map(|m| m.user_id.as_str()).collect();
    assert!(user_ids.contains(&"u1"));
    assert!(user_ids.contains(&"u2"));
    assert!(user_ids.contains(&"u3"));
}

#[tokio::test]
async fn list_members_empty_when_no_members() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "Empty Members".to_string(),
            plan: "basic".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    let members = client.list_members(&tenant.id).await.unwrap();
    assert!(members.is_empty());
}

#[tokio::test]
async fn remove_member_removes_correct_user() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "Remove Test".to_string(),
            plan: "pro".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    client.add_member(&tenant.id, "u1", "admin").await.unwrap();
    client.add_member(&tenant.id, "u2", "member").await.unwrap();
    client.add_member(&tenant.id, "u3", "viewer").await.unwrap();

    client.remove_member(&tenant.id, "u2").await.unwrap();

    let members = client.list_members(&tenant.id).await.unwrap();
    assert_eq!(members.len(), 2);
    let user_ids: Vec<&str> = members.iter().map(|m| m.user_id.as_str()).collect();
    assert!(!user_ids.contains(&"u2"));
    assert!(user_ids.contains(&"u1"));
    assert!(user_ids.contains(&"u3"));
}

#[tokio::test]
async fn remove_nonexistent_member_is_no_op() {
    let client = InMemoryTenantClient::new();
    let tenant = client
        .create_tenant(CreateTenantRequest {
            name: "No-op Remove".to_string(),
            plan: "basic".to_string(),
            admin_user_id: None,
        })
        .await
        .unwrap();

    // Should not panic or return error
    let result = client.remove_member(&tenant.id, "ghost-user").await;
    assert!(result.is_ok());
}

// ===========================================================================
// Provisioning status
// ===========================================================================

#[tokio::test]
async fn provisioning_status_not_found_for_uncreated_tenant() {
    let client = InMemoryTenantClient::new();
    // add_tenant does not set provisioning status
    client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
    let result = client.get_provisioning_status("T-001").await;
    assert!(matches!(result, Err(TenantError::NotFound(_))));
}

// ===========================================================================
// TenantFilter builder
// ===========================================================================

#[test]
fn tenant_filter_default_has_no_filters() {
    let filter = TenantFilter::new();
    assert!(filter.status.is_none());
    assert!(filter.plan.is_none());
}

#[test]
fn tenant_filter_chained_builder() {
    let filter = TenantFilter::new()
        .status(TenantStatus::Active)
        .plan("enterprise");
    assert_eq!(filter.status, Some(TenantStatus::Active));
    assert_eq!(filter.plan, Some("enterprise".to_string()));
}

// ===========================================================================
// TenantSettings
// ===========================================================================

#[test]
fn tenant_settings_new_and_get() {
    let mut values = HashMap::new();
    values.insert("key1".to_string(), "val1".to_string());
    values.insert("key2".to_string(), "val2".to_string());
    let settings = TenantSettings::new(values);
    assert_eq!(settings.get("key1"), Some("val1"));
    assert_eq!(settings.get("key2"), Some("val2"));
    assert_eq!(settings.get("key3"), None);
}

#[test]
fn tenant_settings_empty() {
    let settings = TenantSettings::new(HashMap::new());
    assert_eq!(settings.get("anything"), None);
}

// ===========================================================================
// TenantClientConfig builder
// ===========================================================================

#[test]
fn config_defaults() {
    let config = TenantClientConfig::new("http://localhost:9090");
    assert_eq!(config.server_url, "http://localhost:9090");
    assert_eq!(config.cache_ttl, std::time::Duration::from_secs(300));
    assert_eq!(config.cache_max_capacity, 1000);
}

#[test]
fn config_custom_values() {
    let config = TenantClientConfig::new("http://tenant-server:8080")
        .cache_ttl(std::time::Duration::from_secs(60))
        .cache_max_capacity(500);
    assert_eq!(config.server_url, "http://tenant-server:8080");
    assert_eq!(config.cache_ttl, std::time::Duration::from_secs(60));
    assert_eq!(config.cache_max_capacity, 500);
}

// ===========================================================================
// TenantStatus serde roundtrip
// ===========================================================================

#[test]
fn tenant_status_serde_roundtrip() {
    for status in [
        TenantStatus::Active,
        TenantStatus::Suspended,
        TenantStatus::Deleted,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: TenantStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }
}

#[test]
fn tenant_status_serializes_lowercase() {
    assert_eq!(serde_json::to_string(&TenantStatus::Active).unwrap(), "\"active\"");
    assert_eq!(serde_json::to_string(&TenantStatus::Suspended).unwrap(), "\"suspended\"");
    assert_eq!(serde_json::to_string(&TenantStatus::Deleted).unwrap(), "\"deleted\"");
}

// ===========================================================================
// Tenant serde
// ===========================================================================

#[test]
fn tenant_serde_roundtrip() {
    let tenant = make_tenant("T-001", TenantStatus::Active, "enterprise");
    let json = serde_json::to_string(&tenant).unwrap();
    let deserialized: Tenant = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, tenant.id);
    assert_eq!(deserialized.name, tenant.name);
    assert_eq!(deserialized.status, tenant.status);
    assert_eq!(deserialized.plan, tenant.plan);
}

// ===========================================================================
// InMemoryTenantClient: with_tenants constructor
// ===========================================================================

#[tokio::test]
async fn with_tenants_constructor_populates_store() {
    let tenants = vec![
        make_tenant("A", TenantStatus::Active, "basic"),
        make_tenant("B", TenantStatus::Suspended, "pro"),
    ];
    let client = InMemoryTenantClient::with_tenants(tenants);

    let a = client.get_tenant("A").await.unwrap();
    assert_eq!(a.status, TenantStatus::Active);

    let b = client.get_tenant("B").await.unwrap();
    assert_eq!(b.status, TenantStatus::Suspended);
}

// ===========================================================================
// Error display
// ===========================================================================

#[test]
fn error_display_not_found() {
    let err = TenantError::NotFound("T-999".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("T-999"));
}

#[test]
fn error_display_server_error() {
    let err = TenantError::ServerError("connection refused".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("connection refused"));
}

#[test]
fn error_display_timeout() {
    let err = TenantError::Timeout("5s elapsed".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("5s elapsed"));
}

#[test]
fn error_display_suspended() {
    let err = TenantError::Suspended("T-001".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("T-001"));
}
