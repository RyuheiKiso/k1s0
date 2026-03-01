use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::value_object::relationship_type::RelationshipType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRelationship {
    pub id: Uuid,
    pub source_table_id: Uuid,
    pub source_column: String,
    pub target_table_id: Uuid,
    pub target_column: String,
    pub relationship_type: RelationshipType,
    pub display_name: Option<String>,
    pub is_cascade_delete: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableRelationship {
    pub source_table: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
    pub relationship_type: RelationshipType,
    pub display_name: Option<String>,
    pub is_cascade_delete: Option<bool>,
}
