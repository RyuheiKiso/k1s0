use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PolicyBundle {
    pub id: Uuid,
    pub name: String,
    pub policy_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PolicyBundle {
    pub fn new(name: String, policy_ids: Vec<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            policy_ids,
            created_at: now,
            updated_at: now,
        }
    }
}
