use k1s0_vault_client::{
    InMemoryVaultClient, Secret, SecretRotatedEvent, VaultClient, VaultClientConfig, VaultError,
};
use std::collections::HashMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_config() -> VaultClientConfig {
    VaultClientConfig::new("http://vault.test:8200")
}

fn make_secret(path: &str) -> Secret {
    let mut data = HashMap::new();
    data.insert("password".to_string(), "hunter2".to_string());
    data.insert("username".to_string(), "root".to_string());
    Secret {
        path: path.to_string(),
        data,
        version: 1,
        created_at: chrono::Utc::now(),
    }
}

fn make_secret_with_data(path: &str, pairs: &[(&str, &str)]) -> Secret {
    let data = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Secret {
        path: path.to_string(),
        data,
        version: 1,
        created_at: chrono::Utc::now(),
    }
}

// ===========================================================================
// VaultClientConfig tests
// ===========================================================================

#[test]
fn config_new_sets_server_url() {
    let cfg = VaultClientConfig::new("http://vault:8200");
    assert_eq!(cfg.server_url, "http://vault:8200");
}

#[test]
fn config_default_cache_ttl() {
    let cfg = VaultClientConfig::new("http://vault:8200");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(600));
}

#[test]
fn config_default_cache_max_capacity() {
    let cfg = VaultClientConfig::new("http://vault:8200");
    assert_eq!(cfg.cache_max_capacity, 500);
}

#[test]
fn config_custom_cache_ttl() {
    let cfg = VaultClientConfig::new("http://vault:8200").cache_ttl(Duration::from_secs(120));
    assert_eq!(cfg.cache_ttl, Duration::from_secs(120));
}

#[test]
fn config_custom_cache_max_capacity() {
    let cfg = VaultClientConfig::new("http://vault:8200").cache_max_capacity(200);
    assert_eq!(cfg.cache_max_capacity, 200);
}

#[test]
fn config_builder_chain() {
    let cfg = VaultClientConfig::new("http://vault:8200")
        .cache_ttl(Duration::from_secs(60))
        .cache_max_capacity(50);
    assert_eq!(cfg.server_url, "http://vault:8200");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(60));
    assert_eq!(cfg.cache_max_capacity, 50);
}

#[test]
fn config_default_impl() {
    let cfg = VaultClientConfig::default();
    assert_eq!(cfg.server_url, "http://localhost:8080");
}

// ===========================================================================
// Secret & SecretRotatedEvent model tests
// ===========================================================================

#[test]
fn secret_fields_accessible() {
    let secret = make_secret("system/db");
    assert_eq!(secret.path, "system/db");
    assert_eq!(secret.version, 1);
    assert!(secret.data.contains_key("password"));
    assert!(secret.data.contains_key("username"));
}

#[test]
fn secret_clone() {
    let original = make_secret("system/db");
    let cloned = original.clone();
    assert_eq!(original.path, cloned.path);
    assert_eq!(original.version, cloned.version);
    assert_eq!(original.data, cloned.data);
}

#[test]
fn secret_rotated_event_fields() {
    let event = SecretRotatedEvent {
        path: "system/api-key".to_string(),
        version: 5,
    };
    assert_eq!(event.path, "system/api-key");
    assert_eq!(event.version, 5);
}

#[test]
fn secret_rotated_event_clone() {
    let event = SecretRotatedEvent {
        path: "system/db".to_string(),
        version: 3,
    };
    let cloned = event.clone();
    assert_eq!(event.path, cloned.path);
    assert_eq!(event.version, cloned.version);
}

// ===========================================================================
// VaultError tests
// ===========================================================================

#[test]
fn error_not_found_variant() {
    let err = VaultError::NotFound("secret/missing".to_string());
    assert!(matches!(err, VaultError::NotFound(ref p) if p == "secret/missing"));
    assert!(err.to_string().contains("secret/missing"));
}

#[test]
fn error_permission_denied_variant() {
    let err = VaultError::PermissionDenied("system/restricted".to_string());
    assert!(matches!(err, VaultError::PermissionDenied(_)));
    assert!(err.to_string().contains("system/restricted"));
}

#[test]
fn error_server_error_variant() {
    let err = VaultError::ServerError("internal failure".to_string());
    assert!(matches!(err, VaultError::ServerError(_)));
}

#[test]
fn error_timeout_variant() {
    let err = VaultError::Timeout;
    assert!(matches!(err, VaultError::Timeout));
}

#[test]
fn error_lease_expired_variant() {
    let err = VaultError::LeaseExpired("system/db".to_string());
    assert!(matches!(err, VaultError::LeaseExpired(_)));
    assert!(err.to_string().contains("system/db"));
}

// ===========================================================================
// InMemoryVaultClient — put / get / get_value / list / watch
// ===========================================================================

