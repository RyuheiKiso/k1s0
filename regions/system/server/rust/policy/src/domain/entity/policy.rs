use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub version: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Policy {
    pub fn new(name: String, description: String, rego_content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            rego_content,
            version: 1,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
