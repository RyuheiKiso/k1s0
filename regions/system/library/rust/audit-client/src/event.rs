use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub tenant_id: String,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        tenant_id: impl Into<String>,
        actor_id: impl Into<String>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.into(),
            actor_id: actor_id.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            metadata,
            timestamp: Utc::now(),
        }
    }
}
