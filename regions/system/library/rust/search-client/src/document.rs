use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    pub id: String,
    pub fields: HashMap<String, serde_json::Value>,
}

impl IndexDocument {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fields: HashMap::new(),
        }
    }

    pub fn field(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(name.into(), value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    pub id: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkFailure {
    pub id: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failures: Vec<BulkFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub field_type: String,
    pub indexed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMapping {
    pub fields: HashMap<String, FieldMapping>,
}

impl IndexMapping {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn field(mut self, name: impl Into<String>, field_type: impl Into<String>) -> Self {
        self.fields.insert(
            name.into(),
            FieldMapping {
                field_type: field_type.into(),
                indexed: true,
            },
        );
        self
    }
}

impl Default for IndexMapping {
    fn default() -> Self {
        Self::new()
    }
}
