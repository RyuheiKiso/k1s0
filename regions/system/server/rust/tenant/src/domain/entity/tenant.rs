use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum TenantStatus {
    Provisioning,
    Active,
    Suspended,
    Deleted,
}

impl TenantStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TenantStatus::Provisioning => "provisioning",
            TenantStatus::Active => "active",
            TenantStatus::Suspended => "suspended",
            TenantStatus::Deleted => "deleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    Free,
    Starter,
    Professional,
    Enterprise,
}

impl Plan {
    pub fn as_str(&self) -> &str {
        match self {
            Plan::Free => "free",
            Plan::Starter => "starter",
            Plan::Professional => "professional",
            Plan::Enterprise => "enterprise",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub status: TenantStatus,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

impl Tenant {
    pub fn new(name: String, display_name: String, plan: String, _owner_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            display_name,
            status: TenantStatus::Provisioning,
            plan,
            created_at: Utc::now(),
        }
    }

    pub fn activate(mut self) -> Self {
        self.status = TenantStatus::Active;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_new() {
        let owner = Uuid::new_v4();
        let t = Tenant::new(
            "acme-corp".to_string(),
            "ACME Corporation".to_string(),
            "professional".to_string(),
            Some(owner),
        );
        assert_eq!(t.name, "acme-corp");
        assert_eq!(t.status, TenantStatus::Provisioning);
        assert_eq!(t.plan, "professional");
    }

    #[test]
    fn test_tenant_activate() {
        let t = Tenant::new("t".to_string(), "T".to_string(), "free".to_string(), None);
        let t = t.activate();
        assert_eq!(t.status, TenantStatus::Active);
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(TenantStatus::Provisioning.as_str(), "provisioning");
        assert_eq!(TenantStatus::Active.as_str(), "active");
        assert_eq!(TenantStatus::Suspended.as_str(), "suspended");
        assert_eq!(TenantStatus::Deleted.as_str(), "deleted");
    }

    #[test]
    fn test_plan_as_str() {
        assert_eq!(Plan::Free.as_str(), "free");
        assert_eq!(Plan::Starter.as_str(), "starter");
        assert_eq!(Plan::Professional.as_str(), "professional");
        assert_eq!(Plan::Enterprise.as_str(), "enterprise");
    }
}