#[tokio::test]
async fn inmemory_get_secret_success() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret("system/db/primary"));

    let secret = client.get_secret("system/db/primary").await.unwrap();
    assert_eq!(secret.path, "system/db/primary");
    assert_eq!(secret.data.get("password").unwrap(), "hunter2");
}

#[tokio::test]
async fn inmemory_get_secret_not_found() {
    let client = InMemoryVaultClient::with_config(make_config());
    let err = client.get_secret("does/not/exist").await.unwrap_err();
    assert!(matches!(err, VaultError::NotFound(_)));
}

#[tokio::test]
async fn inmemory_get_secret_value_success() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret("system/db"));

    let val = client
        .get_secret_value("system/db", "username")
        .await
        .unwrap();
    assert_eq!(val, "root");
}

#[tokio::test]
async fn inmemory_get_secret_value_key_not_found() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret("system/db"));

    let err = client
        .get_secret_value("system/db", "nonexistent")
        .await
        .unwrap_err();
    assert!(matches!(err, VaultError::NotFound(_)));
}

#[tokio::test]
async fn inmemory_get_secret_value_path_not_found() {
    let client = InMemoryVaultClient::with_config(make_config());
    let err = client
        .get_secret_value("missing/path", "key")
        .await
        .unwrap_err();
    assert!(matches!(err, VaultError::NotFound(_)));
}

#[tokio::test]
async fn inmemory_list_secrets_with_prefix() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret("system/db/primary"));
    client.put_secret(make_secret("system/db/replica"));
    client.put_secret(make_secret("business/cache/redis"));

    let paths = client.list_secrets("system/db/").await.unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().all(|p| p.starts_with("system/db/")));
}

#[tokio::test]
async fn inmemory_list_secrets_empty_result() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret("system/db"));

    let paths = client.list_secrets("nothing/").await.unwrap();
    assert!(paths.is_empty());
}

#[tokio::test]
async fn inmemory_list_secrets_all() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret("a/1"));
    client.put_secret(make_secret("b/2"));
    client.put_secret(make_secret("c/3"));

    // empty prefix matches everything
    let paths = client.list_secrets("").await.unwrap();
    assert_eq!(paths.len(), 3);
}

#[tokio::test]
async fn inmemory_put_overwrites_existing_secret() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret_with_data("system/db", &[("password", "old")]));
    client.put_secret(make_secret_with_data("system/db", &[("password", "new")]));

    let secret = client.get_secret("system/db").await.unwrap();
    assert_eq!(secret.data.get("password").unwrap(), "new");
}

#[tokio::test]
async fn inmemory_watch_returns_receiver() {
    let client = InMemoryVaultClient::with_config(make_config());
    let rx = client.watch_secret("system/db").await.unwrap();
    // receiver should be valid (not yet closed by sender drop, but will close momentarily)
    drop(rx);
}

// ===========================================================================
// InMemoryVaultClient — constructor variants
// ===========================================================================

#[test]
fn inmemory_new_uses_defaults() {
    let client = InMemoryVaultClient::new();
    let cfg = client.config();
    assert_eq!(cfg.server_url, "http://localhost:8080");
    assert_eq!(cfg.cache_ttl, Duration::from_secs(600));
}

#[test]
fn inmemory_with_config_stores_config() {
    let config = VaultClientConfig::new("http://custom:9090").cache_max_capacity(10);
    let client = InMemoryVaultClient::with_config(config);
    assert_eq!(client.config().server_url, "http://custom:9090");
    assert_eq!(client.config().cache_max_capacity, 10);
}

// ===========================================================================
// Multiple secrets interaction
// ===========================================================================

#[tokio::test]
async fn inmemory_multiple_secrets_independent() {
    let client = InMemoryVaultClient::with_config(make_config());
    client.put_secret(make_secret_with_data("a", &[("key", "val_a")]));
    client.put_secret(make_secret_with_data("b", &[("key", "val_b")]));

    let a = client.get_secret_value("a", "key").await.unwrap();
    let b = client.get_secret_value("b", "key").await.unwrap();
    assert_eq!(a, "val_a");
    assert_eq!(b, "val_b");
}

#[tokio::test]
async fn inmemory_secret_with_many_keys() {
    let client = InMemoryVaultClient::with_config(make_config());
    let pairs = [
        ("host", "db.internal"),
        ("port", "5432"),
        ("user", "admin"),
        ("password", "s3cret"),
        ("dbname", "mydb"),
    ];
    client.put_secret(make_secret_with_data("system/postgres", &pairs));

    for (k, v) in &pairs {
        let val = client
            .get_secret_value("system/postgres", k)
            .await
            .unwrap();
        assert_eq!(val, *v);
    }
}
