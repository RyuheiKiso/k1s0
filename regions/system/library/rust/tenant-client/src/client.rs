use async_trait::async_trait;

use crate::config::TenantClientConfig;
use crate::error::TenantError;
use crate::tenant::{Tenant, TenantFilter, TenantSettings, TenantStatus};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait TenantClient: Send + Sync {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError>;
    async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError>;
    async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError>;
    async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError>;
}

pub struct InMemoryTenantClient {
    tenants: std::sync::RwLock<Vec<Tenant>>,
}

impl InMemoryTenantClient {
    pub fn new() -> Self {
        Self {
            tenants: std::sync::RwLock::new(Vec::new()),
        }
    }

    pub fn with_tenants(tenants: Vec<Tenant>) -> Self {
        Self {
            tenants: std::sync::RwLock::new(tenants),
        }
    }

    pub fn add_tenant(&self, tenant: Tenant) {
        self.tenants.write().unwrap().push(tenant);
    }
}

impl Default for InMemoryTenantClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantClient for InMemoryTenantClient {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError> {
        let tenants = self.tenants.read().unwrap();
        tenants
            .iter()
            .find(|t| t.id == tenant_id)
            .cloned()
            .ok_or_else(|| TenantError::NotFound(tenant_id.to_string()))
    }

    async fn list_tenants(&self, filter: TenantFilter) -> Result<Vec<Tenant>, TenantError> {
        let tenants = self.tenants.read().unwrap();
        let result: Vec<Tenant> = tenants
            .iter()
            .filter(|t| {
                if let Some(status) = &filter.status {
                    if t.status != *status {
                        return false;
                    }
                }
                if let Some(plan) = &filter.plan {
                    if t.plan != *plan {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        Ok(result)
    }

    async fn is_active(&self, tenant_id: &str) -> Result<bool, TenantError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(tenant.status == TenantStatus::Active)
    }

    async fn get_settings(&self, tenant_id: &str) -> Result<TenantSettings, TenantError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(TenantSettings::new(tenant.settings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_tenant(id: &str, status: TenantStatus, plan: &str) -> Tenant {
        let mut settings = HashMap::new();
        settings.insert("max_users".to_string(), "100".to_string());
        Tenant {
            id: id.to_string(),
            name: format!("Tenant {}", id),
            status,
            plan: plan.to_string(),
            settings,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_tenant() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Active, "enterprise"));
        let tenant = client.get_tenant("T-001").await.unwrap();
        assert_eq!(tenant.id, "T-001");
        assert_eq!(tenant.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn test_get_tenant_not_found() {
        let client = InMemoryTenantClient::new();
        let result = client.get_tenant("T-999").await;
        assert!(matches!(result, Err(TenantError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_tenants_by_status() {
        let client = InMemoryTenantClient::with_tenants(vec![
            make_tenant("T-001", TenantStatus::Active, "enterprise"),
            make_tenant("T-002", TenantStatus::Suspended, "basic"),
            make_tenant("T-003", TenantStatus::Active, "basic"),
        ]);
        let filter = TenantFilter::new().status(TenantStatus::Active);
        let tenants = client.list_tenants(filter).await.unwrap();
        assert_eq!(tenants.len(), 2);
    }

    #[tokio::test]
    async fn test_list_tenants_by_plan() {
        let client = InMemoryTenantClient::with_tenants(vec![
            make_tenant("T-001", TenantStatus::Active, "enterprise"),
            make_tenant("T-002", TenantStatus::Active, "basic"),
        ]);
        let filter = TenantFilter::new().plan("enterprise");
        let tenants = client.list_tenants(filter).await.unwrap();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0].id, "T-001");
    }

    #[tokio::test]
    async fn test_is_active_true() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
        assert!(client.is_active("T-001").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_active_false() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Suspended, "basic"));
        assert!(!client.is_active("T-001").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_settings() {
        let client = InMemoryTenantClient::new();
        client.add_tenant(make_tenant("T-001", TenantStatus::Active, "basic"));
        let settings = client.get_settings("T-001").await.unwrap();
        assert_eq!(settings.get("max_users"), Some("100"));
        assert_eq!(settings.get("nonexistent"), None);
    }

    #[test]
    fn test_tenant_status_variants() {
        assert!(matches!(TenantStatus::Active, TenantStatus::Active));
        assert!(matches!(TenantStatus::Suspended, TenantStatus::Suspended));
        assert!(matches!(TenantStatus::Deleted, TenantStatus::Deleted));
    }

    #[test]
    fn test_tenant_filter_builder() {
        let filter = TenantFilter::new()
            .status(TenantStatus::Active)
            .plan("enterprise");
        assert_eq!(filter.status, Some(TenantStatus::Active));
        assert_eq!(filter.plan, Some("enterprise".to_string()));
    }

    #[test]
    fn test_tenant_settings_get() {
        let mut values = HashMap::new();
        values.insert("key1".to_string(), "value1".to_string());
        let settings = TenantSettings::new(values);
        assert_eq!(settings.get("key1"), Some("value1"));
        assert_eq!(settings.get("key2"), None);
    }

    #[test]
    fn test_config_builder() {
        use std::time::Duration;
        let config = TenantClientConfig::new("http://localhost:8080")
            .cache_ttl(Duration::from_secs(60))
            .cache_max_capacity(500);
        assert_eq!(config.server_url, "http://localhost:8080");
        assert_eq!(config.cache_ttl, Duration::from_secs(60));
        assert_eq!(config.cache_max_capacity, 500);
    }
}
