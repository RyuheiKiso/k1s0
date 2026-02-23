use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: TenantStatus,
    pub plan: String,
    pub settings: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct TenantFilter {
    pub status: Option<TenantStatus>,
    pub plan: Option<String>,
}

impl TenantFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: TenantStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn plan(mut self, plan: impl Into<String>) -> Self {
        self.plan = Some(plan.into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantSettings {
    pub values: HashMap<String, String>,
}

impl TenantSettings {
    pub fn new(values: HashMap<String, String>) -> Self {
        Self { values }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|v| v.as_str())
    }
}
