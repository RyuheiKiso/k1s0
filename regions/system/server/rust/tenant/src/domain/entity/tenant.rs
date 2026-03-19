use chrono::{DateTime, Utc};
use std::str::FromStr;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl FromStr for Plan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "free" => Ok(Plan::Free),
            "starter" => Ok(Plan::Starter),
            "professional" => Ok(Plan::Professional),
            "enterprise" => Ok(Plan::Enterprise),
            _ => Err(format!("unknown plan: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub status: TenantStatus,
    pub plan: Plan,
    pub owner_id: Option<String>,
    pub settings: serde_json::Value,
    pub keycloak_realm: Option<String>,
    pub db_schema: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    pub fn new(name: String, display_name: String, plan: Plan, owner_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            display_name,
            status: TenantStatus::Provisioning,
            plan,
            owner_id: owner_id.map(|id| id.to_string()),
            settings: serde_json::json!({}),
            keycloak_realm: None,
            db_schema: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub fn activate(mut self) -> Self {
        self.status = TenantStatus::Active;
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_new() {
        let owner = Uuid::new_v4();
        let t = Tenant::new(
            "acme-corp".to_string(),
            "ACME Corporation".to_string(),
            Plan::Professional,
            Some(owner),
        );
        assert_eq!(t.name, "acme-corp");
        assert_eq!(t.status, TenantStatus::Provisioning);
        assert_eq!(t.plan, Plan::Professional);
    }

    #[test]
    fn test_tenant_activate() {
        let t = Tenant::new("t".to_string(), "T".to_string(), Plan::Free, None);
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

    #[test]
    fn test_plan_from_str() {
        assert_eq!(Plan::from_str("free").unwrap(), Plan::Free);
        assert!(Plan::from_str("invalid").is_err());
    }
}
