use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub id: Uuid,
    pub name: String,
    pub mapping: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl SearchIndex {
    pub fn new(name: String, mapping: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            mapping,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub index_name: String,
    pub query: String,
    pub from: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub total: u64,
    pub hits: Vec<SearchDocument>,
}
