use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiSchema {
    pub name: String,
    pub description: String,
    pub schema_type: SchemaType,
    pub latest_version: u32,
    pub version_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApiSchema {
    pub fn new(name: String, description: String, schema_type: SchemaType) -> Self {
        let now = Utc::now();
        Self {
            name,
            description,
            schema_type,
            latest_version: 1,
            version_count: 1,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    #[serde(rename = "openapi")]
    OpenApi,
    #[serde(rename = "protobuf")]
    Protobuf,
}

impl std::fmt::Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaType::OpenApi => write!(f, "openapi"),
            SchemaType::Protobuf => write!(f, "protobuf"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiSchemaVersion {
    pub name: String,
    pub version: u32,
    pub schema_type: SchemaType,
    pub content: String,
    pub content_hash: String,
    pub breaking_changes: bool,
    pub breaking_change_details: Vec<BreakingChange>,
    pub registered_by: String,
    pub created_at: DateTime<Utc>,
}

impl ApiSchemaVersion {
    pub fn new(
        name: String,
        version: u32,
        schema_type: SchemaType,
        content: String,
        registered_by: String,
    ) -> Self {
        let content_hash = compute_content_hash(&content);
        Self {
            name,
            version,
            schema_type,
            content,
            content_hash,
            breaking_changes: false,
            breaking_change_details: Vec::new(),
            registered_by,
            created_at: Utc::now(),
        }
    }

    pub fn with_breaking_changes(
        mut self,
        breaking_changes: bool,
        details: Vec<BreakingChange>,
    ) -> Self {
        self.breaking_changes = breaking_changes;
        self.breaking_change_details = details;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakingChange {
    pub change_type: String,
    pub path: String,
    pub description: String,
}

impl BreakingChange {
    pub fn new(change_type: String, path: String, description: String) -> Self {
        Self {
            change_type,
            path,
            description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompatibilityResult {
    pub compatible: bool,
    pub breaking_changes: Vec<BreakingChange>,
    pub non_breaking_changes: Vec<ChangeDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeDetail {
    pub change_type: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaDiff {
    pub added: Vec<DiffEntry>,
    pub modified: Vec<DiffModifiedEntry>,
    pub removed: Vec<DiffEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffModifiedEntry {
    pub path: String,
    pub before: String,
    pub after: String,
}

pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(&result[..16]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_schema_new() {
        let schema = ApiSchema::new(
            "k1s0-tenant-api".to_string(),
            "Tenant API schema".to_string(),
            SchemaType::OpenApi,
        );
        assert_eq!(schema.name, "k1s0-tenant-api");
        assert_eq!(schema.description, "Tenant API schema");
        assert_eq!(schema.schema_type, SchemaType::OpenApi);
        assert_eq!(schema.latest_version, 1);
        assert_eq!(schema.version_count, 1);
    }

    #[test]
    fn test_api_schema_version_new() {
        let version = ApiSchemaVersion::new(
            "k1s0-tenant-api".to_string(),
            1,
            SchemaType::OpenApi,
            "openapi: 3.0.3".to_string(),
            "user-001".to_string(),
        );
        assert_eq!(version.name, "k1s0-tenant-api");
        assert_eq!(version.version, 1);
        assert!(!version.breaking_changes);
        assert!(version.breaking_change_details.is_empty());
        assert!(version.content_hash.starts_with("sha256:"));
    }

    #[test]
    fn test_api_schema_version_with_breaking_changes() {
        let version = ApiSchemaVersion::new(
            "k1s0-tenant-api".to_string(),
            2,
            SchemaType::OpenApi,
            "openapi: 3.0.3".to_string(),
            "user-001".to_string(),
        );
        let details = vec![BreakingChange::new(
            "field_removed".to_string(),
            "/api/v1/tenants GET response.properties.legacy_id".to_string(),
            "Field 'legacy_id' was removed".to_string(),
        )];
        let version = version.with_breaking_changes(true, details.clone());
        assert!(version.breaking_changes);
        assert_eq!(version.breaking_change_details.len(), 1);
        assert_eq!(
            version.breaking_change_details[0].change_type,
            "field_removed"
        );
    }

    #[test]
    fn test_schema_type_display() {
        assert_eq!(SchemaType::OpenApi.to_string(), "openapi");
        assert_eq!(SchemaType::Protobuf.to_string(), "protobuf");
    }

    #[test]
    fn test_compute_content_hash_deterministic() {
        let hash1 = compute_content_hash("openapi: 3.0.3");
        let hash2 = compute_content_hash("openapi: 3.0.3");
        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("sha256:"));
    }

    #[test]
    fn test_compute_content_hash_different_content() {
        let hash1 = compute_content_hash("openapi: 3.0.3");
        let hash2 = compute_content_hash("syntax = \"proto3\";");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_breaking_change_new() {
        let bc = BreakingChange::new(
            "type_changed".to_string(),
            "/api/v1/tenants/{id}".to_string(),
            "Type changed".to_string(),
        );
        assert_eq!(bc.change_type, "type_changed");
        assert_eq!(bc.path, "/api/v1/tenants/{id}");
    }

    #[test]
    fn test_compatibility_result() {
        let result = CompatibilityResult {
            compatible: true,
            breaking_changes: Vec::new(),
            non_breaking_changes: vec![ChangeDetail {
                change_type: "field_added".to_string(),
                path: "/api/v1/tenants GET response.properties.display_name".to_string(),
                description: "Field 'display_name' added".to_string(),
            }],
        };
        assert!(result.compatible);
        assert!(result.breaking_changes.is_empty());
        assert_eq!(result.non_breaking_changes.len(), 1);
    }

    #[test]
    fn test_schema_diff() {
        let diff = SchemaDiff {
            added: vec![DiffEntry {
                path: "/api/v1/tenants GET response.properties.display_name".to_string(),
                entry_type: "object".to_string(),
                description: "New field: display_name".to_string(),
            }],
            modified: vec![DiffModifiedEntry {
                path: "/api/v1/tenants GET summary".to_string(),
                before: "Tenants list".to_string(),
                after: "Get tenants list".to_string(),
            }],
            removed: Vec::new(),
        };
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.modified.len(), 1);
        assert!(diff.removed.is_empty());
    }
}
