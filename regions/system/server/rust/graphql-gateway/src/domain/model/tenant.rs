use async_graphql::{Enum, SimpleObject};

#[derive(Debug, Clone, SimpleObject)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: TenantStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

impl From<String> for TenantStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ACTIVE" => TenantStatus::Active,
            "SUSPENDED" => TenantStatus::Suspended,
            "DELETED" => TenantStatus::Deleted,
            _ => TenantStatus::Active,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TenantConnection {
    pub nodes: Vec<Tenant>,
    pub total_count: i32,
    pub has_next: bool,
}
